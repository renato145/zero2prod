use crate::routes::TEMPLATES;
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn change_password_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let flash_msgs = flash_messages.iter().collect::<Vec<_>>();
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("flash_msgs", &flash_msgs);
        TEMPLATES.render("password.html", &context).unwrap()
    };
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body))
}
