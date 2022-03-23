use crate::routes::TEMPLATES;
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn publish_newsletter_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let flash_msgs = flash_messages.iter().collect::<Vec<_>>();
    let idempotency_key = uuid::Uuid::new_v4().to_string();
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("idempotency_key", &idempotency_key);
        context.insert("flash_msgs", &flash_msgs);
        TEMPLATES.render("newsletters.html", &context).unwrap()
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
