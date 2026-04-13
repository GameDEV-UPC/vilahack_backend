use std::io::Write;

use chrono::{DateTime, Utc};
use deadpool_diesel::postgres::Connection;
use uuid::Uuid;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    pg::{Pg, PgValue},
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Jsonb,
};

use exn::ResultExt;

use crate::{
    database::schema::{self},
    error::Error,
    model::attempt::Attempt,
};

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
#[ExistingTypePath = "crate::database::schema::sql_types::Difficulty"]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    VeryHard,
    Hard,
    Moderate,
    Easy,
    VeryEasy,
}

#[derive(
    Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Cryptography,
    ReverseEngineering,
    BinaryExploitation,
    NetworkSecurity,
    WebSecurity,
    Forensics,
    Steganography,
    OSINT,
    Miscellaneous,
}

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = Jsonb)]
pub struct Categories(pub Vec<Category>);

impl FromSql<Jsonb, Pg> for Categories {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        let bytes = bytes.as_bytes();

        if bytes[0] != 1 {
            return Err("Unsupported JSONB encoding version".into());
        }

        serde_json::from_slice(&bytes[1..]).map_err(Into::into)
    }
}

impl ToSql<Jsonb, Pg> for Categories {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        out.write_all(&[1])?;
        serde_json::to_writer(out, self)
            .map(|()| IsNull::No)
            .map_err(Into::into)
    }
}

#[derive(
    Debug,
    Clone,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    FromSqlRow,
    AsExpression,
)]
#[diesel(sql_type = Jsonb)]
pub struct Clues(pub Vec<String>);

impl FromSql<Jsonb, Pg> for Clues {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        let bytes = bytes.as_bytes();

        if bytes[0] != 1 {
            return Err("Unsupported JSONB encoding version".into());
        }

        serde_json::from_slice(&bytes[1..]).map_err(Into::into)
    }
}

impl ToSql<Jsonb, Pg> for Clues {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        out.write_all(&[1])?;
        serde_json::to_writer(out, self)
            .map(|()| IsNull::No)
            .map_err(Into::into)
    }
}

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
#[diesel(primary_key(id))]
#[diesel(table_name = schema::puzzle)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Puzzle {
    pub id: Uuid,
    pub difficulty: Difficulty,
    pub categories: Categories,
    pub points: i16,
    pub name: String,
    pub prompt: String,
    pub clues: Option<Clues>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub ommit: bool,
}

impl Puzzle {
    // TODO: Create

    /// Get the puzzle with the given id
    ///
    /// # Errors
    /// Will return an error if the puzzle doesn't exist.
    /// May return an error if there's an issue communicating with the database.
    pub async fn get(id: Uuid, connection: Connection) -> exn::Result<Self, Error> {
        use schema::puzzle::dsl::puzzle;

        connection
            .interact(move |connection| puzzle.find(id).select(Self::as_select()).first(connection))
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to fetch the puzzle".into()))
    }

    /// Get all the puzzles
    ///
    /// # Errors
    /// May return an error if there's an issue communicating with the database.
    pub async fn get_all(connection: Connection) -> exn::Result<Vec<Self>, Error> {
        use schema::puzzle::dsl::{ommit, puzzle};

        connection
            .interact(move |connection| {
                puzzle
                    .select(Self::as_select())
                    .filter(ommit.eq(false))
                    .get_results(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to fetch the puzzle".into()))
    }

    /// Hides clues that haven't been used by the requesting user's team for the puzzle
    ///
    /// # Errors
    /// May return an error if there's an issue communicating with the database.
    pub async fn hide_unused_clues(
        &mut self,
        user: Uuid,
        connection: Connection,
    ) -> exn::Result<(), Error> {
        let used_count = Attempt::clues_used(user, self.id, connection).await?;

        if let Some(ref mut clues) = self.clues {
            #[allow(clippy::cast_sign_loss)]
            for clue in clues.0.iter_mut().skip(used_count as usize) {
                *clue = String::new();
            }
        }

        Ok(())
    }

    // TODO solve
    // TODO files
}
