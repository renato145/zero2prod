use super::TEMPLATES;
use actix_web::{http::header::ContentType, HttpResponse};

pub async fn home() -> HttpResponse {
    let html_body = TEMPLATES
        .render("home.html", &tera::Context::new())
        .unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
