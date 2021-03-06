#![allow(clippy::async_yields_async)]

pub mod authentication;
pub mod configuration;
pub mod domain;
pub mod email_client;
pub mod idempotency;
pub mod idempotency_expiration_worker;
pub mod issue_delivery_worker;
pub mod routes;
pub mod session_state;
pub mod startup;
pub mod telemetry;
pub mod utils;

pub use startup::*;

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
