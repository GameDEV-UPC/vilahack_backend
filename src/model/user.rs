use std::io::Write;

use chrono::{DateTime, Utc};
use deadpool_diesel::postgres::Connection;
use uuid::Uuid;

use diesel::{
    insert_into,
    pg::Pg,
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
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
#[diesel(table_name = schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    #[serde(skip_deserializing)]
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
    pub qr_code: Option<String>,
    #[serde(skip_deserializing)]
    pub status: Status,
}

impl User {
    /// Insert the user into the public.user table
    ///
    /// # Errors
    /// May return an error if there's an issue communicating with the database
    pub async fn create(mut self, id: Uuid, connection: Connection) -> exn::Result<usize, Error> {
        use schema::user::dsl::user;

        self.id = id;
        self.created_at = Utc::now();

        connection
            .interact(move |connection| insert_into(user).values(self).execute(connection))
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to insert the user".into()))
    }
}
