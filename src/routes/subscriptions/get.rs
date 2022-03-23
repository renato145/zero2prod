use crate::routes::TEMPLATES;
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn subscriptions_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let flash_msgs = flash_messages.iter().collect::<Vec<_>>();
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("flash_msgs", &flash_msgs);
        TEMPLATES.render("subscriptions.html", &context).unwrap()
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
