use deadpool_diesel::postgres::Connection;
use diesel::{insert_into, prelude::*};

use exn::ResultExt;

use crate::{database::schema, error::Error};

#[derive(
    Queryable,
    Identifiable,
    Selectable,
    Insertable,
    Debug,
    Clone,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
#[diesel(primary_key(email))]
#[diesel(table_name = schema::preinscriptions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Preinscription {
    email: String,
}

impl Preinscription {
    #[must_use]
    pub fn new(email: &serde_email::Email) -> Self {
        Self {
            email: email.to_string(),
        }
    }

    /// Insert a new email into the preinscriptions table
    ///
    /// # Errors
    /// May return any of the Database errors
    pub async fn preinscribe(self, connection: Connection) -> exn::Result<usize, Error> {
        use schema::preinscriptions::dsl::preinscriptions;

        connection
            .interact(move |connection| {
                insert_into(preinscriptions)
                    .values(self)
                    .execute(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to insert the preinscription".into()))
    }
}
