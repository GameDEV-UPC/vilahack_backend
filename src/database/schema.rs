pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tshirt_size"))]
    pub struct TshirtSize;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "experience"))]
    pub struct Experience;
}

diesel::table! {
    use diesel::sql_types::{Uuid, Nullable, Text, Jsonb, Int2};
    use super::sql_types::{TshirtSize, Experience};

    user (id) {
        id -> Uuid,
        guardian_email -> Nullable<Text>,
        name -> Text,
        pronouns -> Text,
        country -> Text,
        experience -> Experience,
        motivation -> Text,
        tshirt_size -> TshirtSize,
        dietary_preferences -> Jsonb,
        accessibility_needs -> Int2,
        linkedin -> Text,
        personal_page -> Text,
    }
}
