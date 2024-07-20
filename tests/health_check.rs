use std::future::IntoFuture;

use newsletter_deliverer::run;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get("http://127.0.0.1:8080/health_check")
        .send()
        .await
        .expect("Failed to execute request");

    // Arange
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app() {
    let server = run().await.expect("Failed to get server");

    let _ = tokio::spawn(server.into_future());
}
