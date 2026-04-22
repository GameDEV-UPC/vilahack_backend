use std::{io::Write, path::PathBuf, process::Command};

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
    config::CONFIG,
    database::schema::{self},
    error::{Error, PuzzleError},
    model::attempt::Attempt,
};

fn find_output(directory: PathBuf) -> exn::Result<PathBuf, Error> {
    let directory_contents = match std::fs::read_dir(directory) {
        Ok(contents) => Ok(contents),
        Err(err) => Err(Error::puzzle(PuzzleError::Io, err.to_string())),
    };

    for element in directory_contents?.filter_map(|e| e.ok()) {
        if element.path().is_file()
            && let Some(extension) = element.path().extension()
            && extension == "gz"
        {
            return Ok(element.path());
        }
    }

    Err(exn::Exn::new(Error::puzzle(
        PuzzleError::FilesMissing,
        "Could not find an output .tar.gz".into(),
    )))
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

    fn generate(team: Uuid, path: PathBuf) -> exn::Result<(), Error> {
        let Ok(status) = Command::new("nix develop --command bash generate.sh")
            .arg(team.to_string())
            .current_dir(path)
            .status()
        else {
            return Err(exn::Exn::new(Error::puzzle(
                PuzzleError::Generator,
                "Could not run generator".into(),
            )));
        };

        if !status.success() {
            return Err(exn::Exn::new(Error::puzzle(
                PuzzleError::Generator,
                "Generator exited with an error code".into(),
            )));
        }

        Ok(())
    }

    /// Returns the path to the puzzle's generated archive for the team.
    ///
    /// It is important for only one instance of this function for any (`team`, `puzzle`) pair to
    /// run in any instant. This is because if it were to happen, the same generator could run
    /// several times in parallel, causing issues. It could also happen that this function thinks a
    /// generator is done because it found the output directory but the generator has not yet found
    /// the output archive.
    ///
    /// Maybe this could be made robust with some kind of lockfile mechanic. But it would require
    /// considering.
    ///
    /// # Errors
    /// Returns an error if the puzzle doesn't exist, if there's an issue reading the disk, if
    /// there's an issue running the generator or if the generator exits with an error code.
    pub async fn files(puzzle: Uuid, team: Uuid) -> exn::Result<PathBuf, Error> {
        let puzzle_path = {
            let mut path = CONFIG.puzzle_directory.clone();
            path.push(puzzle.to_string());
            path
        };

        let output_path = {
            let mut path = puzzle_path.clone();
            path.push("out");
            path.push(team.to_string());
            path
        };
        
        if !output_path.is_dir() {
            Self::generate(team, puzzle_path)?;
        }

        find_output(output_path)
    }

    // TODO solve
}
