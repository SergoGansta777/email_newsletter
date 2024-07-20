use axum::{
    routing::{get, post},
    serve::Serve,
    Router,
};
use routes::{health_check, subscribe};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};

pub mod configuration;
pub mod error;
pub mod routes;
pub mod startup;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

#[derive(Clone)]
pub struct ApiContext {
    connection_pool: PgPool,
}

pub async fn run(
    listener: TcpListener,
    connection_pool: PgPool,
) -> anyhow::Result<Serve<Router, Router>> {
    let app_context = ApiContext { connection_pool };

    sqlx::migrate!().run(&app_context.connection_pool).await?;

    let app_router = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().include_headers(true))
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                .on_failure(DefaultOnFailure::new().level(tracing::Level::ERROR)),
        )
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(app_context);

    Ok(axum::serve(listener, app_router))
}
