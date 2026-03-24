use deadpool_diesel::postgres::Connection as PgConnection;
use diesel::{
    Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, Selectable,
    dsl::count,
    insert_into,
    prelude::{Identifiable, Insertable, Queryable},
    result::Error as DieselError,
    update,
};

use uuid::Uuid;

use exn::ResultExt;

use crate::{database::schema, error::Error};

#[derive(Queryable, Insertable, Selectable, Debug, Clone)]
#[diesel(primary_key(user))]
#[diesel(table_name = schema::member_of)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MemberOf {
    pub user: Uuid,
    pub team: Uuid,
}

#[derive(Queryable, Identifiable, Selectable, Insertable, Debug, Clone)]
#[diesel(primary_key(id))]
#[diesel(table_name = schema::team)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub score: i32,
}

#[derive(serde::Serialize)]
pub struct TeamSummary {
    name: String,
    id: String,
    members: Vec<String>,
}

impl Team {
    /// Create the team with the given name and join the creator to it.
    ///
    /// # Errors
    /// Will return an error if the user is already in a team.
    /// May return an error if there's an issue communicating with the database.
    pub async fn new(
        name: String,
        creator: Uuid,
        connection: PgConnection,
    ) -> exn::Result<Self, Error> {
        use schema::{
            member_of::dsl::member_of,
            team::dsl::{name as team_name, team},
        };

        connection
            .interact(move |connection| {
                connection.transaction(|connection| {
                    let inserted_team: Self = insert_into(team)
                        .values(team_name.eq(name))
                        .get_result(connection)?;

                    // Referential integrity will serve as a check of "do this user and team
                    // exist" and the primary key constraint will serve as a check of "is this
                    // user already in a team"
                    //
                    // It's true that checking this on the backend is possible and would _maybe_
                    // save some resources but using the database for this greatly simplifies this
                    // code
                    insert_into(member_of)
                        .values(MemberOf {
                            user: creator,
                            team: inserted_team.id,
                        })
                        .execute(connection)?;

                    Ok::<Self, DieselError>(inserted_team)
                })
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to insert the user".into()))
    }

    /// Join an existing team
    ///
    /// # Errors
    /// Will return an error if the team or user don't exist, or if the user is already in a team.
    /// It will also return an error if the group already has 4 members.
    /// May return an error if there's an issue communicating with the database.
    pub async fn join(user: Uuid, team: Uuid, connection: PgConnection) -> exn::Result<(), Error> {
        use schema::member_of::dsl::{member_of, team as team_dsl, user as user_dsl};

        match connection
            .interact(move |connection| {
                let count: i64 = member_of
                    .filter(team_dsl.eq(team))
                    .select(count(user_dsl))
                    .first(connection)?;

                if count >= 4 {
                    return Err(DieselError::NotFound);
                }

                // Relies on the primary key constraint to ensure that the user isn't already on
                // another group, as well as on referential integrity to make sure both the user
                // and team exist.
                insert_into(member_of)
                    .values(MemberOf { user, team })
                    .execute(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to insert the user".into()))?
        {
            0 => Err(exn::Exn::new(Error::database(
                crate::error::DatabaseError::ConstraintViolation,
                "The user is already in a team".into(),
            ))),
            1 => Ok(()),
            n => {
                tracing::warn!(
                    "{n} rows were updated when trying to join user {user} to team {team}"
                );

                Err(exn::Exn::new(Error::database(
                    crate::error::DatabaseError::Unknown,
                    "Something unexpected happened when trying to join the team".into(),
                )))
            }
        }
    }

    /// Leave whatever team the user is joined to
    ///
    /// # Errors
    /// Will return an error if the user doesn't exists or if the user does not belong to the team.
    /// May return an error if there's an issue communicating with the database.
    pub async fn leave(user: Uuid, connection: PgConnection) -> exn::Result<(), Error> {
        use schema::member_of::dsl::member_of;

        match connection
            .interact(move |connection| diesel::delete(member_of.find(user)).execute(connection))
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to insert the user".into()))?
        {
            0 => Err(exn::Exn::new(Error::database(
                crate::error::DatabaseError::NotFound,
                "The user is not in a team".into(),
            ))),
            1 => Ok(()),
            n => {
                tracing::warn!(
                    "{n} rows were updated when trying to leave {user} from their group"
                );

                Err(exn::Exn::new(Error::database(
                    crate::error::DatabaseError::Unknown,
                    "Something unexpected happened while trying to leave".into(),
                )))
            }
        }
    }

    /// Update the name of the team the user currently belongs to
    ///
    /// # Errors
    /// Will return an error if the user doesn't belong to any team.
    /// May return an error if there's an issue communicating with the database.
    pub async fn update(
        user: Uuid,
        name: String,
        connection: PgConnection,
    ) -> exn::Result<(), Error> {
        use schema::member_of::dsl::{member_of, team as team_id};
        use schema::team::dsl::{id, name as name_dsl, team};

        match connection
            .interact(move |connection| {
                let team_fk: Uuid = member_of.find(user).select(team_id).first(connection)?;

                update(team.filter(id.eq(team_fk)))
                    .set(name_dsl.eq(name))
                    .execute(connection)
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to insert the user".into()))?
        {
            0 => Err(exn::Exn::new(Error::database(
                crate::error::DatabaseError::Unknown,
                "The team's name could not be updated".into(),
            ))),
            1 => Ok(()),
            n => {
                tracing::warn!("{n} rows were updated when trying to update a group name");

                Err(exn::Exn::new(Error::database(
                    crate::error::DatabaseError::Unknown,
                    "Something unexpected happened when trying to update the team name".into(),
                )))
            }
        }
    }

    /// Get a summary of the team
    ///
    /// # Errors
    /// Will return an error if the user is not in any team
    /// May return an error if there's an issue communicating with the database.
    pub async fn summary(user: Uuid, connection: PgConnection) -> exn::Result<TeamSummary, Error> {
        use schema::member_of::dsl::{member_of, team as team_id, user as team_member};
        use schema::team::dsl::{name, team};
        use schema::user::dsl::{id as user_id, name as user_name, user as user_dsl};

        use base64::prelude::{BASE64_STANDARD_NO_PAD, Engine};

        connection
            .interact(move |connection| {
                let team_fk: Uuid = member_of.find(user).select(team_id).first(connection)?;
                let team_name = team.find(team_fk).select(name).first(connection)?;

                let member_names: Vec<String> = member_of
                    .inner_join(user_dsl.on(team_member.eq(user_id)))
                    .filter(team_id.eq(team_fk))
                    .select(user_name)
                    .load::<String>(connection)?;

                Ok::<TeamSummary, DieselError>(TeamSummary {
                    name: team_name,
                    id: BASE64_STANDARD_NO_PAD.encode(team_fk.as_bytes()),
                    members: member_names,
                })
            })
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to get team summary".into()))
    }
}
