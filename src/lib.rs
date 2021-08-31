use rocket::{figment::Figment, http::Status, Build, Rocket};

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

pub fn get_rocket(config: Option<Figment>) -> Rocket<Build> {
    let figment = config.unwrap_or_else(|| Figment::from(rocket::Config::default()));
    rocket::custom(figment).mount("/", routes![index, health_check])
}
