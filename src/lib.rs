use axum::{http::StatusCode, response::Html, routing::get, serve::Serve, Router};
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener) -> anyhow::Result<Serve<Router, Router>> {
    let app = Router::new()
        .route("/", get(get_hello_world))
        .route("/health_check", get(health_check));

    Ok(axum::serve(listener, app))
}

async fn get_hello_world() -> Html<&'static str> {
    Html("<h1>Hello World</h1>")
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
