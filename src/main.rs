use zero2prod::{
    build,
    configuration::get_configuration,
    telemetry::{get_subscriber, init_subscriber},
};

#[macro_use]
extern crate rocket;

#[launch]
async fn rocket() -> _ {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    build(configuration).await
}
