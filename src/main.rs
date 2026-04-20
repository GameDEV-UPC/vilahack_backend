use std::sync::Arc;

use axum::{
    Router,
    http::{
        Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    routing::{get, put},
};
use tower_http::cors::CorsLayer;

use backend::{api, config::CONFIG, database, telemetry::init_tracing_subscriber};

#[tracing::instrument]
#[tokio::main]
async fn main() {
    let _guard = init_tracing_subscriber();

    let cors_layer = CorsLayer::new()
        .allow_origin(CONFIG.allowed_origins.clone())
        .allow_methods([Method::GET, Method::PUT])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let state = Arc::new(
        database::Pool::from_url(&CONFIG.database_url)
            .await
            .unwrap(),
    );
    let router = Router::new()
        .route("/v0/preinscribe", put(api::preinscription::preinscribe))
        .route("/v0/user/application", get(api::user::get))
        .route("/v0/user/application", put(api::user::apply))
        .route("/v0/user/application/update", put(api::user::update))
        .route("/v0/user/application/index", get(api::user::index))
        .route(
            "/v0/user/attendance/accept",
            put(api::user::accept_attendance),
        )
        .route(
            "/v0/user/attendance/confirm",
            put(api::user::confirm_attendance),
        )
        .route(
            "/v0/user/attendance/cancel",
            put(api::user::cancel_attendance),
        )
        .route("/v0/user/attendance/check_in", put(api::user::check_in))
        .route("/v0/user/qr.svg", get(api::user::qr))
        .route("/v0/team", get(api::team::summary))
        .route("/v0/team", put(api::team::new))
        .route("/v0/team/join", put(api::team::join))
        .route("/v0/team/leave", put(api::team::leave))
        .route("/v0/team/update", put(api::team::update))
        .route("/v0/puzzle", get(api::puzzle::get))
        .route("/v0/puzzle/all", get(api::puzzle::get_all))
        .route(
            "/v0/puzzle/all/by_category",
            get(api::puzzle::get_all_by_category),
        )
        .layer(cors_layer)
        .with_state(state);

    tracing::info!("Starting server at {}...", CONFIG.bind_address);
    let listener = tokio::net::TcpListener::bind(CONFIG.bind_address)
        .await
        .unwrap();
    axum::serve(listener, router).await.unwrap();
}
