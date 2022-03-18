use crate::routes::TEMPLATES;
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn publish_newsletter_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_msg = String::new();
    for m in flash_messages.iter() {
        error_msg.push_str(m.content());
    }
    let error_msg = (!error_msg.is_empty()).then(|| error_msg);
    let idempotency_key = uuid::Uuid::new_v4().to_string();
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("error_msg", &error_msg);
        context.insert("idempotency_key", &idempotency_key);
        TEMPLATES.render("newsletters.html", &context).unwrap()
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
