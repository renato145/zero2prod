use crate::{routes::TEMPLATES, HmacSecret};
use actix_web::{http::header::ContentType, web, HttpResponse};
use anyhow::Result;
use hmac::{Hmac, Mac};
use secrecy::ExposeSecret;
use serde::Deserialize;
use tracing::warn;

#[derive(Deserialize)]
pub struct QueryParams {
    error: String,
    tag: String,
}

impl QueryParams {
    fn verify(self, secret: &HmacSecret) -> Result<String> {
        let tag = hex::decode(self.tag)?;
        let query_string = format!("error={}", urlencoding::Encoded::new(&self.error));
        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(secret.0.expose_secret().as_bytes()).unwrap();
        mac.update(query_string.as_bytes());
        mac.verify_slice(&tag)?;
        Ok(self.error)
    }
}

pub async fn login_form(
    query: Option<web::Query<QueryParams>>,
    secret: web::Data<HmacSecret>,
) -> HttpResponse {
    let error = query.map(|query| match query.0.verify(&secret) {
        Ok(error) => error,
        Err(e) => {
            warn!(error.message = %e, error.caused_chain = ?e,
                "Failed to verify query parameters using the HMAC tag"
            );
            "".to_string()
        }
    });
    let html_body = {
        let mut context = tera::Context::new();
        context.insert("error", &error);
        TEMPLATES.render("login.html", &context).unwrap()
        // .context("Failed to construct the HTML email template.")?
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(html_body)
}
