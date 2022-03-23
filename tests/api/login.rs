use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = spawn_app().await;

    // Act - Try to login
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/login");

    // Act - Follow the redirect
    let response = app.get_login().await;
    let html_page = response.text().await.unwrap();
    assert!(html_page.contains(r#"Authentication failed"#));

    // Act - Reload the login page
    let response = app.get_login().await;
    let html_page = response.text().await.unwrap();
    assert!(!html_page.contains(r#"Authentication failed"#));
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    // Arrange
    let app = spawn_app().await;

    // Act - Login
    let response = app.do_login().await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Follow the redirect
    let response = app.get_admin_dashboard().await;
    let html_page = response.text().await.unwrap();
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));
}
