pub mod preinscription;

use axum_extra::TypedHeader;
use headers::{Authorization, authorization::Bearer};

use crate::{authentication::authenticate, error::ErrorResponse};

pub async fn test(
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> Result<String, ErrorResponse> {
    tracing::trace!("test endpoint called");
    authenticate(bearer.token())?;

    Ok(String::from("tested"))
}
