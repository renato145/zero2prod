use rocket::{Build, Rocket, figment::Figment, form::Form, http::Status};

#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

#[get("/health_check")]
async fn health_check() -> Status {
    Status::Ok
}

#[derive(FromForm)]
struct FormData {
    email: String,
    name: String,
}

#[post("/subscriptions", data = "<_form>")]
async fn subscribe(_form: Form<FormData>) -> Status {
    Status::Ok
}

pub fn get_rocket(config: Option<Figment>) -> Rocket<Build> {
    let figment = config.unwrap_or_else(|| Figment::from(rocket::Config::default()));
    rocket::custom(figment).mount("/", routes![index, health_check, subscribe])
}
