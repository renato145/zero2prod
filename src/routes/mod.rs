mod admin;
mod health_check;
mod home;
mod login;
mod newsletter;
mod subscriptions;
mod subscriptions_confirm;

pub use admin::*;
pub use health_check::*;
pub use home::*;
pub use login::*;
pub use newsletter::*;
use once_cell::sync::Lazy;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
use tera::Tera;

static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::new("templates/**/*.html").unwrap();
    tera.autoescape_on(vec![".html"]);
    tera
});

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
