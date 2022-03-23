use crate::{routes::TEMPLATES, session_state::TypedSession, utils::see_other};
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;

pub async fn login_form(
    flash_messages: IncomingFlashMessages,
    session: TypedSession,
) -> HttpResponse {
    if session.is_logged() {
        return see_other("/admin/dashboard");
    }
    let mut error_msg = String::new();
    for m in flash_messages.iter() {
        error_msg.push_str(m.content());
    }
    let error_msg = (!error_msg.is_empty()).then(|| error_msg);
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("error_msg", &error_msg);
        TEMPLATES.render("login.html", &context).unwrap()
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
