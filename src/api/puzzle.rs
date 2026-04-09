use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    extract::{Path, State},
};
use uuid::Uuid;

use crate::{
    authentication::Authenticated,
    database::Pool,
    discrimination::Discriminate,
    error::ErrorResponse,
    model::puzzle::{Category, Puzzle},
};

/// Get the puzzle's info
///
/// # Errors
/// Will return an error if the puzzle doesn't exist, if the user is unauthenticated or if they're
/// not on an authorized network.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/puzzle/{id}", fields(method = "GET"))]
pub async fn get(
    State(pool): State<Arc<Pool>>,
    Authenticated { .. }: Authenticated,
    _: Discriminate,
    Path(id): Path<Uuid>,
) -> Result<Json<Puzzle>, ErrorResponse> {
    Ok(Json(Puzzle::get(id, pool.get().await?).await?))
}

/// Get the all of the puzzles' info
///
/// # Errors
/// Will return an error if the user is unauthenticated or if they're not on an authorized network.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/puzzle/all", fields(method = "GET"))]
pub async fn get_all(
    State(pool): State<Arc<Pool>>,
    Authenticated { .. }: Authenticated,
    _: Discriminate,
) -> Result<Json<Vec<Puzzle>>, ErrorResponse> {
    Ok(Json(Puzzle::get_all(pool.get().await?).await?))
}

/// Get the all of the puzzles' info, grouped by category
///
/// # Errors
/// Will return an error if the user is unauthenticated or if they're not on an authorized network.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/puzzle/all/by_category", fields(method = "GET"))]
pub async fn get_all_by_category(
    State(pool): State<Arc<Pool>>,
    Authenticated { .. }: Authenticated,
    _: Discriminate,
) -> Result<Json<HashMap<Category, Vec<Puzzle>>>, ErrorResponse> {
    let puzzles = Puzzle::get_all(pool.get().await?).await?;
    let mut map: HashMap<Category, Vec<Puzzle>> = HashMap::new();

    for puzzle in puzzles {
        for category in &puzzle.categories.0 {
            if let Some(set) = map.get_mut(category) {
                set.push(puzzle.clone());
            } else {
                map.insert(*category, vec![puzzle.clone()]);
            }
        }
    }

    Ok(Json(map))
}
