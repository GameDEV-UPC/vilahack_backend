use deadpool_diesel::postgres::Connection;
use diesel::{insert_into, prelude::*};

use crate::{
    database::schema,
    error::{Error, FlattenErr},
};

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
#[diesel(table_name = schema::preinscription)]
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
    /// Will return an error if the email is not unique
    /// Might return an error if there's an issue communicating with the database
    pub async fn preinscribe(self, connection: Connection) -> exn::Result<usize, Error> {
        use schema::preinscription::dsl::preinscription;

        connection
            .interact(move |connection| {
                insert_into(preinscription).values(self).execute(connection)
            })
            .await
            .flatten_err()
    }
}
