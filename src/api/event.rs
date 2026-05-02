use std::sync::Arc;

use axum::{Json, extract::State};

use crate::{
    State as Bstate,
    api::{Id, OptionalId},
    authentication::Authenticated,
    config::CONFIG,
    error::ErrorResponse,
    model::event::{Event, Participate, Participation},
};

/// Get the event's details.
///
/// # Errors
/// Will return an error if the event doesn't existor if the query is malformed.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/event", fields(method = "GET"))]
pub async fn get(
    State(state): State<Arc<Bstate>>,
    Id(id): Id,
) -> Result<Json<Event>, ErrorResponse> {
    Ok(Json(Event::get(id, state.get_connection().await?).await?))
}

/// Get a list of all the events, ordered by begin time
///
/// # Errors
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/event/all", fields(method = "GET"))]
pub async fn all(State(state): State<Arc<Bstate>>) -> Result<Json<Vec<Event>>, ErrorResponse> {
    Ok(Json(Event::all(state.get_connection().await?).await?))
}

/// Record the user as participating in that event
///
/// # Errors
/// Will return an error if the caller is not an admin and if the user or event don't exist, or if
/// the query is malformed
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

/// Get all the user's participations, ordered by the event's begin time
///
/// # Errors
/// Will return an error if the user doesn't exist or if the query is malformed.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/event/participation/all", fields(method = "GET"))]
pub async fn participations(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, role }: Authenticated,
    OptionalId(id): OptionalId,
) -> Result<Json<Vec<Participation>>, ErrorResponse> {
    let id = if let Some(id) = id
        && role == CONFIG.jwk.admin_role
    {
        id
    } else {
        sub
    };

    Ok(Json(
        Participation::get(id, state.get_connection().await?).await?,
    ))
}
