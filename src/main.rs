use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::get_configuration,
    get_rocket,
    telemetry::{get_subscriber, init_subscriber},
};

#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPoolOptions::new()
        .connect_timeout(Duration::from_secs(2))
        .connect_with(configuration.database.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    get_rocket(configuration, connection_pool)
}
