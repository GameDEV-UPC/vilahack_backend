pub mod preinscription;
pub mod puzzle;
pub mod team;
pub mod user;

use axum::{
    extract::{FromRequestParts, Query},
    http::{StatusCode, request::Parts},
};
use base64::prelude::{BASE64_STANDARD_NO_PAD, Engine};

#[derive(serde::Deserialize, Debug)]
pub struct UidQuery {
    id: Option<uuid::Uuid>,
    qr: Option<String>,
}

pub struct Uid(uuid::Uuid);
pub struct OptionalUid(Option<uuid::Uuid>);

impl<S> FromRequestParts<S> for OptionalUid
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let query = Query::<UidQuery>::from_request_parts(parts, state)
            .await
            .unwrap();

        if let Some(id) = query.id {
            Ok(Self(Some(id)))
        } else if let Some(encoded) = &query.qr {
            let mut decoded: [u8; 16] = [0; 16];
            if BASE64_STANDARD_NO_PAD
                .decode_slice(encoded, &mut decoded)
                .is_err()
            {
                Err((StatusCode::BAD_REQUEST, "qr could not be decoded"))
            } else {
                Ok(Self(Some(uuid::Uuid::from_bytes(decoded))))
            }
        } else {
            Ok(Self(None))
        }
    }
}

impl<S> FromRequestParts<S> for Uid
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let query = Query::<UidQuery>::from_request_parts(parts, state)
            .await
            .unwrap();

        if let Some(id) = query.id {
            Ok(Self(id))
        } else if let Some(encoded) = &query.qr {
            let mut decoded: [u8; 16] = [0; 16];
            if BASE64_STANDARD_NO_PAD
                .decode_slice(encoded, &mut decoded)
                .is_err()
            {
                Err((StatusCode::BAD_REQUEST, "qr could not be decoded"))
            } else {
                Ok(Self(uuid::Uuid::from_bytes(decoded)))
            }
        } else {
            Err((
                StatusCode::BAD_REQUEST,
                "Either id or qr queries are required",
            ))
        }
    }
}
