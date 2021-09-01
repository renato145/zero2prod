use crate::{
    configuration::Settings,
    routes::{health_check_route, subscribe},
};
use rocket::{fairing::AdHoc, figment::Figment, Build, Rocket};
use sqlx::PgPool;

pub fn get_rocket(configuration: Settings, connection_pool: PgPool) -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(("port", configuration.application.port))
        .merge(("address", configuration.application.address));

    rocket::custom(figment)
        .attach(stage_db(connection_pool))
        // The book uses `tracing_actix_web` to create requests ids
        // I ignored this part as Rocket have not tracing yet, but check
        // https://github.com/SergioBenitez/Rocket/pull/1579 in the future
        .mount("/", routes![health_check_route, subscribe])
}

fn stage_db(connection_pool: PgPool) -> AdHoc {
    AdHoc::try_on_ignite("SQLx Database", |rocket| async {
        Ok(rocket.manage(connection_pool))
    })
}
