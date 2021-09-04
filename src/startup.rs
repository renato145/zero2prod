use crate::{
    configuration::ApplicationSettings,
    email_client::EmailClient,
    routes::{health_check_route, subscribe},
};
use rocket::{figment::Figment, Build, Rocket};
use sqlx::PgPool;

pub fn get_rocket(
    app_configuration: ApplicationSettings,
    connection_pool: PgPool,
    email_client: EmailClient,
) -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(("port", app_configuration.port))
        .merge(("address", app_configuration.address));

    // The book uses `tracing_actix_web` to create requests ids
    // I ignored this part as Rocket have not tracing yet, but check
    // https://github.com/SergioBenitez/Rocket/pull/1579 in the future
    rocket::custom(figment)
        .manage(connection_pool)
        .manage(email_client)
        .mount("/", routes![health_check_route, subscribe])
}
