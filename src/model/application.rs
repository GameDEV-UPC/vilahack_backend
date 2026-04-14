use std::io::Write;

use chrono::{DateTime, Utc};
use deadpool_diesel::postgres::Connection;
use uuid::Uuid;

use diesel::{
    insert_into,
    pg::Pg,
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
    update,
};

use exn::ResultExt;

use crate::{
    database::schema::{self, sql_types::AccessibilityType},
    error::Error,
};

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[ExistingTypePath = "crate::database::schema::sql_types::Experience"]
#[serde(rename_all = "snake_case")]
pub enum Experience {
    Expert,
    Experienced,
    Inexperienced,
    #[default]
    Aware,
    Newbie,
}

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[ExistingTypePath = "crate::database::schema::sql_types::TshirtSize"]
#[serde(rename_all = "lowercase")]
pub enum TshirSize {
    XS,
    S,
    #[default]
    M,
    L,
    XL,
    XXL,
}

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[ExistingTypePath = "crate::database::schema::sql_types::Gender"]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Man,
    Woman,
    Nonbinary,
    Agender,
    Other,
    #[default]
    Unspecified,
}

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[ExistingTypePath = "crate::database::schema::sql_types::Discovery"]
#[serde(rename_all = "snake_case")]
pub enum Discovery {
    #[default]
    SocialMedia,
    Website,
    Acquaintances,
    Posters,
    Other,
}

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[ExistingTypePath = "crate::database::schema::sql_types::Status"]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    Applied,
    Accepted,
    Confirmed,
    Participating,
    Disqualified,
    Finisher,
    Winner,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize, diesel::FromSqlRow, diesel::AsExpression)]
    #[diesel(sql_type = AccessibilityType)]
    pub struct AccessibilityNeeds: i16 {
        const MOBILITY = 0b0001;
        const CAPTIONING = 0b0010;
        const QUIET = 0b0100;
        const OTHER = 0b1000;
    }
}

impl ToSql<AccessibilityType, Pg> for AccessibilityNeeds {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        let value: i16 = self.bits();
        out.write_all(&value.to_le_bytes())?;
        Ok(IsNull::No)
    }
}

use diesel::deserialize::{self, FromSql};

