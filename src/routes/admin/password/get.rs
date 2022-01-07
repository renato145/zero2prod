use crate::{
    routes::TEMPLATES,
    session_state::TypedSession,
    utils::{e500, see_other},
};
use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::{IncomingFlashMessages, Level};

pub async fn change_password_form(
    session: TypedSession,
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    if session.get_user_id().map_err(e500)?.is_none() {
        return Ok(see_other("/login"));
    }
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
        TEMPLATES.render("password.html", &context).unwrap()
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body))
}
