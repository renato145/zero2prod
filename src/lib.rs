pub mod configuration;
pub mod routes;
pub mod startup;

use rocket::{figment::Figment, Build, Rocket};

use crate::{
    configuration::get_configuration,
    routes::{health_check_route, subscribe},
};

#[macro_use]
extern crate rocket;

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

pub fn get_rocket(config: Option<Figment>) -> Rocket<Build> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let figment = config
        .unwrap_or_else(|| Figment::from(rocket::Config::default()))
        .merge(("port", configuration.application_port));

    rocket::custom(figment).mount("/", routes![index, health_check_route, subscribe])
}
