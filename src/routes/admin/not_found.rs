use crate::routes::TEMPLATES;
use actix_web::{http::header::ContentType, HttpResponse};

pub async fn not_found() -> HttpResponse {
    let html_body = TEMPLATES.render("404.html", &tera::Context::new()).unwrap();
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
