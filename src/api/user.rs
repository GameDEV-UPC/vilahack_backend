use std::sync::Arc;

use axum::{
    Json, body,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::Response,
};
use axum_extra::TypedHeader;
use headers::{Authorization, authorization::Bearer};

use base64::prelude::{BASE64_STANDARD_NO_PAD, Engine};
use fast_qr::{
    convert::{Builder, Shape, svg::SvgBuilder},
    qr::QRBuilder,
};

use crate::{
    authentication::{ADMIN_ROLE, Claims, authenticate},
    database::Pool,
    error::{AuthenticationError, Error, ErrorResponse},
    model::user::User,
};

#[derive(serde::Deserialize, Debug)]
pub struct UidQuery {
    id: Option<uuid::Uuid>,
    qr: Option<String>,
}

impl UidQuery {
    fn get(&self) -> Option<uuid::Uuid> {
        if self.id.is_some() {
            self.id
        } else if let Some(encoded) = &self.qr {
            let mut decoded: [u8; 16] = [0; 16];
            if BASE64_STANDARD_NO_PAD
                .decode_slice(encoded, &mut decoded)
                .is_err()
            {
                return None;
            }
            Some(uuid::Uuid::from_bytes(decoded))
        } else {
            None
        }
    }
}

/// Create the row in the public.User table
///
/// # Errors
/// Will return an error if the user object is malformed, if it's not unique, if there's an issue
/// communicating with the database, or if authentication fails.
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip_all, name = "/v0/user/sign_up", fields(method = "PUT"))]
pub async fn sign_up(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(user): Json<User>,
) -> Result<StatusCode, ErrorResponse> {
    let Claims { sub, .. } = authenticate(bearer.token())?;
    _ = user.create(sub, pool.get().await?).await?;

    Ok(StatusCode::OK)
}

/// Set the check in timestamp for the user
///
/// # Errors
/// Will return an error if the user had already been checked in, if it doesn't exist, if the user
/// id to be ckecked in wasn't passed, if there's an issue communicating with the database,
/// or if authentication fails.
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip(pool, bearer), name = "/v0/user/check_in", fields(method = "PUT"))]
pub async fn check_in(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Query(uid): Query<UidQuery>,
) -> Result<(), ErrorResponse> {
    let Claims { role, .. } = authenticate(bearer.token())?;
    if role != ADMIN_ROLE {
        return Err(ErrorResponse::from(exn::Exn::new(Error::authentication(
            AuthenticationError::InsufficientPermissions,
            "This user is not authorized to do this operation".into(),
        ))));
    }

    // I'm going all the way to the database with a thing that I know will cause an error. This is
    // bad on resources. But realistically will never happen, or at least not often. Since this API
    // will be called by a frontend, the frontend will always include the user id.
    // I'm doing it this way because it really simplifies things on the backend.
    let uid = uid.get().unwrap_or_default();

    User::check_in(uid, pool.get().await?).await?;
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
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip_all, name = "/v0/user/qr.svg", fields(method = "GET"))]
pub async fn qr(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Query(style): Query<Colors>,
) -> Result<Response, ErrorResponse> {
    let Claims { sub, .. } = authenticate(bearer.token())?;

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

/// Returns all of the user's data.
///
/// If the caller is an admin and they provided a uid on the query,
/// the data of the user with that uid will be returned. Otherwise, the data of the caller's JWT
/// subject will be returned.
///
/// # Errors
/// Will return an error if the user being fetched doesn't exist or doesn't have an entry
/// associated with it. It will also return an error if the queries are malformed, if
/// authentication fails or if there's an issue communicating with the database.
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip_all, name = "/v0/user", fields(method = "GET"))]
pub async fn get(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Query(uid): Query<UidQuery>,
) -> Result<Json<User>, ErrorResponse> {
    let Claims { sub, role } = authenticate(bearer.token())?;

    // If the caller is an admin and they've provided a uid, use that uid. Otherwise use the
    // JWT's subject
    let uid = match (role == ADMIN_ROLE, uid.get()) {
        (true, Some(uid)) => {
            tracing::trace!(target: "privacy", organizer = sub.to_string(), participant = uid.to_string());
            uid
        }
        _ => sub,
    };

    Ok(Json(User::get(uid, pool.get().await?).await?))
}