impl FromSql<AccessibilityType, Pg> for AccessibilityNeeds {
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> deserialize::Result<Self> {
        let bytes = bytes.as_bytes();
        if bytes.len() != 2 {
            return Err(format!("Expected 2 bytes, found {}", bytes.len()).into());
        }

        let array: [u8; 2] = bytes
            .try_into()
            .map_err(|_| "Failed to convert accessibility bytes to array")?;
        Ok(Self::from_bits_truncate(i16::from_le_bytes(array)))
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(
    Queryable,
    Identifiable,
    Selectable,
    Insertable,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = schema::application)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Application {
    #[serde(skip_deserializing, skip_serializing)]
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub longitude: f32,
    pub latitude: f32,
    pub studies: String,
    pub university: String,
    pub gender: Gender,
    pub discovery: Discovery,
    pub experience: Experience,
    pub first_time: bool,
    pub why: String,
    pub tshirt_size: TshirSize,
    pub dietary_preference: Option<serde_json::Value>,
    pub accessibility_needs: AccessibilityNeeds,
    pub dvcs: Option<String>,
    pub linkedin: Option<String>,
    pub website: Option<String>,
    pub allows_cv_sharing: bool,
    pub allows_marketing: bool,
    pub comment: String,
    #[serde(skip_deserializing)]
    pub created_at: DateTime<Utc>,
    #[serde(skip_deserializing)]
    pub check_in: Option<DateTime<Utc>>,
    #[serde(skip_deserializing)]
    pub status: Status,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(
    Queryable,
    Identifiable,
    Selectable,
    Insertable,
    Debug,
    Clone,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = schema::application)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApplicationSummary {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub status: Status,
}

impl Application {
    /// Insert the the application for the given user
    ///
    /// # Errors
    /// Will return an error if the user doesn't exist or if they've already made an application.
    /// May return an error if there's an issue communicating with the database.
    pub async fn create(mut self, id: Uuid, connection: Connection) -> exn::Result<usize, Error> {
        use schema::application::dsl::application;

        self.id = id;
        self.created_at = Utc::now();

        connection
            .interact(move |connection| insert_into(application).values(self).execute(connection))
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to create the application".into()))
    }

    /// Get the application's details
    ///
    /// # Errors
    /// Will return an error if the user doesn't have an application.
    /// Might return an error if there's an issue communicating with the database.
    pub async fn get(id: Uuid, connection: Connection) -> exn::Result<Self, Error> {
        use schema::application::dsl::application;

        connection
            .interact(move |connection| {
                application
                    .find(id)
                    .select(Self::as_select())
                    .first(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to fetch the application".into()))
    }

    /// Get a summarized list of applications
    ///
    /// # Errors
    /// Might return an error if there's an issue communicating with the database.
    pub async fn index(connection: Connection) -> exn::Result<Vec<ApplicationSummary>, Error> {
        use schema::application::dsl::application;

        connection
            .interact(move |connection| {
                application
                    .select(ApplicationSummary::as_select())
                    .get_results(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to fetch the application".into()))
    }

    // Update handled by ApplicationUpdate
    // Delete handled by cascade of auth.user delete

    /// Check in the user now
    ///
    /// # Errors
    /// Will return an error if the given user isn't accepted, is already checked in or if they
    /// don't have an application in the first place.
    /// Might return an error if there's an issue communicating with the database
    pub async fn check_in(uid: Uuid, connection: Connection) -> exn::Result<(), Error> {
        use schema::application::dsl::{application, check_in, id, status};

        match connection
            .interact(move |connection| {
                update(
                    application
                        .filter(id.eq(uid))
                        .filter(check_in.is_null())
                        .filter(status.eq(Status::Confirmed)),
                )
                .set(check_in.eq(Some(Utc::now())))
                .execute(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to check in the user".into()))?
        {
            0 => Err(exn::Exn::new(Error::database(
                crate::error::DatabaseError::ConstraintViolation,
                "The user doesn't exist or it has already been checked in".into(),
            ))),
            1 => Ok(()),
            n => {
                tracing::warn!("{n} rows were updated when trying to check in a user.");

                Err(exn::Exn::new(Error::database(
                    crate::error::DatabaseError::Unknown,
                    format!(
                        "Something went horribly wrong when trying to check_in user {uid} at {}. Please contact an administrator as soon as possible",
                        Utc::now()
                    ),
                )))
            }
        }
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Queryable, AsChangeset, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[diesel(table_name = schema::application)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApplicationUpdate {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub longitude: Option<f32>,
    pub latitude: Option<f32>,
    pub studies: Option<String>,
    pub university: Option<String>,
    pub gender: Option<Gender>,
    pub discovery: Option<Discovery>,
    pub experience: Option<Experience>,
    pub first_time: Option<bool>,
    pub why: Option<String>,
    pub tshirt_size: Option<TshirSize>,
    pub dietary_preference: Option<serde_json::Value>,
    pub accessibility_needs: Option<AccessibilityNeeds>,
    pub dvcs: Option<String>,
    pub linkedin: Option<String>,
    pub website: Option<String>,
    pub allows_cv_sharing: Option<bool>,
    pub allows_marketing: Option<bool>,
    pub comment: Option<String>,
}

impl ApplicationUpdate {
    /// Updates the given value for the application
    ///
    /// # Errors
    /// Will return an error if the user hasn't made an application or if they're already accepted.
    /// Might return an error if there's an issue communicating with the database
    pub async fn update(self, uid: Uuid, connection: Connection) -> exn::Result<usize, Error> {
        use schema::application::dsl::{application, id, status};

        connection
            .interact(move |connection| {
                update(application.filter(id.eq(uid).and(status.eq(Status::Applied))))
                    .set(self)
                    .execute(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to update the application".into()))
    }
}
