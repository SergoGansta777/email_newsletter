use axum::{
    routing::{get, post},
    serve::Serve,
    Router,
};
use routes::{health_check, subscribe};
use sqlx::PgPool;
use tokio::net::TcpListener;

pub mod configuration;
pub mod error;
pub mod routes;
pub mod startup;

pub type Result<T, E = error::Error> = std::result::Result<T, E>;

#[derive(Clone)]
pub struct ApiContext {
    db_pool: PgPool,
}

pub async fn run(listener: TcpListener, db_pool: PgPool) -> anyhow::Result<Serve<Router, Router>> {
    let app_context = ApiContext { db_pool };

    sqlx::migrate!().run(&app_context.db_pool).await?;

    let app_router = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(app_context);

    Ok(axum::serve(listener, app_router))
}
