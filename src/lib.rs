use std::{
    future::IntoFuture,
    net::{Ipv4Addr, SocketAddr},
};

use axum::{http::StatusCode, response::Html, routing::get, serve::Serve, Router};
use tokio::net::TcpListener;

pub async fn run() -> anyhow::Result<Serve<Router, Router>> {
    let app = Router::new()
        .route("/", get(get_hello_world))
        .route("/health_check", get(health_check));

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    let listener = TcpListener::bind(addr).await?;

    Ok(axum::serve(listener, app))
}

async fn get_hello_world() -> Html<&'static str> {
    Html("<h1>Hello World</h1>")
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
