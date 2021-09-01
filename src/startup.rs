use crate::{
    configuration::get_configuration,
    routes::{health_check_route, subscribe},
};
use rocket::{
    fairing::{self, AdHoc},
    figment::Figment,
    Build, Rocket,
};
use sqlx::{postgres::PgConnectOptions, ConnectOptions, PgPool};

pub fn get_rocket(config: Option<Figment>, connection_pool: Option<PgPool>) -> Rocket<Build> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let figment = config.unwrap_or_else(|| {
        Figment::from(rocket::Config::default()).merge(("port", configuration.application_port))
    });

    rocket::custom(figment)
        .attach(stage_db(connection_pool))
        // The book uses `tracing_actix_web` to create requests ids
        // I ignored this part as Rocket have not tracing yet, but check
        // https://github.com/SergioBenitez/Rocket/pull/1579 in the future
        .mount("/", routes![health_check_route, subscribe])
}

fn stage_db(connection_pool: Option<PgPool>) -> AdHoc {
    AdHoc::try_on_ignite("SQLx Database", |rocket| async {
        init_db(rocket, connection_pool).await
    })
}

async fn init_db(rocket: Rocket<Build>, connection_pool: Option<PgPool>) -> fairing::Result {
    let db = match connection_pool {
        Some(db) => db,
        None => {
            let mut opts: PgConnectOptions = get_configuration()
                .expect("Failed to read configuration.")
                .database
                .into();
            opts.disable_statement_logging();
            let db = match PgPool::connect_with(opts).await {
                Ok(db) => db,
                Err(e) => {
                    error!("Failed to connect to SQLx database: {}", e);
                    return Err(rocket);
                }
            };
            db
        }
    };

    Ok(rocket.manage(db))
}
