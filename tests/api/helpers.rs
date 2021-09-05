use once_cell::sync::Lazy;
use rocket::{fairing::AdHoc, tokio::sync::oneshot};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    get_rocket,
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

    let mut configuration = get_configuration().expect("Failed to read configuration.");

    // Port 0 give us a random available port
    configuration.application.port = 0;

    // Get a custom PgPool
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

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

    // Use a oneshot channel to retrieve the running port
    let (tx, rx) = oneshot::channel();
    let server = get_rocket(
        configuration.application,
        connection_pool.clone(),
        email_client,
    )
    .attach(AdHoc::on_liftoff("Config", |rocket| {
        Box::pin(async move {
            let address = format!("http://127.0.0.1:{}", rocket.config().port);
            tx.send(address).unwrap();
        })
    }));
    rocket::tokio::spawn(server.launch());
    let address = rx.await.expect("Failed to get running port.");

    TestApp {
        address,
        db_pool: connection_pool,
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
