use rocket::http::Status;

#[derive(FromForm)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
#[get("/subscriptions/confirm?<parameters..>")]
pub async fn confirm(parameters: Parameters) -> Status {
    Status::Ok
}
