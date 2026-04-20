use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode};

use crate::{database::Pool, error::ErrorResponse, model::preinscription::Preinscription};

/// Add the email to the preinscriptions table in the database
///
/// # Errors
/// Will return an error if the email is not unique.
/// Might return an error if there's an issue communicating with the database.
#[tracing::instrument(skip_all, name = "/v0/preinscribe", fields(method = "PUT"))]
pub async fn preinscribe(
    State(pool): State<Arc<Pool>>,
    Json(email): Json<String>,
) -> Result<StatusCode, ErrorResponse> {
    Preinscription::new(email)
        .preinscribe(pool.get().await?)
        .await?;

    Ok(StatusCode::OK)
}
