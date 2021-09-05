use once_cell::sync::Lazy;
use rocket::{fairing::AdHoc, tokio::sync::oneshot};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    build,
    configuration::{get_configuration, DatabaseSettings},
    get_connection_pool,
    telemetry::{get_subscriber, init_subscriber},
};

// Ensure that 'tracing' stack is only initialized once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    // Set up tracing
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        // Port 0 give us a random available port
        c.application.port = 0;
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        c
    };

    // Create and migrate database
    configure_database(&configuration.database).await;

    // Launch app as background task
    let (tx, rx) = oneshot::channel();
    let server =
        build(configuration.clone())
            .await
            .attach(AdHoc::on_liftoff("Get port", |rocket| {
                Box::pin(async move {
                    let address = format!("http://127.0.0.1:{}", rocket.config().port);
                    tx.send(address).unwrap();
                })
            }));
    rocket::tokio::spawn(server.launch());
    let address = rx.await.expect("Failed to get running port.");

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database)
            .await
            .expect("Failed to connect to the database."),
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres.");

    connection
        .execute(&*format!(r#"CREATE DATABASE "{}""#, config.database_name))
        .await
        .expect("Failed to create database.");

    // Execute migrations
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database.");

    connection_pool
}
