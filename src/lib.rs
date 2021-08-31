pub mod configuration;
pub mod routes;
pub mod startup;

use rocket::{figment::Figment, Build, Rocket};

use crate::routes::{health_check_route, subscribe};

#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

pub fn get_rocket(config: Option<Figment>) -> Rocket<Build> {
    let figment = config.unwrap_or_else(|| Figment::from(rocket::Config::default()));
    rocket::custom(figment).mount("/", routes![index, health_check_route, subscribe])
}
