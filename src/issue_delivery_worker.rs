use crate::{
    configuration::{IssueDeliverySettings, Settings},
    domain::SubscriberEmail,
    email_client::EmailClient,
    error_chain_fmt, get_connection_pool,
};
use anyhow::Context;
use chrono::Utc;
use rand::Rng;
use sqlx::{PgPool, Postgres, Transaction};
use std::{str::FromStr, time::Duration};
use tracing::{field::display, Span};
use uuid::Uuid;

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    let email_client = configuration.email_client.client()?;
    worker_loop(connection_pool, email_client, configuration.issue_delivery).await
}

async fn worker_loop(
    pool: PgPool,
    email_client: EmailClient,
    settings: IssueDeliverySettings,
) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&pool, &email_client, &settings).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
        }
    }
}

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(
	skip_all,
	fields(
		newsletter_issue_id=tracing::field::Empty,
		subscriber_email=tracing::field::Empty
	),
	err
)]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
    settings: &IssueDeliverySettings,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let task = dequeue_task(pool).await?;
    if task.is_none() {
        return Ok(ExecutionOutcome::EmptyQueue);
    }
    let (mut transaction, issue_id, email, n_retries) = task.unwrap();
    dbg!(&n_retries);
    Span::current()
        .record("newsletter_issue_id", &display(issue_id))
        .record("subscriber_email", &display(&email));
    match SubscriberEmail::from_str(&email) {
        Ok(email) => {
            let issue = get_issue(pool, issue_id).await?;
            if let Err(e) = email_client
                .send_email(
                    &email,
                    &issue.title,
                    &issue.html_content,
                    &issue.text_content,
                )
                .await
            {
                if retry_task(
                    e,
                    &mut transaction,
                    issue_id,
                    email.as_ref(),
                    n_retries,
                    settings,
                )
                .await
                .is_ok()
                {
                    transaction.commit().await?;
                    return Ok(ExecutionOutcome::TaskCompleted);
                }
            }
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Skipping a confirmed subscriber. \
                 Their stored contact details are invalid"
            );
        }
    }
    delete_task(&mut transaction, issue_id, &email).await?;
    transaction.commit().await?;
    Ok(ExecutionOutcome::TaskCompleted)
}

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(skip_all)]
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String, i16)>, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let r = sqlx::query!(
        r#"
		SELECT newsletter_issue_id, subscriber_email, n_retries
		FROM issue_delivery_queue
        WHERE execute_after <= now()
		FOR UPDATE
		SKIP LOCKED
		LIMIT 1
		"#
    )
    .fetch_optional(&mut transaction)
    .await?;
    if let Some(r) = r {
        Ok(Some((
            transaction,
            r.newsletter_issue_id,
            r.subscriber_email,
            r.n_retries,
        )))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
async fn delete_task(
    transaction: &mut PgTransaction,
    issue_id: Uuid,
    email: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
		DELETE FROM issue_delivery_queue
		WHERE
			newsletter_issue_id = $1 AND
			subscriber_email = $2
		"#,
        issue_id,
        email
    )
    .execute(transaction)
    .await?;
    Ok(())
}

struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(skip_all)]
async fn get_issue(pool: &PgPool, issue_id: Uuid) -> Result<NewsletterIssue, anyhow::Error> {
    let issue = sqlx::query_as!(
        NewsletterIssue,
        r#"
		SELECT title, text_content, html_content
		FROM newsletter_issues
		WHERE newsletter_issue_id = $1
		"#,
        issue_id
    )
    .fetch_one(pool)
    .await?;
    Ok(issue)
}

/// Retry using exponential backoff with full-jitter
#[tracing::instrument(skip_all, fields(error=%error, n_retries=n_retries))]
async fn retry_task(
    error: reqwest::Error,
    transaction: &mut PgTransaction,
    issue_id: Uuid,
    email: &str,
    n_retries: i16,
    settings: &IssueDeliverySettings,
) -> Result<(), anyhow::Error> {
    let n_retries = n_retries + 1;
    if n_retries >= settings.max_retries {
        anyhow::bail!("Max retries reached {}. Skipping.", n_retries);
    }
    let backoff = get_expo_backoff_full_jitter(
        settings.backoff_base_secs * 1000,
        settings.backoff_cap_secs * 1000,
        n_retries as u32,
    );
    let execute_after = Utc::now() + chrono::Duration::milliseconds(backoff);
    sqlx::query!(
        r#"
        UPDATE issue_delivery_queue
        SET
            n_retries = $1,
            execute_after = $2
        WHERE
			newsletter_issue_id = $3 AND
			subscriber_email = $4
		"#,
        n_retries,
        execute_after,
        issue_id,
        email
    )
    .execute(transaction)
    .await
    .context("Failed to set task for retry. Skipping.")?;
    tracing::info!("Issue scheduled to retry after {} milliseconds.", backoff);
    Ok(())
}

/// Using: https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
fn get_expo_backoff_full_jitter(base: i64, cap: i64, n: u32) -> i64 {
    let mut rng = rand::thread_rng();
    let expo = cap.min(2i64.pow(n) * base);
    rng.gen_range(0..=expo)
}
