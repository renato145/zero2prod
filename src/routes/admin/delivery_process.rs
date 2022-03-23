use crate::{routes::TEMPLATES, utils::e500};
use actix_web::{http::header::ContentType, web, HttpResponse};
use anyhow::Context;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub async fn delivery_process(pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    let queue_data = get_queue_data(&pool).await.map_err(e500)?;
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("queue_len", &queue_data.len());
        context.insert("queue_data", &queue_data);
        TEMPLATES.render("delivery_process.html", &context).unwrap()
    };
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body))
}

#[derive(serde::Serialize)]
struct QueueData {
    issue_title: String,
    subscriber_email: String,
    n_retries: i16,
    next_retry: String,
}

async fn get_queue_data(pool: &PgPool) -> Result<Vec<QueueData>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
		SELECT title, subscriber_email, n_retries, execute_after
		FROM issue_delivery_queue a
            INNER JOIN newsletter_issues b ON a.newsletter_issue_id = b.newsletter_issue_id
        ORDER BY execute_after
		"#
    )
    .fetch_all(pool)
    .await
    .context("Failed to get delivery queue data.")?
    .into_iter()
    .map(|row| {
        let execute_after: DateTime<Utc> = row.execute_after;
        let next_retry = (execute_after - Utc::now())
            .to_std()
            .map(|d| humantime::format_duration(d).to_string())
            .unwrap_or_else(|_| "ready for retry".to_string());
        QueueData {
            issue_title: row.title,
            subscriber_email: row.subscriber_email,
            n_retries: row.n_retries,
            next_retry,
        }
    })
    .collect::<Vec<_>>();
    Ok(rows)
}
