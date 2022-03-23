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
    let flash_msgs = flash_messages.iter().collect::<Vec<_>>();
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("flash_msgs", &flash_msgs);
        TEMPLATES.render("login.html", &context).unwrap()
    };
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
