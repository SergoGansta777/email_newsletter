use std::{
    future::IntoFuture,
    net::{Ipv4Addr, SocketAddr},
};

use newsletter_deliverer::run;
use serde_json::json;
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

/// Spin up an instance of application
/// and returns its address (i.e. http://localhost:XXXX)
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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = json!({
        "name": "Sergey Nekhoroshev",
        "email": "sergo777ser777@gmail.com"
    });

    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (json!({"name": "Sergey"}), "missing the email"),
        (
            json!({"email": "sergo777ser777@gmail.com"}),
            "missing the name",
        ),
        (json!({}), "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
