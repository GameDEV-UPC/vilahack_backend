use std::io::Write;

use chrono::{DateTime, Utc};
use deadpool_diesel::postgres::Connection;
use uuid::Uuid;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    expression::AsExpression,
    insert_into,
    pg::{Pg, PgValue},
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Jsonb,
};

use exn::ResultExt;

use crate::{
    database::schema::{self},
    error::Error,
};

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
pub struct Flags(Vec<String>);

impl FromSql<Jsonb, Pg> for Flags {
    fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
        let bytes = bytes.as_bytes();

        if bytes[0] != 1 {
            return Err("Unsupported JSONB encoding version".into());
        }

        serde_json::from_slice(&bytes[1..]).map_err(Into::into)
    }
}

impl ToSql<Jsonb, Pg> for Flags {
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
    Default,
    Debug,
    Clone,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
#[diesel(primary_key(team, puzzle))]
#[diesel(table_name = schema::attempt)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Attempt {
    pub team: Uuid,
    pub puzzle: Uuid,
    pub created_at: DateTime<Utc>,
    pub solved_at: Option<DateTime<Utc>>,
    pub clues_used: i16,
    pub flags: Option<Flags>,
}

impl Attempt {
    /// Get the number of clues a team has used for a certain puzzle.
    ///
    /// # Errors
    /// Will return an error if the attempt doesn't exist.
    /// May return an error if there's an issue communicating with the database.
    pub async fn clues_used(
        user: Uuid,
        puzzle: Uuid,
        connection: Connection,
    ) -> exn::Result<i16, Error> {
        use schema::attempt::dsl::{
            attempt, clues_used, puzzle as attempt_puzzle, team as attempt_team,
        };
        use schema::member_of::dsl::{member_of, team as member_team, user as member_user};

        connection
            .interact(move |connection| {
                attempt
                    .inner_join(member_of.on(attempt_team.eq(member_team)))
                    .filter(member_user.eq(user))
                    .filter(attempt_puzzle.eq(puzzle))
                    .select(clues_used)
                    .first::<i16>(connection)
                    .optional()
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to fetch the puzzle".into()))
            .map(|opt| opt.unwrap_or(0))
    }

    /// Begins a puzzle attempt if it hasn't already been done
    ///
    /// # Errors
    /// Will return an error if the user does not have an application.
    /// May return an error if there's an issue communicating with the database
    pub async fn begin(self, connection: Connection) -> exn::Result<usize, Error> {
        use schema::attempt::dsl::{attempt, puzzle, team};

        connection
            .interact(move |connection| {
                insert_into(attempt)
                    .values(self)
                    .on_conflict((team, puzzle))
                    .do_nothing()
                    .execute(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to begin an attempt".into()))
    }
}
