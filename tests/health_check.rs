use std::{
    future::IntoFuture,
    net::{Ipv4Addr, SocketAddr},
};

use anyhow::Context;
use newsletter_deliverer::{configuration::get_configuration, run};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, Connection, PgConnection, PgPool};
use tokio::net::TcpListener;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await.unwrap();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    // Arange
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

/// Spin up an instance of application
/// and returns its address (i.e. http://localhost:XXXX)
async fn spawn_app() -> anyhow::Result<TestApp> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to addr");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let db_pool = PgPoolOptions::new()
        .max_connections(50)
        .connect(&configuration.database.connection_string())
        .await
        .context("Could not connect to database url")?;

    let server = run(listener, db_pool.clone())
        .await
        .context("Failed to get server")?;

    let _ = tokio::spawn(server.into_future());

    Ok(TestApp { address, db_pool })
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await.unwrap();
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");
    let client = reqwest::Client::new();

    // Act
    let body = json!({
        "name": "Sergey Nekhoroshev",
        "email": "sergo777ser777@gmail.com"
    });

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    // Assert
    assert_eq!(200, response.status().as_u16());
    assert_eq!(saved.email, "sergo777ser777@gmail.com");
    assert_eq!(saved.name, "Sergey Nekhoroshev");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await.unwrap();
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
            .post(&format!("{}/subscriptions", &app.address))
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 when the payload was {}.",
            error_message
        );
    }
}
