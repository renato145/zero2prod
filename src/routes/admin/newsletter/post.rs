use crate::{
    authentication::UserId,
    domain::NewsletterIssue,
    error_chain_fmt,
    idempotency::{save_response, try_processing, IdempotencyKey, NextAction},
    utils::see_other,
};
use actix_web::{error::InternalError, web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use serde::Deserialize;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
    idempotency_key: String,
}

#[derive(thiserror::Error)]
pub enum NewsletterError {
    #[error("{0}")]
    ValidationError(String),
    #[error("Something went wrong.")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for NewsletterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl From<String> for NewsletterError {
    fn from(e: String) -> Self {
        Self::ValidationError(e)
    }
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip_all,
    fields(user_id=%&*user_id)
)]
pub async fn publish_newsletter(
    user_id: web::ReqData<UserId>,
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, InternalError<NewsletterError>> {
    let user_id = user_id.into_inner();
    let FormData {
        title,
        text_content,
        html_content,
        idempotency_key,
    } = form.0;
    let idempotency_key: IdempotencyKey =
        idempotency_key.try_into().map_err(newsletter_redirect)?;
    let newsletter_issue =
        NewsletterIssue::try_new(title, text_content, html_content).map_err(newsletter_redirect)?;
    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(newsletter_redirect)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
    };
    let issue_id = insert_newsletter_issue(&mut transaction, &newsletter_issue)
        .await
        .context("Failed to insert newsletter_issue into db.")
        .map_err(newsletter_redirect)?;
    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue delivery tasks")
        .map_err(newsletter_redirect)?;
    let response = see_other("/admin/newsletters");
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(newsletter_redirect)?;
    success_message().send();
    Ok(response)
}

/// Redirect to the newsletters page with an error message.
#[tracing::instrument(fields(e=%e))]
fn newsletter_redirect(
    e: impl Into<NewsletterError> + std::fmt::Display,
) -> InternalError<NewsletterError> {
    let e = e.into();
    FlashMessage::error(e.to_string()).send();
    InternalError::from_response(e, see_other("/admin/newsletters"))
}

fn success_message() -> FlashMessage {
    FlashMessage::success(
        "The newsletter issue has been accepted - \
                 emails will go out shortly.",
    )
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue: &NewsletterIssue,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at
        )
        VALUES ($1, $2, $3, $4, now())
        "#,
        newsletter_issue_id,
        newsletter_issue.title(),
        newsletter_issue.text_content(),
        newsletter_issue.html_content()
    )
    .execute(transaction)
    .await?;
    Ok(newsletter_issue_id)
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue (
            newsletter_issue_id,
            subscriber_email,
            n_retries,
            execute_after
        )
        SELECT $1, email, 0, now()
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id
    )
    .execute(transaction)
    .await?;
    Ok(())
}
