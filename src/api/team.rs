use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use base64::prelude::{BASE64_STANDARD_NO_PAD, Engine};

use crate::{
    api::{Id, Name},
    authentication::Authenticated,
    database::Pool,
    error::ErrorResponse,
    model::team::{Team, TeamSummary},
};

/// Get info about the team the caller belongs to
///
/// # Errors
/// Will return an error if the user doesn't belong to any team, or if they're unauthenticated.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/team", fields(method = "GET"))]
pub async fn summary(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, .. }: Authenticated,
) -> Result<Json<TeamSummary>, ErrorResponse> {
    Ok(Json(Team::summary(sub, pool.get().await?).await?))
}

/// Create a new team and immediately join it
///
/// # Errors
/// Will return an error if the user doesn't have an application, if they already belong to a team
/// or if they're unauthenticated.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/team", fields(method = "PUT"))]
pub async fn new(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, .. }: Authenticated,
    Query(name): Query<Name>,
) -> Result<Json<String>, ErrorResponse> {
    let team = Team::new(name.name, sub, pool.get().await?).await?;

    Ok(Json(BASE64_STANDARD_NO_PAD.encode(team.id.as_bytes())))
}

/// Join a team
///
/// # Errors
/// Will return an error if the user doesn't have an application, if the team is full or if the
/// user is unauthenticated. Will also return an error if the request is malformed (i.e the group
/// id cannot be decoded).
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/team/join", fields(method = "PUT"))]
pub async fn join(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, .. }: Authenticated,
    Id(id): Id,
) -> Result<Json<TeamSummary>, ErrorResponse> {
    Team::join(sub, id, pool.get().await?).await?;

    Ok(Json(Team::summary(sub, pool.get().await?).await?))
}

/// Leave whichever team the user is currently joined to
///
/// # Errors
/// Will return an error if the user is not in a team or if they're unauthenticated.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/team/leave", fields(method = "PUT"))]
pub async fn leave(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, .. }: Authenticated,
) -> Result<(), ErrorResponse> {
    Ok(Team::leave(sub, pool.get().await?).await?)
}

/// Update the name of whichever team the user is joined to
///
/// # Errors
/// Will return an error if the user is not in a team or if they're unauthenticated.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/team/update", fields(method = "PUT"))]
pub async fn update(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, .. }: Authenticated,
    Query(name): Query<Name>,
) -> Result<(), ErrorResponse> {
    Ok(Team::update(sub, name.name, pool.get().await?).await?)
}
