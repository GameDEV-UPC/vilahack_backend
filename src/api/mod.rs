pub mod event;
pub mod preinscription;
pub mod puzzle;
pub mod team;
pub mod user;

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
    pub user: Option<String>,
    pub event: Option<Uuid>,
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

        let Some(ref user) = query.user else {
            return Err((StatusCode::BAD_REQUEST, "user query is missing"));
        };

        let Some(event) = query.event else {
            return Err((StatusCode::BAD_REQUEST, "event query is missing"));
        };

        let user = if let Ok(id) = Uuid::try_parse(user) {
            id
        } else {
            let mut decoded: [u8; 16] = [0; 16];
            if BASE64_STANDARD_NO_PAD
                .decode_slice(user, &mut decoded)
                .is_err()
            {
                return Err((StatusCode::BAD_REQUEST, "id could not be decoded"));
            }

            uuid::Uuid::from_bytes(decoded)
        };

        Ok(Self {
            user,
            event,
            created_at: Utc::now(),
        })
    }
}

#[derive(PartialEq, Eq)]
pub enum ParticipationFilter {
    User(Uuid),
    Event(Uuid),
    None,
}

impl<S> FromRequestParts<S> for ParticipationFilter
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Ok(query) = Query::<ParticipateQuery>::from_request_parts(parts, state).await else {
            return Err((StatusCode::BAD_REQUEST, "Queries missing"));
        };

        if let Some(ref user) = query.user {
            if let Ok(id) = Uuid::try_parse(user) {
                return Ok(Self::User(id));
            }

            let mut decoded: [u8; 16] = [0; 16];
            if BASE64_STANDARD_NO_PAD
                .decode_slice(user, &mut decoded)
                .is_err()
            {
                return Err((StatusCode::BAD_REQUEST, "id could not be decoded"));
            }

            return Ok(Self::User(uuid::Uuid::from_bytes(decoded)));
        }

        if let Some(event) = query.event {
            return Ok(Self::Event(event));
        }

        Ok(Self::None)
    }
}
