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
    api::{OptionalUid, Uid},
    authentication::{ADMIN_ROLE, Authenticated},
    database::Pool,
    error::ErrorResponse,
    model::application::{Application, ApplicationSummary, ApplicationUpdate},
};

/// Create the row in the Application table
///
/// # Errors
/// Will return an error if the application object is malformed, if the user is unauthenticated or
/// if they've already applied.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/application", fields(method = "PUT"))]
pub async fn apply(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, .. }: Authenticated,
    Json(application): Json<Application>,
) -> Result<StatusCode, ErrorResponse> {
    _ = application.create(sub, pool.get().await?).await?;
    Ok(StatusCode::OK)
}

/// Returns the user's application
///
/// If the caller is an admin and they provided a uid on the query,
/// the data of the user with that uid will be returned. Otherwise, the data of the caller's JWT
/// subject will be returned.
///
/// # Errors
/// Will return an error if the user hasn't made an application, if the queries are malformed or if
/// the user is unauthenticated.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/application", fields(method = "GET"))]
pub async fn get(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, role }: Authenticated,
    OptionalUid(uid): OptionalUid,
) -> Result<Json<Application>, ErrorResponse> {
    // If the caller is an admin and they've provided a uid, use that uid. Otherwise use the
    // JWT's subject
    let uid = match (role == ADMIN_ROLE, uid) {
        (true, Some(uid)) => {
            tracing::info!(target: "privacy", organizer = sub.to_string(), participant = uid.to_string());
            uid
        }
        _ => sub,
    };

    Ok(Json(Application::get(uid, pool.get().await?).await?))
}

/// Updates the user's application
///
/// # Errors
/// Will return an error if the user hasn't made an application or if they're unauthenticated
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/application/update", fields(method = "PUT"))]
pub async fn update(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, .. }: Authenticated,
    Json(updated): Json<ApplicationUpdate>,
) -> Result<(), ErrorResponse> {
    let _ = updated.update(sub, pool.get().await?).await?;
    Ok(())
}

/// Get a summarized list of all applications
///
/// # Errors
/// Will return an error if the user is not authenticated as an admin.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/application/index", fields(method = "GET"))]
pub async fn index(
    State(pool): State<Arc<Pool>>,
    Authenticated { role, .. }: Authenticated,
) -> Result<Json<Vec<ApplicationSummary>>, ErrorResponse> {
    if role != ADMIN_ROLE {
        return Err(ErrorResponse::insufficient_permissions());
    }

    Ok(Json(Application::index(pool.get().await?).await?))
}

/// Set the check in timestamp for the user
///
/// # Errors
/// Will return an error if the user hasn't made an applicationm if they've already checked in or
/// if they're not accepted. Will also return an error if the caller is not an admin.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/user/check_in", fields(method = "PUT"))]
pub async fn check_in(
    State(pool): State<Arc<Pool>>,
    Authenticated { role, sub }: Authenticated,
    Uid(uid): Uid,
) -> Result<(), ErrorResponse> {
    if role != ADMIN_ROLE {
        return Err(ErrorResponse::insufficient_permissions());
    }

    Application::check_in(uid, pool.get().await?).await?;

    tracing::info!(target: "check_in", organizer = sub.to_string(), participant = uid.to_string());

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
