use std::time::Duration;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

const EMAIL_PORT: u16 = 10000;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next();
    let delay = args.next().map(|s| s.parse().unwrap()).unwrap_or(2);
    println!("Delay set to {delay} seconds.");
    println!("Starting fake email server...");
    let email_server = {
        let listener = std::net::TcpListener::bind(format!("127.0.0.1:{EMAIL_PORT}")).unwrap();
        MockServer::builder().listener(listener).start().await
    };
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(delay)))
        .mount_as_scoped(&email_server)
        .await;
    println!("Receiving connections");
    std::thread::sleep(Duration::MAX);
}
