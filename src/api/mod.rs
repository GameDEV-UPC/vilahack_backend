pub mod preinscription;
pub mod puzzle;
pub mod team;
pub mod user;
pub mod event;

use axum::{
    extract::{FromRequestParts, Query},
    http::{StatusCode, request::Parts},
};
use base64::prelude::{BASE64_STANDARD_NO_PAD, Engine};
use chrono::Utc;
use uuid::Uuid;

use crate::model::event::Participate;

#[derive(serde::Deserialize, Debug)]
pub struct UidQuery {
    id: Option<String>,
}

pub struct Id(uuid::Uuid);
pub struct OptionalId(Option<uuid::Uuid>);

impl<S> FromRequestParts<S> for Id
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Ok(query) = Query::<UidQuery>::from_request_parts(parts, state).await else {
            return Err((StatusCode::BAD_REQUEST, "id query missing"));
        };

        let Some(ref query_id) = query.id else {
            return Err((StatusCode::BAD_REQUEST, "id query missing"));
        };

        Uuid::try_parse(query_id).map_or_else(
            |_| {
                let mut decoded: [u8; 16] = [0; 16];
                if BASE64_STANDARD_NO_PAD
                    .decode_slice(query_id, &mut decoded)
                    .is_err()
                {
                    Err((StatusCode::BAD_REQUEST, "id could not be decoded"))
                } else {
                    Ok(Self(uuid::Uuid::from_bytes(decoded)))
                }
            },
            |uuid| Ok(Self(uuid)),
        )
    }
}

impl<S> FromRequestParts<S> for OptionalId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Ok(query) = Query::<UidQuery>::from_request_parts(parts, state).await else {
            return Ok(Self(None));
        };

        let Some(ref query_id) = query.id else {
            return Ok(Self(None));
        };

        Uuid::try_parse(query_id).map_or_else(
            |_| {
                let mut decoded: [u8; 16] = [0; 16];
                if BASE64_STANDARD_NO_PAD
                    .decode_slice(query_id, &mut decoded)
                    .is_err()
                {
                    Err((StatusCode::BAD_REQUEST, "id could not be decoded"))
                } else {
                    Ok(Self(Some(uuid::Uuid::from_bytes(decoded))))
                }
            },
            |uuid| Ok(Self(Some(uuid))),
        )
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Name {
    name: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct FlagCheckQuery {
    pub id: Uuid,
    pub flag: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct ParticipateQuery {
    pub user: String,
    pub event: Uuid,
}

impl<S> FromRequestParts<S> for Participate
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Ok(query) = Query::<ParticipateQuery>::from_request_parts(parts, state).await else {
            return Err((StatusCode::BAD_REQUEST, "Queries missing"));
        };

        let user = if let Ok(id) = Uuid::try_parse(&query.user) {
            id
        } else {
            let mut decoded: [u8; 16] = [0; 16];
            if BASE64_STANDARD_NO_PAD
                .decode_slice(&query.user, &mut decoded)
                .is_err()
            {
                return Err((StatusCode::BAD_REQUEST, "id could not be decoded"));
            }

            uuid::Uuid::from_bytes(decoded)
        };

        Ok(Self {
            user,
            event: query.event,
            created_at: Utc::now(),
        })
    }
}
