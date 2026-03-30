use std::{env, str::FromStr, sync::Arc};

use axum::{
    Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION},
    },
    routing::{get, put},
};
use tower_http::cors::CorsLayer;

use backend::{api, database, telemetry::init_tracing_subscriber};
use tracing::Level;

#[tracing::instrument]
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // Load .env file as env variables
    let deployment = env::var("DEPLOYMENT").expect("Missing `DEPLOYMENT` env variable");
    let log_level =
        Level::from_str(&env::var("RUST_LOG").expect("Missing `TRACER_NAME` env variable"))
            .expect("Could not parce `RUST_LOG` env variable");

    let _guard = init_tracing_subscriber(deployment, log_level);

    let bind = env::var("BIND_ADDRESS").expect("Missing server's `BIND_ADDRESS` env variable");
    let database_url = env::var("DATABASE_URL").expect("Missing `DATABASE_URL` env variable");

    let allow_origins: Vec<HeaderValue> = env::var("ALLOW_ORIGIN")
        .expect("Missing `ALLOW_ORIGIN`")
        .split(' ')
        .map(|origin| {
            origin
                .parse::<HeaderValue>()
                .expect("Failed to parse `ALLOW_ORIGIN`")
        })
        .collect();

    let cors_layer = CorsLayer::new()
        .allow_origin(allow_origins)
        .allow_methods([Method::GET, Method::PUT])
        .allow_headers([AUTHORIZATION, ACCEPT]);

    let state = Arc::new(database::Pool::from_url(&database_url).await.unwrap());
    let router = Router::new()
        .route("/v0/preinscribe", put(api::preinscription::preinscribe))
        .route("/v0/user/application", get(api::user::get))
        .route("/v0/user/application", put(api::user::apply))
        .route("/v0/user/application/update", put(api::user::update))
        .route("/v0/user/check_in", put(api::user::check_in))
        .route("/v0/user/qr.svg", get(api::user::qr))
        .route("/v0/team", get(api::team::summary))
        .route("/v0/team/{name}", put(api::team::new))
        .route("/v0/team/join/{id}", put(api::team::join))
        .route("/v0/team/leave", put(api::team::leave))
        .route("/v0/team/update/{name}", put(api::team::update))
        .layer(cors_layer)
        .with_state(state);

    tracing::info!("Starting server at {bind}...");
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
