use once_cell::sync::Lazy;
use rocket::{fairing::AdHoc, tokio::sync::oneshot};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
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

async fn spawn_app() -> TestApp {
    // Set up tracing
    Lazy::force(&TRACING);

    let mut configuration = get_configuration().expect("Failed to read configuration.");

    // Port 0 give us a random available port
    configuration.application.port = 0;

    // Get a custom PgPool
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    // Use a oneshot channel to retrieve the running port
    let (tx, rx) = oneshot::channel();
    let server = get_rocket(configuration, connection_pool.clone()).attach(AdHoc::on_liftoff(
        "Config",
        |rocket| {
            Box::pin(async move {
                let address = format!("http://127.0.0.1:{}", rocket.config().port);
                tx.send(address).unwrap();
            })
        },
    ));
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

#[rocket::async_test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[rocket::async_test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email,name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[rocket::async_test]
async fn subscribe_returns_client_error_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert!(
            response.status().is_client_error(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[rocket::async_test]
async fn subscribe_returns_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return 400 Bad Request when the payload was {}.",
            description
        );
    }
}
