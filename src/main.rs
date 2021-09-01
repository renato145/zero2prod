use env_logger::Env;
use zero2prod::get_rocket;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    get_rocket(None, None)
}
