use crate::{
    configuration::{ApplicationSettings, DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{confirm, health_check_route, subscribe},
};
use rocket::{figment::Figment, Build, Rocket};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub async fn build(configuration: Settings) -> Rocket<Build> {
    let connection_pool = get_connection_pool(&configuration.database)
        .await
        .expect("Failed to connect to Postgres.");

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    )
    .expect("Failed to build email client.");

    get_rocket(configuration.application, connection_pool, email_client)
}

pub async fn get_connection_pool(configuration: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_with(configuration.with_db())
        .await
}

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
        .mount("/", routes![health_check_route, subscribe, confirm])
}
