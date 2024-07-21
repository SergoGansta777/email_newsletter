use axum::{extract::State, Json};
use serde::Deserialize;
use sqlx::types::{chrono::Utc, Uuid};

use crate::{
    domain::{NewSubscriber, SubscriberName},
    error::Error,
    ApiContext,
};

#[derive(Deserialize)]
pub struct Subscription {
    pub name: String,
    pub email: String,
}

pub async fn subscribe(
    ctx: State<ApiContext>,
    Json(payload): Json<Subscription>,
) -> Result<(), Error> {
    let new_subscriber = NewSubscriber {
        email: payload.email,
        name: SubscriberName::parse(payload.name).expect("Subscriber name validation failed"),
    };

    insert_subscriber(new_subscriber, ctx).await?;

    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(ctx, new_subscriber)
)]
async fn insert_subscriber(
    new_subscriber: NewSubscriber,
    ctx: State<ApiContext>,
) -> Result<(), Error> {
    sqlx::query!(
        r#"
            INSERT INTO subscriptions (id, email, name, subscribed_at)
            VALUES ($1, $2, $3, $4)
            "#,
        Uuid::new_v4(),
        new_subscriber.email,
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(&ctx.connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
