use std::sync::Arc;

use axum::{Json, extract::State};
use chrono::Utc;

use crate::{
    State as Bstate,
    authentication::Authenticated,
    config::CONFIG,
    error::ErrorResponse,
    model::{
        attempt::{Attempt, ScoreboardEntry},
        team::Team,
    },
};

/// Get the puzzle's info
///
/// # Errors
/// Will return an error if the puzzle doesn't exist, if the user is unauthenticated, if they're
/// not on an authorized network or if the query is malformed.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/puzzle", fields(method = "GET"))]
pub async fn get(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, role }: Authenticated,
) -> Result<Json<Vec<ScoreboardEntry>>, ErrorResponse> {
    let team = if Utc::now() >= CONFIG.scoreboard_hides && role != CONFIG.jwk.admin_role {
        Some(Team::id(sub, state.get_connection().await?).await?)
    } else {
        None
    };

    Ok(Json(
        Attempt::scoreboard(team, state.get_connection().await?).await?,
    ))
}
