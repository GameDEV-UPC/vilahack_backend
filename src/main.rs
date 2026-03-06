use std::{env, sync::Arc};

use axum::{
    Router,
    http::{HeaderValue, Method},
    routing::{get, put},
};
use tower_http::cors::{Any, CorsLayer};

use tracing_subscriber::{EnvFilter, FmtSubscriber};

use backend::{api, database};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok(); // Load .env file as env variables
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let bind = env::var("BIND_ADDRESS").expect("Missing server's `BIND_ADDRESS` env variable");
    let database_url = env::var("DATABASE_URL").expect("Missing `DATABASE_URL` env variable");

    let allow_origin = env::var("ALLOW_ORIGIN")
        .expect("Missing `ALLOW_ORIGIN`")
        .parse::<HeaderValue>()
        .expect("Failed to parse `ALLOW_ORIGIN`");
    let cors_layer = CorsLayer::new()
        .allow_origin(allow_origin)
        .allow_methods([Method::GET, Method::PUT])
        .allow_headers(Any);

    let state = Arc::new(database::Pool::from_url(&database_url).await.unwrap());
    let router = Router::new()
        .route("/v0/preinscribe", put(api::preinscription::preinscribe))
        .route("/v0/user/sign_up", put(api::user::sign_up))
        // .route("/v0/user/check_in", put(api::user::check_in))
        .route("/v0/user/qr.svg", get(api::user::qr))
        .layer(cors_layer)
        .with_state(state);

    tracing::info!("Starting server at {bind}...");
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
