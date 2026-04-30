use chrono::{DateTime, Utc};
use deadpool_diesel::postgres::Connection;
use uuid::Uuid;

use diesel::{insert_into, prelude::*};

use exn::ResultExt;

use crate::{
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
