use anyhow::Result;
use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use zero2prod::{
    configuration::get_configuration,
    idempotency_expiration_worker, issue_delivery_worker,
    telemetry::{get_subscriber, init_subscriber},
    Application,
};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    let issue_delivery_worker_task = tokio::spawn(issue_delivery_worker::run_worker_until_stopped(
        configuration.clone(),
    ));
    let idempotency_expiration_worker_task = tokio::spawn(
        idempotency_expiration_worker::run_worker_until_stopped(configuration),
    );

    tokio::select! {
        o = application_task => report_exit("API", o),
        o = issue_delivery_worker_task => report_exit("Background worker (issue_delivery)", o),
        o = idempotency_expiration_worker_task => report_exit("Background worker (idempotency_expiration)", o),
    }

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(_)) => {
            tracing::info!("{} has exited", task_name);
        }
        Ok(Err(e)) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{} failed",
                task_name
            );
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "{}' task failed to complete",
                task_name
            );
        }
    }
}
