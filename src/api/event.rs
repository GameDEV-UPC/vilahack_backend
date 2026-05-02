use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    State as Bstate,
    api::Id,
    authentication::Authenticated,
    config::CONFIG,
    error::ErrorResponse,
    model::event::{Event, Participate, Participation},
};

/// Get the event's details.
///
/// # Errors
/// Will return an error if the event doesn't exist.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/event", fields(method = "GET"))]
pub async fn get(
    State(state): State<Arc<Bstate>>,
    Id(id): Id,
) -> Result<Json<Event>, ErrorResponse> {
    Ok(Json(Event::get(id, state.get_connection().await?).await?))
}

/// Get all the events, sorted by begin date
///
/// # Errors
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/event/all", fields(method = "GET"))]
pub async fn all(State(state): State<Arc<Bstate>>) -> Result<Json<Vec<Event>>, ErrorResponse> {
    Ok(Json(Event::all(state.get_connection().await?).await?))
}

/// Set the check in timestamp for the user
///
/// # Errors
/// Will return an error if the caller is not an admin and if the user or event don't exist.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/event/participation", fields(method = "PUT"))]
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

/// Set the check in timestamp for the user
///
/// # Errors
/// Will return an error if the caller is not an admin or if the user doesn't exist.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/event/participation/all", fields(method = "GET"))]
pub async fn participations(
    State(state): State<Arc<Bstate>>,
    Authenticated { role, .. }: Authenticated,
    Id(id): Id,
) -> Result<Json<Vec<Participation>>, ErrorResponse> {
    if role != CONFIG.jwk.admin_role {
        return Err(ErrorResponse::insufficient_permissions());
    }

    Ok(Json(
        Participation::get(id, state.get_connection().await?).await?,
    ))
}
