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
    authentication::{Claims, authenticate},
    database::Pool,
    error::ErrorResponse,
    model::user::User,
};

/// Create the row in the public.User table
///
/// # Errors
/// Will return an error if the user object is malformed, if it's not unique, if there's an issue
/// communicating with the database, or if authentication fails.
pub async fn sign_up(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Json(user): Json<User>,
) -> Result<StatusCode, ErrorResponse> {
    tracing::trace!("/v0/user/sign_up endpoint called");

    let Claims { sub, .. } = authenticate(bearer.token())?;
    _ = user.create(sub, pool.get().await?).await?;

    Ok(StatusCode::OK)
}

// Wow! I don't know what I was thinking.
// /// Set the check in timestamp for the user
// ///
// /// # Errors
// /// Will return an error if the user had already been checked in, if it doesn't exists, if there's
// /// an issue communicating with the database, or if authentication fails.
// pub async fn check_in(
//     State(pool): State<Arc<Pool>>,
//     TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
// ) -> Result<StatusCode, ErrorResponse> {
//     tracing::trace!("/v0/user/check_in endpoint called");
//
//     let Claims { sub, .. } = authenticate(bearer.token())?;
//
//     match User::check_in(sub, pool.get().await?).await? {
//         0 => Err(ErrorResponse::from(exn::Exn::new(Error::database(
//             crate::error::DatabaseError::ConstraintViolation,
//             "The user doesn't exist or it has already been checked in".into(),
//         )))),
//         1 => Ok(StatusCode::OK),
//         n => {
//             tracing::warn!("{n} rows were updated when trying to check in a user.");
//
//             Err(ErrorResponse::from(exn::Exn::new(Error::database(
//                 crate::error::DatabaseError::Unknown,
//                 format!(
//                     "Something went horribly wrong when trying to check_in user {sub} at {}. Please contact an administrator as soon as possible",
//                     Utc::now()
//                 ),
//             ))))
//         }
//     }
// }
//
// let mut decoded: [u8; 16] = [0; 16];
// BASE64_STANDARD_NO_PAD.decode_slice(encoded_sub, &mut decoded).unwrap();
// let uuid = uuid::Uuid::from_bytes(decoded);
// println!("Decoded: {}", uuid);

#[derive(serde::Deserialize)]
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
#[allow(clippy::unused_async)]
pub async fn qr(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Query(style): Query<Colors>,
) -> Result<Response, ErrorResponse> {
    tracing::trace!("/v0/user/qr.svg");

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
