#![allow(clippy::struct_field_names)]

use std::io::Write;

use uuid::Uuid;

use diesel::{
    pg::Pg,
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
};

use crate::database::schema;

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

#[derive(diesel::SqlType)]
#[diesel(postgres_type(name = "Int2"))]
pub struct AccessibilityType;

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
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: Uuid,
    pub guardian_email: Option<String>,
    pub name: String,
    pub pronouns: String,
    pub country: String,
    pub experience: Experience,
    pub motivation: String,
    pub tshirt_size: TshirSize,
    pub dietary_preferences: serde_json::Value,
    pub accessibility_needs: i16,
    pub linkedin: String,
    pub personal_page: String,
}
