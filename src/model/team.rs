use deadpool_diesel::postgres::Connection as PgConnection;
use diesel::{
    Connection, ExpressionMethods, RunQueryDsl, Selectable, insert_into,
    prelude::{Identifiable, Insertable, Queryable},
    result::Error as DieselError,
};

use uuid::Uuid;

use exn::ResultExt;

use crate::{database::schema, error::Error};

#[derive(Queryable, Insertable, Debug, Clone)]
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
    pub member_count: i16,
    pub score: i32,
}

impl Team {
    /// Create the team with the given name and join the creator to it.
    ///
    /// # Errors
    /// Will return an error if the user or team don't exist, or if the user is already in a team.
    /// May return an error if there's an issue communicating with the database.
    pub async fn new(
        name: String,
        creator: Uuid,
        connection: PgConnection,
    ) -> exn::Result<Team, Error> {
        use schema::{
            member_of::dsl::member_of,
            team::dsl::{name as team_name, team},
        };

        connection
            .interact(move |connection| {
                connection.transaction(|connection| {
                    let inserted_team: Team = insert_into(team)
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

                    Ok::<Team, DieselError>(inserted_team)
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
    /// May return an error if there's an issue communicating with the database.
    pub async fn join(
        user: Uuid,
        team: Uuid,
        connection: PgConnection,
    ) -> exn::Result<usize, Error> {
        use schema::member_of::dsl::member_of;

        connection
            .interact(move |connection| {
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
            .or_raise(|| Error::upstream("Failed to insert the user".into()))
    }

    /// Leave a team
    ///
    /// # Errors
    /// Will return an error if the user doesn't exists or if the user does not belong to the team.
    /// May return an error if there's an issue communicating with the database.
    pub async fn leave(
        user: Uuid,
        team: Uuid,
        connection: PgConnection,
    ) -> exn::Result<usize, Error> {
        connection
            .interact(move |connection| todo!())
            .await
            .map_err(Error::from) // Això és una mica lleig però bueno
            .or_raise(|| Error::upstream("Failed to interact with connection pool".into()))?
            .map_err(Error::from)
            .or_raise(|| Error::upstream("Failed to insert the user".into()))
    }
}
