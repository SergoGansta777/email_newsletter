use anyhow::Context;
use axum::{extract::State, Json};
use serde::Deserialize;
use sqlx::types::{chrono::Utc, Uuid};

use crate::{error::Error, ApiContext};

#[derive(Deserialize)]
pub struct Subscription {
    pub name: String,
    pub email: String,
}

pub async fn subscribe(
    ctx: State<ApiContext>,
    Json(payload): Json<Subscription>,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        payload.email,
        payload.name,
        Utc::now()
    )
    .execute(&ctx.connection_pool)
    .await
    .context("Failed to execute subscribe sql script")?;

    Ok(())
}
