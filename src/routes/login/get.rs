use crate::routes::TEMPLATES;
use actix_web::{http::header::ContentType, HttpResponse};

pub async fn login_form() -> HttpResponse {
    let html_body = {
        let context = tera::Context::new();
        TEMPLATES.render("login.html", &context).unwrap()
        // .context("Failed to construct the HTML email template.")?
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
