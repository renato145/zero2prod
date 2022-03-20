use crate::{routes::TEMPLATES, utils::e500};
use actix_web::{http::header::ContentType, web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;

pub async fn delivery_process(pool: web::Data<PgPool>) -> Result<HttpResponse, actix_web::Error> {
    let queue_len = get_queue_len(&pool).await.map_err(e500)?;
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("queue_len", &queue_len);
        TEMPLATES.render("delivery_process.html", &context).unwrap()
    };
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body))
}

async fn get_queue_len(pool: &PgPool) -> Result<i64, anyhow::Error> {
    let row = sqlx::query!(
        r#"
		SELECT count(newsletter_issue_id) as "n!"
		FROM issue_delivery_queue
		"#
    )
    .fetch_one(pool)
    .await
    .context("Failed to get delivery queue lenght.")?;
    Ok(row.n)
}
