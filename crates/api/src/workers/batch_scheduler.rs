use std::sync::Arc;

use chrono::Timelike;
use delivery_application::services::batch_service::BatchService;
use sqlx::PgPool;

pub async fn batch_scheduler_worker(
    batch_service: Arc<BatchService>,
    pool: PgPool,
    schedule_hours: Vec<u32>,
) {
    tracing::info!(?schedule_hours, "Batch scheduler worker started");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        interval.tick().await;

        let now = chrono::Utc::now();
        let current_hour_24 = now.hour();
        let current_minute = now.minute();

        // Only trigger at the start of scheduled hours (first minute)
        if schedule_hours.contains(&current_hour_24) && current_minute == 0 {
            // Advisory lock to prevent duplicate runs across replicas
            let lock_result = sqlx::query_scalar::<_, bool>(
                "SELECT pg_try_advisory_lock(42)"
            )
            .fetch_one(&pool)
            .await;

            match lock_result {
                Ok(true) => {
                    tracing::info!(hour = current_hour_24, "Running batch scheduler");
                    match batch_service.run_batch(current_hour_24 as i32).await {
                        Ok(run_id) => {
                            tracing::info!(run_id = %run_id, "Batch run completed successfully");
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Batch run returned error");
                        }
                    }

                    // Release advisory lock
                    let _ = sqlx::query("SELECT pg_advisory_unlock(42)")
                        .execute(&pool)
                        .await;
                }
                Ok(false) => {
                    tracing::debug!("Another replica is running the batch scheduler");
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to acquire advisory lock");
                }
            }
        }
    }
}
