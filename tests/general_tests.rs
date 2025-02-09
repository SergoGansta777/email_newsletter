use std::{
    future::IntoFuture,
    net::{Ipv4Addr, SocketAddr},
};

use anyhow::Context;
use newsletter_deliverer::{
    configuration::{get_configuration, DatabaseSettings},
    run,
};
use serde_json::json;
use sqlx::{Connection, PgConnection, PgPool};
use test_case::test_case;
use tokio::net::TcpListener;
use uuid::Uuid;

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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_data() {
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

#[test_case(json!({"name": "Sergey"}), "missing the email")]
#[test_case(json!({"email": "sergo777ser777@gmail.com"}), "missing the name")]
#[test_case(json!({}), "missing both name and email")]
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing(
    invalid_body: serde_json::Value,
    error_message: &str,
) {
    // Arrange
    let app = spawn_app().await.unwrap();
    let client = reqwest::Client::new();

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

#[test_case(json!({"name": "Sergey", "email" : ""}), "empty email")]
#[test_case(json!({"name": "","email": "sergo777ser777@gmail.com"}),  "empty name")]
#[test_case(json!({"name": "Sergey","email": "defently-not-valid_email.com"}), "invalid email")]
#[tokio::test]
async fn subscribe_returns_a_400_when_field_are_present_but_empty(
    body: serde_json::Value,
    description: &str,
) {
    // Arrange
    let app = spawn_app().await.unwrap();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with 400 Bad Request when the payload was {}.",
        description
    );
}

/// Spin up an instance of application
/// and returns its address (i.e. http://localhost:XXXX)
async fn spawn_app() -> anyhow::Result<TestApp> {
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to addr");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let connection_pool = configure_database(&configuration.database).await.unwrap();

    let server = run(listener, connection_pool.clone())
        .await
        .context("Failed to get server")?;

    let _ = tokio::spawn(server.into_future());

    Ok(TestApp {
        address,
        db_pool: connection_pool,
    })
}

pub async fn configure_database(config: &DatabaseSettings) -> anyhow::Result<PgPool> {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    // Create the new database
    let db_name = &config.database_name;
    let create_db_query = format!(r#"CREATE DATABASE "{}""#, db_name);
    sqlx::query(&create_db_query)
        .execute(&mut connection)
        .await
        .context("Failed to create database")?;

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    Ok(connection_pool)
}
