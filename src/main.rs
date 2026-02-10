use std::{env, sync::Arc};

use axum::{Router, routing::get};

use fastrace::{collector::Config, collector::ConsoleReporter, prelude::*};

use backend::{api, database};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // Load .env file as env variables
    fastrace::set_reporter(ConsoleReporter, Config::default()); // Set tracing reporter

    let bind = env::var("BIND_ADDRESS").expect("Missing server's `BIND_ADDRESS` env variable");

    let database_url = env::var("DATABASE_URL").expect("Missing `DATABASE_URL` env variable");

    let state = Arc::new(database::Pool::from_url(&database_url).await.unwrap());
    let router = Router::new()
        .route("/v0/cat", get(api::test))
        .with_state(state);

    LocalSpan::add_event(Event::new("Starting server..."));
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
