use crate::routes::{admin::middleware::UserId, TEMPLATES};
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn change_password_form(
    _user_id: UserId,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut error_msg = String::new();
    for m in flash_messages.iter() {
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
        TEMPLATES.render("password.html", &context).unwrap()
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body))
}
