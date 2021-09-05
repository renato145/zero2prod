use std::convert::{TryFrom, TryInto};

use chrono::Utc;
use rocket::{form::Form, http::Status, State};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[derive(FromForm)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client),
    fields(
        request_id = %Uuid::new_v4(),
        subcriber_email = %form.email,
        subcriber_name = %form.name
    )
)]
#[post("/subscriptions", data = "<form>")]
pub async fn subscribe(
    form: Form<FormData>,
    pool: &State<PgPool>,
    email_client: &State<EmailClient>,
) -> Status {
    let new_subscriber = match form.into_inner().try_into() {
        Ok(subcriber) => subcriber,
        Err(_) => return Status::BadRequest,
    };

    if insert_subscriber(&**pool, &new_subscriber).await.is_err() {
        return Status::InternalServerError;
    }

    if email_client
        .send_email(
            new_subscriber.email,
            "Welcome!",
            "Welcome to our newsletter!",
            "Welcome to our newsletter!",
        )
        .await
        .is_err()
    {
        return Status::InternalServerError;
    }

    Status::Ok
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
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
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
