use axum::{
    http::StatusCode,
    routing::{get, post},
    serve::Serve,
    Json, Router,
};
use serde::Deserialize;
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener) -> anyhow::Result<Serve<Router, Router>> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe));

    Ok(axum::serve(listener, app))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize)]
struct Subscription {
    name: Option<String>,
    email: Option<String>,
}

async fn subscribe(Json(payload): Json<Subscription>) -> StatusCode {
    if payload.name.is_none() || payload.email.is_none() {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::OK
    }
}
