#![allow(clippy::struct_field_names)]

use std::io::Write;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use diesel::{
    pg::Pg,
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
};

use crate::database::schema;
use crate::database::schema::sql_types::AccessibilityType;

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
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
    Aware,
    Newbie,
}

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
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
#[serde(rename_all = "snake_case")]
pub enum TshirSize {
    XS,
    S,
    M,
    L,
    XL,
    XXL,
}

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
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
    Unspecified,
}

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
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
    SocialMedia,
    Website,
    Acquaintances,
    Posters,
    Other,
}

#[derive(
    diesel_derive_enum::DbEnum,
    Debug,
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
    Applied,
    Accepted,
    Disqualified,
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

#[derive(
    Queryable,
    Identifiable,
    Selectable,
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
    pub created_at: DateTime<Utc>,
    pub check_in: Option<DateTime<Utc>>,
    pub comment: String,
    pub is_at_end: bool,
    pub qr_code: String,
    pub status: Status,
}
