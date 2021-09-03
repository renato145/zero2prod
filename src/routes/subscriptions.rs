use chrono::Utc;
use rocket::{form::Form, http::Status, State};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(FromForm)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        request_id = %Uuid::new_v4(),
        subcriber_email = %form.email,
        subcriber_name = %form.name
    )
)]
#[post("/subscriptions", data = "<form>")]
pub async fn subscribe(form: Form<FormData>, pool: &State<PgPool>) -> Status {
    let form = form.into_inner();
    let name = match SubscriberName::parse(form.name) {
        Ok(name) => name,
        Err(_) => return Status::BadRequest,
    };
    let email = match SubscriberEmail::parse(form.email) {
        Ok(email) => email,
        Err(_) => return Status::BadRequest,
    };
    let new_subscriber = NewSubscriber { email, name };

    match insert_subscriber(&**pool, &new_subscriber).await {
        Ok(_) => Status::Ok,
        Err(_) => Status::InternalServerError,
    }
}

#[tracing::instrument(
    name = "Saving a new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
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
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
