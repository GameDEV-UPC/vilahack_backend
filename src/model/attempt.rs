use std::{collections::HashMap, io::Write};

use chrono::{DateTime, Utc};
use deadpool_diesel::postgres::Connection;
use uuid::Uuid;

use diesel::{
    deserialize::{FromSql, FromSqlRow},
    dsl::{jsonb_array_length, sql},
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ScoreboardEntry {
    name: String,
    score: i64,
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
    pub async fn begin(
        connection: Connection,
        puzzle: Uuid,
        team: Uuid,
    ) -> exn::Result<usize, Error> {
        use schema::attempt::dsl::{attempt, puzzle as puzzle_dsl, team as team_dsl};

        let att = Self {
            puzzle,
            team,
            created_at: Utc::now(),
            ..Default::default()
        };

        connection
            .interact(move |connection| {
                insert_into(attempt)
                    .values(att)
                    .on_conflict((team_dsl, puzzle_dsl))
                    .do_nothing()
                    .execute(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to begin an attempt".into()))
    }

    /// Appends an attempted flag to an attempt. Marks the attempt as solved if the flag is correct.
    ///
    /// Will create an attempt if one does't already exist
    ///
    /// # Errors
    /// Will return an error if the user does not have an application.
    /// May return an error if there's an issue communicating with the database
    pub async fn append(
        connection: Connection,
        puzzle: Uuid,
        team: Uuid,
        flag: String,
        correct: bool,
    ) -> exn::Result<usize, Error> {
        use schema::attempt::dsl;

        let solved_at = if correct { Some(Utc::now()) } else { None };

        let att = Self {
            puzzle,
            team,
            created_at: Utc::now(),
            solved_at,
            flags: Some(Flags(vec![flag])),
            ..Default::default()
        };

        connection
            .interact(move |connection| {
                insert_into(dsl::attempt)
                    .values(att)
                    .on_conflict((dsl::team, dsl::puzzle))
                    .do_update()
                    .set((
                        dsl::flags.eq(sql::<diesel::sql_types::Nullable<Jsonb>>(
                            "COALESCE(attempt.flags, '[]'::jsonb) || EXCLUDED.flags",
                        )),
                        dsl::solved_at.eq(sql::<
                            diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>,
                        >(
                            "COALESCE(attempt.solved_at, EXCLUDED.solved_at)"
                        )),
                    ))
                    .execute(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to append to the attempt".into()))
    }

    /// Increments the `clues_used` counter by 1
    ///
    /// Will create an attempt if one does't already exist
    ///
    /// # Errors
    /// Will return an error if the user does not have an application.
    /// May return an error if there's an issue communicating with the database
    pub async fn next_clue(
        puzzle: Uuid,
        team: Uuid,
        connection: Connection,
    ) -> exn::Result<usize, Error> {
        use schema::attempt::dsl;

        let att = Self {
            puzzle,
            team,
            created_at: Utc::now(),
            clues_used: 1,
            ..Default::default()
        };

        connection
            .interact(move |connection| {
                insert_into(dsl::attempt)
                    .values(att)
                    .on_conflict((dsl::team, dsl::puzzle))
                    .do_update()
                    .set((dsl::clues_used
                        .eq(sql::<diesel::sql_types::SmallInt>("attempt.clues_used + 1")),))
                    .execute(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to append to the attempt".into()))
    }

    /// Calculate the scoreboard
    ///
    /// # Errors
    /// May return an error if there's an issue communicating with the database
    pub async fn scoreboard(
        filter: Option<Uuid>,
        connection: Connection,
    ) -> exn::Result<Vec<ScoreboardEntry>, Error> {
        let attempts: Vec<(Uuid, String, i16, Option<i32>, i16)> = connection
            .interact(move |connection| {
                use schema::attempt::dsl::{
                    attempt as attempt_dsl, clues_used, puzzle as attempt_puzzle, solved_at,
                    team as attempt_team,
                };
                use schema::puzzle::dsl::{clues, id as puzzle_id, points, puzzle};
                use schema::team::dsl::{id as team_id, name as team_name, team};

                let mut query = attempt_dsl
                    .filter(solved_at.is_not_null())
                    .inner_join(team.on(attempt_team.eq(team_id)))
                    .inner_join(puzzle.on(attempt_puzzle.eq(puzzle_id)))
                    .select((
                        attempt_team,
                        team_name,
                        points,
                        jsonb_array_length(clues),
                        clues_used,
                    ))
                    .into_boxed();

                if let Some(team_filter) = filter {
                    query = query.filter(attempt_team.eq(team_filter));
                }

                query.get_results(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to append to the attempt".into()))?;

        let mut scores: HashMap<Uuid, (String, Vec<f64>)> = HashMap::new();
        for (team_id, team_name, puzzle_points, clue_count, clues_used) in attempts {
            let clue_count = match clue_count {
                Some(0) => continue,
                Some(n) => f64::from(n),
                _ => continue,
            };

            let clues_used = f64::from(clues_used);
            let clues_used = if (clues_used - clue_count).abs() < 0.01 {
                clues_used + 1.0
            } else {
                clues_used
            };

            let puzzle_points = f64::from(puzzle_points);

            let substracted = puzzle_points * ((clues_used / (clue_count + 1.0)) * 0.4);

            let score = puzzle_points - substracted;

            if let Some((_, score_set)) = scores.get_mut(&team_id) {
                score_set.push(score);
            } else {
                scores.insert(team_id, (team_name, vec![score]));
            }
        }

        let mut scoreboard: Vec<ScoreboardEntry> = Vec::new();
        for (name, score_set) in scores.values() {
            scoreboard.push(ScoreboardEntry {
                name: name.clone(),
                #[allow(clippy::cast_possible_truncation)]
                score: score_set.iter().sum::<f64>().round() as i64,
            });
        }

        scoreboard.sort_by_key(|v| v.score);
        scoreboard.reverse();

        Ok(scoreboard)
    }
}
