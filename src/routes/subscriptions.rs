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

pub async fn subscribe(
    ctx: State<ApiContext>,
    Json(payload): Json<Subscription>,
) -> impl IntoResponse {
    let new_subscriber = match parse_subscriber(payload) {
        Ok(subscriber) => subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match insert_subscriber(new_subscriber, ctx).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub fn parse_subscriber(subscripton: Subscription) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(subscripton.name)?;
    let email = SubscriberEmail::parse(subscripton.email)?;
    Ok(NewSubscriber { email, name })
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
