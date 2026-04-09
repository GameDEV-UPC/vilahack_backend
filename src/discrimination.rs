use std::{env, net::IpAddr, sync::LazyLock};

use axum::{extract::FromRequestParts, http::request::Parts};
use ipnet::IpNet;

use crate::error::{AuthenticationError as Ae, Error, ErrorResponse};

static ALLOWED_RANGES: LazyLock<Vec<IpNet>> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    _ = &env::var("ALLOWED_RANGES").expect("Missing ALLOWED_RANGES env variable");

    env::var("ALLOWED_RANGES")
        .expect("Missing `ALLOWED_RANGES` env variable")
        .split(' ')
        .map(|range| range.parse().expect("Failed to parse `ALLOWED_RANGES`"))
        .collect()
});

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
                    if ALLOWED_RANGES.iter().any(|range| range.contains(&ip)) {
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
