use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, MutexGuard},
    time::Duration,
};

use axum::{Json, extract::State};
use chrono::Utc;
use uuid::Uuid;

use tokio::task::JoinHandle;

use crate::{
    State as Bstate,
    api::Id,
    authentication::Authenticated,
    config::CONFIG,
    discrimination::Discriminate,
    error::{Error, ErrorResponse},
    model::{
        attempt::Attempt,
        puzzle::{Category, Puzzle},
        team::Team,
    },
};

/// Get the puzzle's info
///
/// # Errors
/// Will return an error if the puzzle doesn't exist, if the user is unauthenticated or if they're
/// not on an authorized network.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/puzzle", fields(method = "GET"))]
pub async fn get(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, role }: Authenticated,
    _: Discriminate,
    Id(id): Id,
) -> Result<Json<Puzzle>, ErrorResponse> {
    let mut puzzle = Puzzle::get(id, state.get_connection().await?).await?;

    if role != CONFIG.jwk.admin_role {
        puzzle
            .hide_unused_clues(sub, state.get_connection().await?)
            .await?;
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
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, role }: Authenticated,
    _: Discriminate,
) -> Result<Json<Vec<Puzzle>>, ErrorResponse> {
    let mut puzzles = Puzzle::get_all(state.get_connection().await?).await?;

    if role == CONFIG.jwk.admin_role {
        return Ok(Json(puzzles));
    }

    // This could be parallelized. For now, there aren't enough puzzles to warrant the effort.
    for puzzle in &mut puzzles {
        puzzle
            .hide_unused_clues(sub, state.get_connection().await?)
            .await?;
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
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, role }: Authenticated,
    _: Discriminate,
) -> Result<Json<HashMap<Category, Vec<Puzzle>>>, ErrorResponse> {
    let mut puzzles = Puzzle::get_all(state.get_connection().await?).await?;
    let mut map: HashMap<Category, Vec<Puzzle>> = HashMap::new();

    // This could be parallelized. For now, there aren't enough puzzles to warrant the effort.
    for puzzle in &mut puzzles {
        if role != CONFIG.jwk.admin_role {
            puzzle
                .hide_unused_clues(sub, state.get_connection().await?)
                .await?;
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

// async fn check_finished(
//     handle: &JoinHandle<Result<PathBuf, Error>>,
//     mut tasks: MutexGuard<'_, HashMap<(Uuid, Uuid), JoinHandle<Result<PathBuf, exn::Exn<Error>>>>>,
//     key: &(Uuid, Uuid),
// ) -> exn::Result<PathBuf, Error> {
//     if handle.is_finished() {
//         let handle = tasks.remove(key).expect("Mutex got poisoned");
//         drop(tasks);
// 
//         Ok(handle.await.expect("Should never panic")?)
//     } else {
//         todo!(); // TODO Warn the client that the task ain't done running
//     }
// }

// /// Get the puzzle's archive
// ///
// /// # Errors
// /// Will return an error if the puzzle doesn't exist, if the user is unauthenticated or if they're
// /// not on an authorized network.
// /// Might return an error if there's an issue communicating with the database.
// #[tracing::instrument(skip_all, name = "/v0/puzzle/files", fields(method = "GET"))]
// pub async fn files(
//     State(state): State<Arc<Bstate>>,
//     Authenticated { sub, .. }: Authenticated,
//     _: Discriminate,
//     Id(id): Id,
// ) -> Result<(), ErrorResponse> {
//     let team = Team::id(sub, state.get_connection().await?).await?;
// 
//     Attempt {
//         puzzle: id,
//         team,
//         created_at: Utc::now(),
//         ..Default::default()
//     }
//     .begin(state.get_connection().await?)
//     .await?;
// 
//     let Ok(mut tasks) = state.tasks.lock() else {
//         todo!();
//     };
// 
//     let mut path: Option<PathBuf> = None;
// 
//     match tasks.get(&(id, team)) {
//         // If the task is already runing, one shouldn't be spawned.
//         Some(handle) => {
//             check_finished(handle, tasks, &(id, team)).await?;
//         }
// 
//         // If the task isn't already running, spawn it.
//         None => {
//             // The handle is put directly in the hashmap before waiting to avoid races
//             tasks.insert((id, team), tokio::spawn(Puzzle::files(team, id)));
//             drop(tasks);
// 
//             // Wait 100ms for it to run. If the files already exist, this will save the client from
//             // making another call
//             match tokio::time::timeout(Duration::from_millis(100), async {
//                 let mut tasks = state.tasks.lock().expect("Mutex got poisoned");
// 
//                 // We expect the handle to still be present under the same key.
//                 if let Some(handle) = tasks.get(&(id, team)) {
//                     if !handle.is_finished() {
//                         let handle = tasks.remove(&(id, team)).expect("key existed");
//                         drop(tasks);
//                         Some(handle.await.expect("Should never panic"))
//                     } else {
//                         // Not finished yet
//                         None
//                     }
//                 } else {
//                     // Not found (shouldn't normally happen) — treat as absent
//                     None
//                 }
//             })
//             .await
//             {
//                 Ok(Some(res)) => path = Some(res?),
//                 _ => todo!(), // Tell the client to wait
//             }
//         }
//     };
// 
//     Ok(())
// }
