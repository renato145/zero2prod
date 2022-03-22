use crate::{
    configuration::{IdempotencySettings, Settings},
    get_connection_pool,
};
use anyhow::Context;
use rand::Rng;
use sqlx::{postgres::types::PgInterval, PgPool};
use std::time::Duration;

const MAX_RETRIES: usize = 3;

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    worker_loop(connection_pool, configuration.idempotency).await
}

async fn worker_loop(pool: PgPool, settings: IdempotencySettings) -> Result<(), anyhow::Error> {
    let mut retries = 0;
    let frequency = settings.expiration_frequency_secs as f32;
    let expiration_interval = Duration::from_secs(settings.expiration_secs)
        .try_into()
        .unwrap();
    // When server restarts wait half the expiration time to start working
    tokio::time::sleep(Duration::from_secs_f32(frequency / 2.0)).await;
    loop {
        if try_execute_task(&pool, &expiration_interval).await.is_err() {
            retries += 1;
            if retries < MAX_RETRIES {
                tokio::time::sleep(Duration::from_secs_f32(add_jitter(10.0))).await;
                continue;
            } else {
                retries = 0;
            }
        }
        tokio::time::sleep(Duration::from_secs_f32(add_jitter(frequency))).await;
    }
}

/// Adds a random jittering around `secs` of +/- 10%.
fn add_jitter(secs: f32) -> f32 {
    let mut rng = rand::thread_rng();
    let x = secs * 0.1;
    let jitter = rng.gen_range(-x..=x);
    (secs + jitter).max(0.0)
}

#[tracing::instrument(skip_all, err)]
pub async fn try_execute_task(
    pool: &PgPool,
    expiration_interval: &PgInterval,
) -> Result<(), anyhow::Error> {
    let n = sqlx::query!(
        r#"
		DELETE FROM idempotency
		wHERE (created_at + $1) < now()
		"#,
        expiration_interval
    )
    .execute(pool)
    .await
    .context("Failed to clear expired idempotency keys from database.")?
    .rows_affected();
    tracing::info!("Removed {} expired keys.", n);
    Ok(())
}
