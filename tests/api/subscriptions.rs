use crate::helpers::{assert_is_redirect_to, spawn_app};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn subscribe_returns_correct_message_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    let response = app.post_subscriptions(body.into()).await;
    assert_is_redirect_to(&response, "/subscriptions");

    // Assert
    let html_page = app.get_subscriptions_html().await;
    assert!(
        html_page.contains("A confirmation email was sent to ursula_le_guin@gmail.com"),
        "Current page: {}",
        html_page
    );
}

#[tokio::test]
async fn subscribe_persist_the_new_subscribe() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    let saved = sqlx::query!("SELECT email,name,status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_for_valid_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;
}

#[tokio::test]
async fn subscribe_sends_confirmation_email_with_link() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

#[tokio::test]
async fn subscribe_two_times_sends_confirmation_email_twice() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body.into()).await;
    app.post_subscriptions(body.into()).await;
}

#[tokio::test]
async fn subscribe_returns_client_error_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_subscriptions(invalid_body.into()).await;
        assert_is_redirect_to(&response, "/subscriptions");

        // Assert
        let html_page = app.get_subscriptions_html().await;
        assert!(
            html_page.contains("Parse error: missing field"),
            "The API did not fail with Client Error when the payload was {}.\n\
             Current page: {}",
            error_message,
            html_page
        );
    }
}

#[tokio::test]
async fn subscribe_fails_when_fields_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        (
            "name=&email=ursula_le_guin%40gmail.com",
            "empty name",
            "Subscriber name can't be empty.",
        ),
        (
            "name=Ursula&email=",
            "empty email",
            "Subscriber email can't be empty.",
        ),
        (
            "name=Ursula&email=definitely-not-an-email",
            "invalid email",
            "definitely-not-an-email is not a valid subscriber email.",
        ),
    ];
    for (body, description, expected) in test_cases {
        // Act
        let response = app.post_subscriptions(body.into()).await;
        assert_is_redirect_to(&response, "/subscriptions");

        // Assert
        let html_page = app.get_subscriptions_html().await;
        assert!(
            html_page.contains(expected),
            "The API did not fail when the payload was {}.\n\
             Current page: {}",
            description,
            html_page
        );
    }
}

#[tokio::test]
async fn subscribe_fail_if_there_is_a_fatal_database_error() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // Sabotage the database
    sqlx::query!("ALTER TABLE subscriptions DROP COLUMN email;")
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.post_subscriptions(body.into()).await;
    assert_is_redirect_to(&response, "/subscriptions");

    // Assert
    let html_page = app.get_subscriptions_html().await;
    assert!(
        html_page.contains("Something went wrong."),
        "Current page: {}",
        html_page
    );
}

#[tokio::test]
async fn subscribe_fails_when_email_already_exists() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    // Subscribe and confirm subscription
    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);
    reqwest::get(confirmation_links.html).await.unwrap();

    let response = app.post_subscriptions(body.into()).await;
    assert_is_redirect_to(&response, "/subscriptions");

    // Assert
    let html_page = app.get_subscriptions_html().await;
    assert!(
        html_page.contains("ursula_le_guin@gmail.com already exists, use another email."),
        "Current page: {}",
        html_page
    );
}
