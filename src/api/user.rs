use std::sync::Arc;

use axum::{
    Json, body,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::Response,
};

use base64::prelude::{BASE64_STANDARD_NO_PAD, Engine};
use fast_qr::{
    convert::{Builder, Shape, svg::SvgBuilder},
    qr::QRBuilder,
};

use crate::{
    State as Bstate,
    api::{Id, OptionalId},
    authentication::Authenticated,
    config::CONFIG,
    error::ErrorResponse,
    model::{
        application::{Application, ApplicationSummary, ApplicationUpdate, Status},
        event::Participate,
    },
};

/// Create the row in the Application table
///
/// # Errors
/// Will return an error if the application object is malformed, if the user is unauthenticated or
/// if they've already applied.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/application", fields(method = "PUT"))]
pub async fn apply(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, .. }: Authenticated,
    Json(application): Json<Application>,
) -> Result<StatusCode, ErrorResponse> {
    _ = application
        .create(sub, state.get_connection().await?)
        .await?;
    Ok(StatusCode::OK)
}

/// Returns the user's application
///
/// If the caller is an admin and they provided an id on the query,
/// the data of the user with that id will be returned. Otherwise, the data of the caller's JWT
/// subject will be returned.
///
/// # Errors
/// Will return an error if the user hasn't made an application, if the queries are malformed or if
/// the user is unauthenticated.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/application", fields(method = "GET"))]
pub async fn get(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, role }: Authenticated,
    OptionalId(id): OptionalId,
) -> Result<Json<Application>, ErrorResponse> {
    // If the caller is an admin and they've provided an id, use that id. Otherwise use the
    // JWT's subject
    let id = match (role == CONFIG.jwk.admin_role, id) {
        (true, Some(id)) => {
            tracing::info!(target: "privacy", organizer = sub.to_string(), participant = id.to_string());
            id
        }
        _ => sub,
    };

    Ok(Json(
        Application::get(id, state.get_connection().await?).await?,
    ))
}

/// Updates the user's application
///
/// # Errors
/// Will return an error if the user hasn't made an application or if they're unauthenticated
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/application/update", fields(method = "PUT"))]
pub async fn update(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, .. }: Authenticated,
    Json(updated): Json<ApplicationUpdate>,
) -> Result<(), ErrorResponse> {
    let _ = updated.update(sub, state.get_connection().await?).await?;
    Ok(())
}

/// Get a summarized list of all applications
///
/// # Errors
/// Will return an error if the user is not authenticated as an admin.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/application/index", fields(method = "GET"))]
pub async fn index(
    State(state): State<Arc<Bstate>>,
    Authenticated { role, .. }: Authenticated,
) -> Result<Json<Vec<ApplicationSummary>>, ErrorResponse> {
    if role != CONFIG.jwk.admin_role {
        return Err(ErrorResponse::insufficient_permissions());
    }

    Ok(Json(
        Application::index(state.get_connection().await?).await?,
    ))
}

/// Change the application status from `applied` to `accepted`
///
/// # Errors
/// Will return an erro if the user doesn't have an application on the `applied` state and if the
/// caller is not an authenticated admin
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/attendance/accept", fields(method = "PUT"))]
pub async fn accept_attendance(
    State(state): State<Arc<Bstate>>,
    Authenticated { role, .. }: Authenticated,
    Id(id): Id,
) -> Result<(), ErrorResponse> {
    if role != CONFIG.jwk.admin_role {
        return Err(ErrorResponse::insufficient_permissions());
    }

    Application::change_status(
        id,
        vec![Status::Applied],
        Status::Accepted,
        state.get_connection().await?,
    )
    .await?;

    Ok(())
}

/// Change the application status from `accepted` to `confirmed`
///
/// # Errors
/// Will return an error if the user hasn't made an application or if they're not in the `accepted`
/// state. Will return an error if the caller is not authenticated.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/attendance/confirm", fields(method = "PUT"))]
pub async fn confirm_attendance(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, .. }: Authenticated,
) -> Result<(), ErrorResponse> {
    Application::change_status(
        sub,
        vec![Status::Accepted],
        Status::Confirmed,
        state.get_connection().await?,
    )
    .await?;

    Ok(())
}

