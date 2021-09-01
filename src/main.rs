use zero2prod::{
    get_rocket,
    telemetry::{get_subscriber, init_subscriber},
};

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    get_rocket(None, None)
}
