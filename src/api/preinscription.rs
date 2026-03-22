use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde_email::Email;

use crate::{database::Pool, error::ErrorResponse, model::preinscription::Preinscription};

/// Add the email to the preinscriptions table in the database
///
/// # Errors
/// Will return an error if the email is malformed, if it's not unique, if there's an issue
/// communicating with the database, or if the authentication fails.
#[tracing::instrument(err(Debug, level = tracing::Level::INFO), skip_all, name = "/v0/preinscribe", fields(method = "PUT"))]
pub async fn preinscribe(
    State(pool): State<Arc<Pool>>,
    Json(email): Json<Email>,
) -> Result<StatusCode, ErrorResponse> {
    Preinscription::new(&email)
        .preinscribe(pool.get().await?)
        .await?;

    Ok(StatusCode::OK)
}
