use crate::routes::TEMPLATES;
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_msg = String::new();
    for m in flash_messages.iter().filter(|m| m.level() == Level::Error) {
        error_msg.push_str(m.content());
    }
    let error_msg = if error_msg.is_empty() {
        None
    } else {
        Some(error_msg)
    };
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("error_msg", &error_msg);
        TEMPLATES.render("login.html", &context).unwrap()
        // .context("Failed to construct the HTML email template.")?
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
