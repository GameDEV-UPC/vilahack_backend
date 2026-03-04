use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::TypedHeader;
use chrono::Utc;
use headers::{Authorization, authorization::Bearer};

use crate::{
    authentication::{Claims, authenticate},
    database::Pool,
    error::{Error, ErrorResponse},
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

/// Set the check in timestamp for the user
///
/// # Errors
/// Will return an error if the user had already been checked in, if it doesn't exists, if there's
/// an issue communicating with the database, or if authentication fails.
pub async fn check_in(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<StatusCode, ErrorResponse> {
    tracing::trace!("/v0/user/check_in endpoint called");

    let Claims { sub, .. } = authenticate(bearer.token())?;

    match User::check_in(sub, pool.get().await?).await? {
        0 => Err(ErrorResponse::from(exn::Exn::new(Error::database(
            crate::error::DatabaseError::ConstraintViolation,
            "The user doesn't exist or it has already been checked in".into(),
        )))),
        1 => Ok(StatusCode::OK),
        n => {
            tracing::warn!("{n} rows were updated when trying to check in a user.");

            Err(ErrorResponse::from(exn::Exn::new(Error::database(
                crate::error::DatabaseError::Unknown,
                format!(
                    "Something went horribly wrong when trying to check_in user {sub} at {}. Please contact an administrator as soon as possible",
                    Utc::now()
                ),
            ))))
        }
    }
}
