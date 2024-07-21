use axum::{extract::State, response::IntoResponse, Json};
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::types::{chrono::Utc, Uuid};

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    error::Error,
    ApiContext,
};

#[derive(Deserialize)]
pub struct Subscription {
    pub name: String,
    pub email: String,
}

impl TryFrom<Subscription> for NewSubscriber {
    type Error = String;

    fn try_from(value: Subscription) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

pub async fn subscribe(
    ctx: State<ApiContext>,
    Json(payload): Json<Subscription>,
) -> impl IntoResponse {
    let new_subscriber = match payload.try_into() {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match insert_subscriber(new_subscriber, ctx).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
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
        new_subscriber.email.as_ref(),
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