/// Change the application status from `confirmed` to `accepted`
///
/// Users that are checked in can technically change their attendance commitment,
/// but I don't think that's an issue really.
///
/// # Errors
/// Will return an error if the user hasn't made an application, if they're not in the `confirmed`
/// state or if they're unauthenticated.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/attendance/cancel", fields(method = "PUT"))]
pub async fn cancel_attendance(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, .. }: Authenticated,
) -> Result<(), ErrorResponse> {
    Application::change_status(
        sub,
        vec![Status::Confirmed, Status::Accepted],
        Status::Cancelled,
        state.get_connection().await?,
    )
    .await?;

    Ok(())
}

/// Set the check in timestamp for the user
///
/// # Errors
/// Will return an error if the user hasn't made an application, if they've already checked in or
/// if they're not accepted. Will also return an error if the caller is not an admin.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(
    skip_all,
    name = "/v0/user/attendance/check_in",
    fields(method = "PUT")
)]
pub async fn check_in(
    State(state): State<Arc<Bstate>>,
    Authenticated { role, sub }: Authenticated,
    Id(id): Id,
) -> Result<(), ErrorResponse> {
    if role != CONFIG.jwk.admin_role {
        return Err(ErrorResponse::insufficient_permissions());
    }

    Application::check_in(id, state.get_connection().await?).await?;

    tracing::info!(target: "check_in", organizer = sub.to_string(), participant = id.to_string());

    Ok(())
}

#[derive(serde::Deserialize, Debug)]
pub struct Colors {
    #[serde(rename = "m")]
    module: Option<String>,
    #[serde(rename = "b")]
    background: Option<String>,
}

/// Returns an SVG of the QR code that identifies
///
/// # Errors
/// Will return an error if the queries are malformed or if authentication fails
///
/// # Panics
/// Never, should be infallible
/// (Technically, it can panic if the hardcoded string "`image/svg+xml`" stops being considered
/// ASCII or if it stops being considered a valid value for the `CONTENT_TYPE` header.)
#[tracing::instrument(skip_all, name = "/v0/user/qr.svg", fields(method = "GET"))]
pub async fn qr(
    Authenticated { sub, .. }: Authenticated,
    Query(style): Query<Colors>,
) -> Result<Response, ErrorResponse> {
    let Ok(qr) = QRBuilder::new(BASE64_STANDARD_NO_PAD.encode(sub.as_bytes())).build() else {
        todo!()
    };

    let qr = SvgBuilder::default()
        .shape(Shape::Square)
        .module_color(style.module.unwrap_or_else(|| "#000000ff".into()))
        .background_color(style.background.unwrap_or_else(|| "#ffffffff".into()))
        .to_str(&qr);

    // These expects _should_ be infallible
    Ok(Response::builder()
        .status(200)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_str("image/svg+xml")
                .expect("ASCII is broken! Call the fire department!"),
        )
        .body(body::Body::from(qr))
        .expect("HTTP headers are broken! The web is in shambles."))
}

/// Set the check in timestamp for the user
///
/// # Errors
/// Will return an error if the user hasn't made an application, if they've already checked in or
/// if they're not accepted. Will also return an error if the caller is not an admin.
/// Might return an error if there's an issue communicating with the database.
#[axum::debug_handler]
#[tracing::instrument(skip_all, name = "/v0/user/participate", fields(method = "PUT"))]
pub async fn participate(
    State(state): State<Arc<Bstate>>,
    Authenticated { role, .. }: Authenticated,
    participate: Participate,
) -> Result<Json<usize>, ErrorResponse> {
    if role != CONFIG.jwk.admin_role {
        return Err(ErrorResponse::insufficient_permissions());
    }

    Ok(Json(participate.post(state.get_connection().await?).await?))
}
