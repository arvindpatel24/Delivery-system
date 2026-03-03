use std::sync::Arc;

use delivery_application::services::dispatch_service::DispatchService;

pub async fn stale_order_checker_worker(dispatch_service: Arc<DispatchService>) {
    tracing::info!("Stale order checker worker started");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

    loop {
        interval.tick().await;

        match dispatch_service.handle_stale_offers().await {
            Ok(order_ids) => {
                if !order_ids.is_empty() {
                    tracing::info!(count = order_ids.len(), "Re-dispatched stale orders");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Stale order check error");
            }
        }
    }
}
