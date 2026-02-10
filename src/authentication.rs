use std::{env, sync::LazyLock};

use jsonwebtoken::{DecodingKey, Validation, decode, decode_header, jwk::JwkSet};

use crate::error::{AuthenticationError as Ae, Error};
use exn::{ResultExt, bail};

static JWKSET: LazyLock<JwkSet> = LazyLock::new(|| {
    dotenvy::dotenv().ok();
    serde_json::from_str(&env::var("JWKS").expect("Missing JWKS env variable"))
        .expect("Failed to deserialze JWK set from JWKS env variable")
});

const AUTHENTICATED_AUDIENCE: [&str; 1] = ["authenticated"];
const ISSUER: [&str; 1] = ["https://tfhmghtvgexflhdwcrer.supabase.co/auth/v1"];

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub role: String,
    // Ignored:
    // aal, amr, app_metadata, aud, email, exp, iat, is_anonymous, iss, phone, sission_id,
    // user_metadata
}

/// TODO
///
/// # Errors
/// TODO
pub fn authenticate(token: &str) -> exn::Result<uuid::Uuid, Error> {
    let jwkset: &JwkSet = &JWKSET;
    let header = decode_header(token)
        .map_err(Error::from)
        .or_raise(|| Error::upstream("Could not decode token".into()))?;

    let Some(kid) = header.kid else {
        bail!(Error::authentication(
            Ae::InvalidClaim,
            "JWT does not contain a kid claim".into(),
        ));
    };

    let Some(jwk) = jwkset.find(&kid) else {
        bail!(Error::authentication(
            Ae::NoMatchingKey,
            "No matching jwk found for the kiven kid. Your JWT might be outdated or the issuer might have outdated keys.".into(),
        ));
    };

    let validation = {
        let mut validation = Validation::new(header.alg);
        validation.set_audience(&AUTHENTICATED_AUDIENCE);
        validation.set_issuer(&ISSUER);
        validation
    };

    let key = &DecodingKey::try_from(jwk)
        .map_err(Error::from)
        .or_raise(|| Error::upstream("Could not convert JWK into decoding key".into()))?;

    Ok(decode::<Claims>(token, key, &validation)
        .map_err(Error::from)
        .or_raise(|| Error::upstream("Authentication failed".into()))?
        .claims
        .sub)
}
