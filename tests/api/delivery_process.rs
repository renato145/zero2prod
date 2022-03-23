use crate::helpers::spawn_app;
use crate::newsletter::{create_confirmed_subscriber, publis_newsletter};

#[tokio::test]
async fn page_shows_pending_queue() {
    // Arrange
    let app = spawn_app().await;
    let (_, email) = create_confirmed_subscriber(&app).await;
    app.do_login().await;
    publis_newsletter(&app).await;

    // Act
    let html_page = app.get_delivery_process_html().await;

    // Assert
    assert!(html_page.contains(&email));
}
