use rocket::http::Status;

#[get("/health_check")]
pub async fn health_check_route() -> Status {
    Status::Ok
}
