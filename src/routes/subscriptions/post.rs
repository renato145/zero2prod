use crate::{
    domain::{NewSubscriber, SubscriptionToken},
    email_client::EmailClient,
    routes::{error_chain_fmt, TEMPLATES},
    utils::see_other,
    ApplicationBaseUrl,
};
use actix_web::{error::InternalError, web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use chrono::Utc;
use sqlx::{PgPool, Postgres, Transaction};
use std::convert::{TryFrom, TryInto};
use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = value.name.parse()?;
        let email = value.email.parse()?;
        Ok(NewSubscriber { email, name })
    }
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error("{msg} already exists, use another email.")]
    DuplicatedEmail {
        msg: String,
        source: Box<dyn sqlx::error::DatabaseError>,
    },
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<String> for SubscribeError {
    fn from(e: String) -> Self {
        Self::ValidationError(e)
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subcriber_email = tracing::field::Empty,
        subcriber_name = tracing::field::Empty
    )
)]
pub async fn subscribe(
    form: Result<web::Form<FormData>, actix_web::Error>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, InternalError<SubscribeError>> {
    let form = match form {
        Ok(f) => {
            tracing::Span::current().record("subscriber_email", &tracing::field::display(&f.email));
            tracing::Span::current().record("subscriber_name", &tracing::field::display(&f.name));
            Ok(f.0)
        }
        Err(e) => Err(e.to_string()),
    }
    .map_err(subscriptions_redirect)?;
    let new_subscriber = form.try_into().map_err(subscriptions_redirect)?;
    let mut transaction = pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool.")
        .map_err(subscriptions_redirect)?;
    let subscriber_id = match check_existing_pending_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to check if new subscriber is present in the database.")
        .map_err(subscriptions_redirect)?
    {
        Some(id) => id,
        None => {
            let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
                .await
                .map_err(subscriptions_redirect)?;
            subscriber_id
        }
    };
    let subscription_token = SubscriptionToken::new();
    store_token(&mut transaction, subscriber_id, subscription_token.as_ref())
        .await
        .context("Failed to store the confirmation token for a new subscriber.")
        .map_err(subscriptions_redirect)?;
    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.")
        .map_err(subscriptions_redirect)?;
    let subscriber_email = new_subscriber.email.to_string();
    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url.0,
        subscription_token,
    )
    .await
    .map_err(subscriptions_redirect)?;

    FlashMessage::info(format!(
        "A confirmation email was sent to {}",
        subscriber_email
    ))
    .send();
    Ok(see_other("/subscriptions"))
}

/// Redirect to the subscriptions page with an error message.
#[tracing::instrument(fields(e=%e))]
fn subscriptions_redirect(
    e: impl Into<SubscribeError> + std::fmt::Display,
) -> InternalError<SubscribeError> {
    let e = e.into();
    FlashMessage::error(e.to_string()).send();
    InternalError::from_response(e, see_other("/subscriptions"))
}

#[tracing::instrument(
    name = "Checking if a new subscriber already exists in the database in pending state",
    skip(transaction, new_subscriber)
)]
pub async fn check_existing_pending_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT id
        FROM subscriptions
        wHERE email = $1 AND name = $2 AND status = 'pending_confirmation'
        "#,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref()
    )
    .fetch_optional(transaction)
    .await?;
    Ok(result.map(|r| (r.id)))
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(transaction, new_subscriber)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, SubscribeError> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(e) if e.to_string().contains("duplicate key value") => {
            SubscribeError::DuplicatedEmail {
                msg: new_subscriber.email.to_string(),
                source: e,
            }
        }
        e => anyhow::Error::from(e)
            .context("Failed to insert new subscriber in the database.")
            .into(),
    })?;
    Ok(subscriber_id)
}

pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to store a subscription token."
        )
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(transaction, subscription_token)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(StoreTokenError)?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
    subscription_token: SubscriptionToken,
) -> Result<(), anyhow::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url,
        subscription_token.as_ref()
    );

    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    let html_body = {
        let mut context = tera::Context::new();
        context.insert("confirmation_link", &confirmation_link);
        TEMPLATES
            .render("email.html", &context)
            .context("Failed to construct the HTML email template.")?
    };

    email_client
        .send_email(&new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
        .context("Failed to send a confirmation email.")
}
