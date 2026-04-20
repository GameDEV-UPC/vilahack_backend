use std::net::IpAddr;

use axum::{extract::FromRequestParts, http::request::Parts};

use crate::{
    config::CONFIG,
    error::{AuthenticationError as Ae, Error, ErrorResponse},
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Discriminate;

impl<S> FromRequestParts<S> for Discriminate
where
    S: Send + Sync,
{
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(parts
            .headers
            .get("X-Forwarded-For")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<IpAddr>().ok())
            .map_or_else(
                || {
                    exn::bail!(Error::authentication(
                        Ae::NotInNetwork,
                        "Failed to determine the origin of the call".into(),
                    ))
                },
                |ip| {
                    if CONFIG
                        .allowed_ranges
                        .iter()
                        .any(|range| range.contains(&ip))
                    {
                        Ok(Self)
                    } else {
                        exn::bail!(Error::authentication(
                            Ae::NotInNetwork,
                            "Request made from outside the authorized networks".into(),
                        ))
                    }
                },
            )?)
    }
}
