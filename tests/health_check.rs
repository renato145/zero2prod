use rocket::{fairing::AdHoc, figment::Figment, tokio::sync::oneshot};
use zero2prod::get_rocket;

async fn spawn_app() -> String {
    // Port 0 give us a random available port
    let figment = Figment::from(rocket::Config::default()).merge(("port", 0));

    // Use a oneshot channel to retrieve the running port
    let (tx, rx) = oneshot::channel();
    let server = get_rocket(Some(figment)).attach(AdHoc::on_liftoff("Config", |rocket| {
        Box::pin(async move {
            tx.send(rocket.config().port).unwrap();
        })
    }));

    rocket::tokio::spawn(server.launch());
    let port = rx.await.expect("Failed to get running port.");
    format!("http://127.0.0.1:{}", port)
}

#[rocket::async_test]
async fn health_check_works() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[rocket::async_test]
async fn subscribe_returns_200_for_valid_form_data() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=1e%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(format!("{}/subscriptions", address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[rocket::async_test]
async fn subscribe_returns_client_error_when_data_is_missing() {
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", address))
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
