use chrono::{DateTime, Utc};
use deadpool_diesel::postgres::Connection;
use uuid::Uuid;

use diesel::{insert_into, prelude::*};

use exn::ResultExt;

use crate::{
    api::ParticipationFilter,
    database::schema::{self},
    error::Error,
};

#[derive(
    Queryable,
    Identifiable,
    Selectable,
    Insertable,
    Default,
    Debug,
    Clone,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = schema::event)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Event {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub location: String,
    pub begins_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(skip_deserializing, skip_serializing)]
    pub ommit: bool,
}

impl Event {
    /// Get the event with the given id
    ///
    /// # Errors
    /// Will return an error if an event with the given id doesn't exist
    /// Might return an error if there's an issue communicating with the database
    pub async fn get(id: Uuid, connection: Connection) -> exn::Result<Self, Error> {
        use crate::database::schema::event::dsl::event;

        connection
            .interact(move |connection| event.find(id).get_result(connection))
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to get event".into()))
    }

    /// Get all the events
    ///
    /// # Errors
    /// Might return an error if there's an issue communicating with the database
    pub async fn all(connection: Connection) -> exn::Result<Vec<Self>, Error> {
        use crate::database::schema::event::dsl::{begins_at, event};

        connection
            .interact(move |connection| {
                event
                    .select(Self::as_select())
                    .order(begins_at.asc())
                    .get_results(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to get all events".into()))
    }
}

#[derive(
    Queryable,
    Identifiable,
    Selectable,
    Insertable,
    Default,
    Debug,
    Clone,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
#[diesel(primary_key(user, event))]
#[diesel(table_name = schema::participate)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Participate {
    pub user: Uuid,
    pub event: Uuid,
    pub created_at: DateTime<Utc>,
}

impl Participate {
    /// Record the participation
    ///
    /// # Errors
    /// Will return an error if the user hasn't made an application, or if they've already participated
    /// on the event.
    /// Might return an error if there's an issue communicating with the database
    pub async fn post(self, connection: Connection) -> exn::Result<usize, Error> {
        use crate::database::schema::participate::dsl::participate;

        connection
            .interact(move |connection| insert_into(participate).values(self).execute(connection))
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to update the application".into()))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Participation {
    pub event: Uuid,
    pub name: String,
    pub began_at: DateTime<Utc>,
    pub participated_at: DateTime<Utc>,
}

impl Participation {
    /// Get all the user's participations
    ///
    /// # Errors
    /// Might return an error if there's an issue communicating with the database
    pub async fn get(
        filter: ParticipationFilter,
        connection: Connection,
    ) -> exn::Result<Vec<Self>, Error> {
        use crate::database::schema::{
            event::dsl::{begins_at, event as event_dsl, id as event_id, name as event_name},
            participate::dsl::{created_at, event as p_event, participate, user},
        };

        let mut query = participate
            .inner_join(event_dsl)
            .select((event_id, event_name, begins_at, created_at))
            .into_boxed::<diesel::pg::Pg>();

        match filter {
            ParticipationFilter::User(id) => {
                query = query.filter(user.eq(id));
            }
            ParticipationFilter::Event(id) => {
                query = query.filter(p_event.eq(id));
            }
            ParticipationFilter::None => {}
        }

        let participations = connection
            .interact(move |connection| query.get_results(connection))
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to update the application".into()))?;

        Ok(participations
            .into_iter()
            .map(|(event, name, began_at, participated_at)| Self {
                event,
                name,
                began_at,
                participated_at,
            })
            .collect())
    }
}
