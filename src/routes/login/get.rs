use crate::routes::TEMPLATES;
use actix_web::{http::header::ContentType, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryParams {
    error: Option<String>,
}

pub async fn login_form(query: web::Query<QueryParams>) -> HttpResponse {
    let error = query.0.error;
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("error", &error);
        TEMPLATES.render("login.html", &context).unwrap()
        // .context("Failed to construct the HTML email template.")?
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
