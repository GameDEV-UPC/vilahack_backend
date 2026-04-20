use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode, decode_header};

use crate::{
    config::CONFIG,
    error::{AuthenticationError as Ae, Error, ErrorResponse},
};
use exn::{ResultExt, bail};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Authenticated {
    pub sub: uuid::Uuid,
    pub role: String,
    // Ignored:
    // aal, amr, app_metadata, aud, email, exp, iat, is_anonymous, iss, phone, sission_id,
    // user_metadata
}

impl Authenticated {
    /// Validates a JWT
    ///
    /// # Errors
    /// Returns an error if the token could not be parsed or validated
    pub fn from_token(token: &str) -> exn::Result<Self, Error> {
        let header = decode_header(token)
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Could not decode token".into()))?;

        let Some(kid) = header.kid else {
            bail!(Error::authentication(
                Ae::InvalidClaim,
                "JWT does not contain a kid claim".into(),
            ));
        };

        let Some(jwk) = CONFIG.jwk.set.find(&kid) else {
            bail!(Error::authentication(
                Ae::NoMatchingKey,
                "No matching jwk found for the kiven kid. Your JWT might be outdated or the issuer might have outdated keys.".into(),
            ));
        };

        let validation = {
            let mut validation = Validation::new(header.alg);
            validation.set_audience(&CONFIG.jwk.authenticated_audiences);
            validation.set_issuer(&CONFIG.jwk.issuers);
            validation
        };

        let key = &DecodingKey::try_from(jwk)
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Could not convert JWK into decoding key".into()))?;

        Ok(decode::<Self>(token, key, &validation)
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Authentication failed".into()))?
            .claims)
    }
}

impl<S> FromRequestParts<S> for Authenticated
where
    S: Send + Sync,
{
    type Rejection = ErrorResponse;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(Ok(token)) = parts.headers.get(AUTHORIZATION).map(|v| v.to_str()) {
            let Some(token) = token.get(7..) else {
                return Err(ErrorResponse::from(exn::Exn::new(Error::authentication(
                    Ae::Missing,
                    "Missing Authorization token".into(),
                ))));
            };

            let res = Self::from_token(token)
                .or_raise(|| Error::upstream("Failed to create Authenticated from token".into()));

            Ok(res?)
        } else {
            Err(ErrorResponse::from(exn::Exn::new(Error::authentication(
                Ae::Missing,
                "Missing Authorization header".into(),
            ))))
        }
    }
}
