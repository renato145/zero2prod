use rocket::{form::Form, http::Status};

#[derive(FromForm)]
pub struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions", data = "<_form>")]
pub async fn subscribe(_form: Form<FormData>) -> Status {
    Status::Ok
}
