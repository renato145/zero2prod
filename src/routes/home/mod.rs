use super::TEMPLATES;
use actix_web::{http::header::ContentType, HttpResponse};

pub async fn home() -> HttpResponse {
    let html_body = {
        let context = tera::Context::new();
        TEMPLATES.render("home.html", &context).unwrap()
        // .context("Failed to construct the HTML email template.")?
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
