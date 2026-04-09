pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "experience"))]
    pub struct Experience;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tshirt_size"))]
    pub struct TshirtSize;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "gender"))]
    pub struct Gender;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "discovery"))]
    pub struct Discovery;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "difficulty"))]
    pub struct Difficulty;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "status"))]
    pub struct Status;

    #[derive(diesel::SqlType)]
    #[diesel(postgres_type(name = "int2"))]
    pub struct AccessibilityType;
}

diesel::table! {
    use super::sql_types::{TshirtSize, Experience, Gender, Discovery, Status, AccessibilityType};
    use diesel::sql_types::{Uuid, Nullable, Text, Jsonb, Int2, Bool, Float, Timestamptz};

    application (id) {
        id -> Uuid,
        name -> Text,
        phone -> Text,
        longitude -> Float,
        latitude -> Float,
        studies -> Text,
        university -> Text,
        gender -> Gender,
        discovery -> Discovery,
        experience -> Experience,
        first_time -> Bool,
        why -> Text,
        tshirt_size -> TshirtSize,
        dietary_preference -> Nullable<Jsonb>,
        accessibility_needs -> AccessibilityType,
        dvcs -> Nullable<Text>,
        linkedin -> Nullable<Text>,
        website -> Nullable<Text>,
        allows_cv_sharing -> Bool,
        allows_marketing -> Bool,
        created_at -> Timestamptz,
        check_in -> Nullable<Timestamptz>,
        comment -> Text,
        is_at_end -> Bool,
        status -> Status,
    }
}

diesel::joinable!(member_of -> application (user));

diesel::table! {
    use diesel::sql_types::Uuid;

    member_of (user) {
        user -> Uuid,
        team -> Uuid,
    }
}

diesel::joinable!(member_of -> team (team));

diesel::table! {
    use diesel::sql_types::{Uuid, Nullable, Text, Int4};

    team (id) {
        id -> Uuid,
        name -> Text,
        score -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(application, member_of, team);

diesel::table! {
    use diesel::sql_types::Text;

    preinscription (email) {
        email -> Text,
    }
}

diesel::table! {
    use super::sql_types::{Difficulty};
    use diesel::sql_types::{Uuid, Nullable, Text, Jsonb, Int2, Bool, Timestamptz};

    puzzle (id) {
        id -> Uuid,
        categories -> Jsonb,
        difficulty -> Difficulty,
        clues -> Nullable<Jsonb>,
        points -> Int2,
        name -> Text,
        prompt -> Text,
        start -> Nullable<Timestamptz>,
        end -> Nullable<Timestamptz>,
        ommit -> Bool,

    }
}

diesel::table! {
    use diesel::sql_types::{Uuid, Nullable, Jsonb, Int2, Timestamptz};

    attempt (team, puzzle) {
        team -> Uuid,
        puzzle -> Uuid,
        created_at -> Timestamptz,
        solved_at -> Nullable<Timestamptz>,
        clues_used -> Int2,
        flags -> Nullable<Jsonb>,
    }
}
