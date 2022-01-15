use super::middleware::UserId;
use crate::{session_state::TypedSession, utils::see_other};
use actix_web::{web::ReqData, HttpResponse};
use actix_web_flash_messages::FlashMessage;

pub async fn log_out(
    session: TypedSession,
    _user_id: ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    session.log_out();
    FlashMessage::info("You have successfully logged out.").send();
    Ok(see_other("/login"))
}
