use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};
use serde_email::Email;

use crate::{database::Pool, error::ErrorResponse, model::preinscriptions::Preinscription};

/// Add the email to the preinscriptions table in the database
///
/// # Errors
/// Will return an error if the email is malformed, if it's not unique or if there's an issue
/// communicating with the database
pub async fn preinscribe(
    State(pool): State<Arc<Pool>>,
    Json(email): Json<Email>,
) -> Result<StatusCode, ErrorResponse> {
    tracing::trace!("Endpoint /v0/preinscribe called");

    Preinscription::new(&email)
        .preinscribe(pool.get().await?)
        .await?;

    Ok(StatusCode::OK)
}
