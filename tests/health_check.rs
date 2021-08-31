use rocket::{fairing::AdHoc, figment::Figment, tokio::sync::oneshot};
use zero2prod::get_rocket;

#[rocket::async_test]
async fn health_check_works() {
    let port = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://127.0.0.1:{}/health_check", port))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_app() -> u16 {
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
    rx.await.expect("Failed to get running port.")
}
