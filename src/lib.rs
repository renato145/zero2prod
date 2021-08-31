use rocket::{http::Status, Build, Rocket};

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

pub fn get_rocket() -> Rocket<Build> {
    rocket::build().mount("/", routes![index, health_check])
}
