mod admin;
mod health_check;
mod home;
mod login;
mod subscriptions;
mod subscriptions_confirm;

pub use admin::*;
pub use health_check::*;
pub use home::*;
pub use login::*;
use once_cell::sync::Lazy;
pub use subscriptions::*;
pub use subscriptions_confirm::*;
use tera::Tera;

static TEMPLATES: Lazy<Tera> = Lazy::new(|| {
    let mut tera = Tera::new("templates/**/*.html").unwrap();
    tera.autoescape_on(vec![]);
    tera
});
