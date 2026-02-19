use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::TypedHeader;
use headers::{Authorization, authorization::Bearer};

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
