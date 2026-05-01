use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    State as Bstate, authentication::Authenticated, config::CONFIG, error::ErrorResponse,
    model::event::{Event, Participate},
};

/// Get all the events, sorted by begin date
///
/// # Errors
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/event/all", fields(method = "PUT"))]
pub async fn all(
    State(state): State<Arc<Bstate>>,
) -> Result<Json<Vec<Event>>, ErrorResponse> {
    Ok(Json(Event::all(state.get_connection().await?).await?))
}

/// Set the check in timestamp for the user
///
/// # Errors
/// Will return an error if the user hasn't made an application, if they've already checked in or
/// if they're not accepted. Will also return an error if the caller is not an admin.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/event/participate", fields(method = "PUT"))]
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
