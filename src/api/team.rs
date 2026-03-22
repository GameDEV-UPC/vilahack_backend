use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use axum_extra::TypedHeader;
use headers::{Authorization, authorization::Bearer};

use base64::prelude::{BASE64_STANDARD_NO_PAD, Engine};

use crate::{
    authentication::{Claims, authenticate},
    database::Pool,
    error::ErrorResponse,
    model::team::{Team, TeamSummary},
};

/// Get info about the team the caller belongs to
///
/// # Errors
/// Will return an error if the user doesn't belong to any team, or if they fail
/// to authenticate. May return an error if there's an issue communicating with the database
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip_all, name = "/v0/team", fields(method = "GET"))]
pub async fn summary(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<TeamSummary>, ErrorResponse> {
    let Claims { sub, .. } = authenticate(bearer.token())?;

    Ok(Json(Team::summary(sub, pool.get().await?).await?))
}

/// Create a new team and immediately join it
///
/// # Errors
/// Will return an error if the user doesn't exist, if they already belong to a team or if they
/// fail to authenticate. May return an error if there's an issue communicating with the database.
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip_all, name = "/v0/team", fields(method = "PUT"))]
pub async fn new(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(name): Path<String>,
) -> Result<Json<String>, ErrorResponse> {
    let Claims { sub, .. } = authenticate(bearer.token())?;

    let team = Team::new(name, sub, pool.get().await?).await?;
    Ok(Json(BASE64_STANDARD_NO_PAD.encode(team.id.as_bytes())))
}

/// Join a team
///
/// # Errors
/// Will return an error if the team doesn't exist, if the team is full or if the user
/// fails to authenticate. Will also return an error if the request is malformed (i.e the group id
/// cannot be decoded.) May return an error if there's an issue communicating with the database.
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip_all, name = "/v0/team/join", fields(method = "PUT"))]
pub async fn join(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(id): Path<String>,
) -> Result<Json<TeamSummary>, ErrorResponse> {
    let Claims { sub, .. } = authenticate(bearer.token())?;

    let mut decoded: [u8; 16] = [0; 16];
    if BASE64_STANDARD_NO_PAD
        .decode_slice(id, &mut decoded)
        .is_err()
    {
        decoded = [0; 16]; // Make a nil uuid if it fails to decode
    }

    let id = uuid::Uuid::from_bytes(decoded);
    Team::join(sub, id, pool.get().await?).await?;

    Ok(Json(Team::summary(sub, pool.get().await?).await?))
}

/// Leave whichever team the user is currently joined to
///
/// # Errors
/// Will return an error if the user is not in a team or if they fail to authenticate. May return
/// an error if there's an issue communicating to the database.
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip_all, name = "/v0/team/leave", fields(method = "PUT"))]
pub async fn leave(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<(), ErrorResponse> {
    let Claims { sub, .. } = authenticate(bearer.token())?;

    Ok(Team::leave(sub, pool.get().await?).await?)
}

/// Update the name of whichever team the user is joined to
///
/// # Errors
/// Will return an error if the user is not in a team or if they fail to authenticate. May return
/// an error if there's an issue communicating to the database.
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip_all, name = "/v0/team/update", fields(method = "PUT"))]
pub async fn update(
    State(pool): State<Arc<Pool>>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    Path(name): Path<String>,
) -> Result<(), ErrorResponse> {
    let Claims { sub, .. } = authenticate(bearer.token())?;

    Ok(Team::update(sub, name, pool.get().await?).await?)
}
