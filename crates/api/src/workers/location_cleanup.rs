use std::sync::Arc;

use delivery_application::services::location_service::LocationService;

pub async fn location_cleanup_worker(location_service: Arc<LocationService>) {
    tracing::info!("Location cleanup worker started");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(6 * 3600));

    loop {
        interval.tick().await;

        match location_service.cleanup_old_locations().await {
            Ok(count) => {
                tracing::info!(count, "Cleaned up old location records");
            }
            Err(e) => {
                tracing::error!(error = %e, "Location cleanup error");
            }
        }
    }
}
