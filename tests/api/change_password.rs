use crate::helpers::{assert_is_redirect_to, spawn_app};
use uuid::Uuid;

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_change_password().await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Act
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_check": &new_password
        }))
        .await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    // Act - Login
    app.do_login().await;

    // Act - Try to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": Uuid::new_v4().to_string(),
            "new_password": &new_password,
            "new_password_check": &another_new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    // Act - Follow the redirect
    let response = app.get_change_password().await;
    let html_page = response.text().await.unwrap();
    assert!(html_page.contains(
        "You entered two different new passwords - \
         the field values must match.",
    ));
}

#[tokio::test]
async fn changing_password_works() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Act - Login
    let response = app.do_login().await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    // Act - Follow the redirect
    let html_page = app.get_change_password().await.text().await.unwrap();
    assert!(html_page.contains("Your password has been changed.",));

    // Act - Logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // Act - Follow the redirect
    let html_page = app.get_login().await.text().await.unwrap();
    assert!(html_page.contains("You have successfully logged out."));

    // Act - Login using the new password
    let response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &new_password,
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/dashboard");
}

#[tokio::test]
async fn new_password_should_be_have_valid_length() {
    // Arrange
    let app = spawn_app().await;
    let new_password = "short";

    // Act - Login
    app.do_login().await;

    // Act - Change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    // Act - Follow the redirect
    let response = app.get_change_password().await;
    let html_page = response.text().await.unwrap();
    assert!(html_page.contains("New password should have between 12 and 128 characters."));
}

#[tokio::test]
async fn current_password_must_be_valid() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    // Act - Login
    app.do_login().await;

    // Act - Try to change password
    let response = app
        .post_change_password(&serde_json::json!({
            "current_password": &wrong_password,
            "new_password": &new_password,
            "new_password_check": &new_password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    // Act - Follow the redirect
    let response = app.get_change_password().await;
    let html_page = response.text().await.unwrap();
    assert!(html_page.contains("The current password is incorrect.",));
}
