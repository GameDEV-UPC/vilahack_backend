use std::{collections::HashMap, sync::Arc};

use axum::{
    Json,
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, header},
    response::IntoResponse,
};

use tokio_util::io::ReaderStream;

use crate::{
    State as Bstate,
    api::{FlagCheckQuery, Id},
    authentication::Authenticated,
    config::CONFIG,
    discrimination::Discriminate,
    error::ErrorResponse,
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

/// Get the puzzle's archive
///
/// # Errors
/// Will return an error if the puzzle doesn't exist, if the user is unauthenticated or if they're
/// not on an authorized network.
/// Might return an error if there's an issue communicating with the database, if the generator
/// fails to run or it there's any io issue.
///
/// # Panics
/// Should never panic. It will panic if `application/gzip` stops being a valid `CONTENT_TYPE`
/// header value and if `attachment; filename="..."` stops being a valid `CONTENT_DISPOSITION` value.
/// It will also panic if the filename is somehow bad and not valid inside the header
#[tracing::instrument(skip_all, name = "/v0/puzzle/files", fields(method = "GET"))]
pub async fn files(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, .. }: Authenticated,
    _: Discriminate,
    Id(id): Id,
) -> Result<impl IntoResponse, ErrorResponse> {
    let team = Team::id(sub, state.get_connection().await?).await?;
    Attempt::begin(state.get_connection().await?, id, team).await?;

    // Check if a generation task exists. This prevents more than one instance of `Puzzle::generate`
    // being ran at any instance.
    //
    // There's an issue here. Finished tasks can accumulate on the hashmap if a client asks for
    // something to be generated but never asks for the results. There should be come sort of
    // garbage collector for these.
    let mut tasks = state.tasks.lock().await;
    if let Some(handle) = tasks.get(&(id, team)) {
        if handle.is_finished() {
            let Some(handle) = tasks.remove(&(id, team)) else {
                return Err(ErrorResponse::internal("Mutex got poisoned".into()));
            };

            match handle.await {
                Ok(Ok(())) => (),
                Ok(Err(err)) => Err(err)?,
                _ => return Err(ErrorResponse::internal("Generator crashed".into())),
            }
        } else {
            return Err(ErrorResponse::busy());
        }
    }

    let Some(path) = Puzzle::archive(id, team)? else {
        let handle = tokio::task::spawn_blocking(move || Puzzle::generate(id, team));
        tasks.insert((id, team), handle);

        return Err(ErrorResponse::busy());
    };

    drop(tasks);

    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(err) => {
            return Err(ErrorResponse::internal(format!(
                "Could not open file: {err}"
            )));
        }
    };

    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(ErrorResponse::internal(
            "Could not get filename for generator result".into(),
        ));
    };

    let Ok(content_header) = format!("attachment; filename=\"{file_name}\"").parse() else {
        return Err(ErrorResponse::internal(
            "Could no parse CONTENT_DISPOSITION header".into(),
        ));
    };

    let body = Body::from_stream(ReaderStream::new(file));

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/gzip"
            .parse()
            .expect("application/gzip is no longer a valid mimetype."),
    );
    headers.insert(header::CONTENT_DISPOSITION, content_header);

    Ok((headers, body))
}

/// Register and attempt to solve and check the flag
///
/// # Errors
/// Will return an error if the puzzle doesn't exist, if the user is unauthenticated or if they're
/// not on an authorized network.
/// Might return an error if there's an issue communicating with the database or if the check fails
/// to run.
#[tracing::instrument(skip_all, name = "/v0/puzzle/solve", fields(method = "POST"))]
pub async fn solve(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, .. }: Authenticated,
    Query(query): Query<FlagCheckQuery>,
    _: Discriminate,
) -> Result<(), ErrorResponse> {
    let team = Team::id(sub, state.get_connection().await?).await?;

    Ok(Puzzle::solve(query.id, team, query.flag, state.get_connection().await?).await?)
}

/// Register and attempt to solve and check the flag
///
/// # Errors
/// Will return an error if the puzzle doesn't exist, if the user is unauthenticated or if they're
/// not on an authorized network.
/// Might return an error if there's an issue communicating with the database or if the check fails
/// to run.
#[tracing::instrument(skip_all, name = "/v0/puzzle/clue/next", fields(method = "POST"))]
pub async fn next_clue(
    State(state): State<Arc<Bstate>>,
    Authenticated { sub, .. }: Authenticated,
    _: Discriminate,
    Id(id): Id,
) -> Result<Json<usize>, ErrorResponse> {
    let team = Team::id(sub, state.get_connection().await?).await?;

    Ok(Json(
        Attempt::next_clue(id, team, state.get_connection().await?).await?,
    ))
}
