use std::{collections::HashMap, sync::Arc};

use axum::{Json, extract::State};

use crate::{
    api::Id,
    authentication::{ADMIN_ROLE, Authenticated},
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
#[tracing::instrument(skip_all, name = "/v0/puzzle", fields(method = "GET"))]
pub async fn get(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, role }: Authenticated,
    _: Discriminate,
    Id(id): Id,
) -> Result<Json<Puzzle>, ErrorResponse> {
    let mut puzzle = Puzzle::get(id, pool.get().await?).await?;

    if role != ADMIN_ROLE {
        puzzle.hide_unused_clues(sub, pool.get().await?).await?;
    }

    Ok(Json(puzzle))
}

/// Get the all of the puzzles' info
///
/// # Errors
/// Will return an error if the user is unauthenticated or if they're not on an authorized network.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/puzzle/all", fields(method = "GET"))]
pub async fn get_all(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, role }: Authenticated,
    _: Discriminate,
) -> Result<Json<Vec<Puzzle>>, ErrorResponse> {
    let mut puzzles = Puzzle::get_all(pool.get().await?).await?;

    if role == ADMIN_ROLE {
        return Ok(Json(puzzles));
    }

    // This could be parallelized. For now, there aren't enough puzzles to warrant the effort.
    for puzzle in &mut puzzles {
        puzzle.hide_unused_clues(sub, pool.get().await?).await?;
    }

    Ok(Json(puzzles))
}

/// Get the all of the puzzles' info, grouped by category
///
/// # Errors
/// Will return an error if the user is unauthenticated or if they're not on an authorized network.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/puzzle/all/by_category", fields(method = "GET"))]
pub async fn get_all_by_category(
    State(pool): State<Arc<Pool>>,
    Authenticated { sub, role }: Authenticated,
    _: Discriminate,
) -> Result<Json<HashMap<Category, Vec<Puzzle>>>, ErrorResponse> {
    let mut puzzles = Puzzle::get_all(pool.get().await?).await?;
    let mut map: HashMap<Category, Vec<Puzzle>> = HashMap::new();

    // This could be parallelized. For now, there aren't enough puzzles to warrant the effort.
    for puzzle in &mut puzzles {
        if role != ADMIN_ROLE {
            puzzle.hide_unused_clues(sub, pool.get().await?).await?;
        }

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
