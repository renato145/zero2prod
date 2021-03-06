use crate::helpers::{assert_is_redirect_to, spawn_app, ConfirmationLinks, TestApp};
use fake::{
    faker::{internet::en::SafeEmail, name::en::Name},
    Fake,
};
use reqwest::Response;
use std::time::Duration;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

/// Use the public API of the application under test to create
/// an unconfirmed subscriber
/// Returns: name, email and confimation links
async fn create_unconfirmed_subscriber(app: &TestApp) -> (String, String, ConfirmationLinks) {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(&serde_json::json!({
        "name": name,
        "email": email
    }))
    .unwrap();

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    (name, email, app.get_confirmation_links(&email_request))
}

/// Returns: name and email
pub async fn create_confirmed_subscriber(app: &TestApp) -> (String, String) {
    let (name, email, confirmation_link) = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    (name, email)
}

pub async fn publis_newsletter(app: &TestApp) -> Response {
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    app.post_publish_newsletters(&newsletter_request_body).await
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    app.do_login().await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act - Submit newsletter form
    let response = publis_newsletter(&app).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("The newsletter issue has been accepted - emails will go out shortly."),
        "Current value: {}",
        html_page
    );
    app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.do_login().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Submit newsletter form
    let response = publis_newsletter(&app).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("The newsletter issue has been accepted - emails will go out shortly."),
        "Current value: {}",
        html_page
    );
    app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_newsletter_form() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_publish_newsletters().await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_publish_a_newsletter() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = publis_newsletter(&app).await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.do_login().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_publish_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("The newsletter issue has been accepted - emails will go out shortly."),
        "Current value: {}",
        html_page
    );

    // Act - Part 3 - Submit newsletter form **again**
    let response = app.post_publish_newsletters(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 4 - Follow the redirect
    let html_page = app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("The newsletter issue has been accepted - emails will go out shortly."),
        "Current value: {}",
        html_page
    );
    app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.do_login().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        // Setting a long delay to ensure that the second requiest
        // arrives before the first one completes
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Submit 2 newsletter forms concurrently
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response1 = app.post_publish_newsletters(&newsletter_request_body);
    let response2 = app.post_publish_newsletters(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);

    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
    app.dispatch_all_pending_emails().await;
    // Mock verifies on Drop that we have sent the newsletter email **once**
}

#[tokio::test]
async fn newsletters_deliver_retries_on_external_error() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.do_login().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Submit newsletter
    publis_newsletter(&app).await;
    app.dispatch_all_pending_emails().await;

    // Assert
    let saved = sqlx::query!("SELECT n_retries FROM issue_delivery_queue")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch n_retries.");
    assert_eq!(saved.n_retries, 1);
}

#[tokio::test]
async fn newsletters_deliver_skip_retries_after_max_retries_setting() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.do_login().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(2)
        .mount(&app.email_server)
        .await;

    // Act - Submit newsletter
    publis_newsletter(&app).await;
    app.dispatch_all_pending_emails().await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    app.dispatch_all_pending_emails().await;

    // Assert
    let saved = sqlx::query!("SELECT n_retries FROM issue_delivery_queue")
        .fetch_optional(&app.db_pool)
        .await
        .expect("Failed to fetch query.");
    assert!(saved.is_none());
}

#[tokio::test]
async fn idempotency_keys_are_removed_after_they_expire() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.do_login().await;

    // Act
    publis_newsletter(&app).await;
    tokio::time::sleep(Duration::from_secs_f32(2.0)).await;
    app.remove_expired_idempotency_keys().await;

    // Assert
    let row = sqlx::query!(r#"SELECT COUNT(*) as "n!" FROM idempotency"#)
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(row.n, 0);
}

#[tokio::test]
async fn non_expired_idempotency_keys_are_not_removed() {
    // Arrange
    let mut app = spawn_app().await;
    app.idempotency_settings.expiration_secs = 2;
    create_confirmed_subscriber(&app).await;
    app.do_login().await;

    // Act
    let newsletter_request_body_1 = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let idempotency_key = uuid::Uuid::new_v4().to_string();
    let newsletter_request_body_2 = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": idempotency_key
    });
    app.post_publish_newsletters(&newsletter_request_body_1)
        .await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    app.post_publish_newsletters(&newsletter_request_body_2)
        .await;
    app.remove_expired_idempotency_keys().await;

    // Assert
    let row = sqlx::query!(r#"SELECT idempotency_key FROM idempotency"#)
        .fetch_one(&app.db_pool)
        .await
        .unwrap();
    assert_eq!(row.idempotency_key, idempotency_key);
}
