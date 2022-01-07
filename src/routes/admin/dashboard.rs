use crate::{routes::TEMPLATES, session_state::TypedSession};
use actix_web::{http::header::ContentType, web, HttpResponse};
use anyhow::{Context, Result};
use reqwest::header::LOCATION;
use sqlx::PgPool;
use uuid::Uuid;

// Return an opaque 500 while preserving the error's root cause
fn e500<T>(e: T) -> actix_web::error::InternalError<T> {
    actix_web::error::InternalError::from_response(e, HttpResponse::InternalServerError().finish())
}

pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish());
    };
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
async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String> {
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
