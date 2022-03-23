use fake::{
    faker::{internet::en::SafeEmail, name::en::Name},
    Fake,
};
use reqwest::{Client, Url};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

const APP_PORT: u16 = 8000;
const EMAIL_PORT: u16 = 10000;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next();
    let n = args.next().map(|s| s.parse().unwrap()).unwrap_or(10);
    println!("Adding {n} subscribers...");

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();
    let email_server = {
        let listener = std::net::TcpListener::bind(format!("127.0.0.1:{EMAIL_PORT}")).unwrap();
        MockServer::builder().listener(listener).start().await
    };
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount_as_scoped(&email_server)
        .await;

    for _ in 0..n {
        let (name, email) = create_subscriber(&client, &email_server).await;
        println!("- {name}: {email}");
    }
    println!("\nDone :)");
}

async fn create_subscriber(client: &Client, email_server: &MockServer) -> (String, String) {
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(&serde_json::json!({
        "name": name,
        "email": email
    }))
    .unwrap();

    client
        .post(format!("http://localhost:{APP_PORT}/subscribe"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request")
        .error_for_status()
        .unwrap();

    let email_request = email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    let confirmation_link = get_confirmation_links(&email_request);
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    (name, email)
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub fn get_confirmation_links(email_request: &wiremock::Request) -> ConfirmationLinks {
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    let get_link = |s: &str| {
        let links = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect::<Vec<_>>();
        assert_eq!(links.len(), 1);
        let raw_link = links[0].as_str().to_owned();
        let mut confirmation_link = Url::parse(&raw_link).unwrap();
        // Make sure we don't call a random API
        assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
        // Include test port
        confirmation_link.set_port(Some(APP_PORT)).unwrap();
        confirmation_link
    };
    let html = get_link(&body["HtmlBody"].as_str().unwrap());
    let plain_text = get_link(&body["TextBody"].as_str().unwrap());
    ConfirmationLinks { html, plain_text }
}
