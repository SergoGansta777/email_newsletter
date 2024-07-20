use std::{
    future::IntoFuture,
    net::{Ipv4Addr, SocketAddr},
};

use newsletter_deliverer::run;
use tokio::net::TcpListener;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request");

    // Arange
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app() -> String {
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to addr");
    let port = listener.local_addr().unwrap().port();

    let server = run(listener).await.expect("Failed to get server");

    let _ = tokio::spawn(server.into_future());

    format!("http://127.0.0.1:{}", port)
}
