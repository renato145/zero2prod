use crate::{
    configuration::get_configuration,
    routes::{health_check_route, subscribe},
};
use rocket::{
    fairing::{self, AdHoc},
    figment::Figment,
    Build, Rocket,
};
use sqlx::{postgres::PgConnectOptions, ConnectOptions};

pub fn get_rocket(config: Option<Figment>) -> Rocket<Build> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let figment = config.unwrap_or_else(|| {
        Figment::from(rocket::Config::default()).merge(("port", configuration.application_port))
    });

    rocket::custom(figment)
        .attach(stage())
        .mount("/", routes![health_check_route, subscribe])
}

type Db = sqlx::PgPool;

fn stage() -> AdHoc {
    AdHoc::try_on_ignite("SQLx Database", init_db)
}

async fn init_db(rocket: Rocket<Build>) -> fairing::Result {
    let mut opts: PgConnectOptions = get_configuration()
        .expect("Failed to read configuration.")
        .database
        .into();
    opts.disable_statement_logging();

    let db = match Db::connect_with(opts).await {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to connect to SQLx database: {}", e);
            return Err(rocket);
        }
    };

    Ok(rocket.manage(db))
}
