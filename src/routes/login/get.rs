use crate::routes::TEMPLATES;
use actix_web::{http::header::ContentType, web, HttpResponse};

pub async fn login_form(request: web::HttpRequest) -> HttpResponse {
    let error = request.cookie("_flash").map(|c| c.value().to_string());
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
