use super::middleware::UserId;
use crate::{routes::TEMPLATES, utils::e500};
use actix_web::{
    http::header::ContentType,
    web::{self, ReqData},
    HttpResponse,
};
use anyhow::{Context, Result};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn admin_dashboard(
    user_id: ReqData<UserId>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = get_username(user_id.0, &pool).await.map_err(e500)?;
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("username", &username);
        TEMPLATES.render("admin_dashboard.html", &context).unwrap()
        // .context("Failed to construct the HTML email template.")?
    };
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body))
}

#[tracing::instrument(name = "Get username", skip(pool))]
pub async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(row.username)
}
