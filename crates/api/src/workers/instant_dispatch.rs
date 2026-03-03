use std::sync::Arc;
use tokio::sync::mpsc;

use delivery_application::services::dispatch_service::DispatchService;

pub async fn instant_dispatch_worker(
    dispatch_service: Arc<DispatchService>,
    mut rx: mpsc::Receiver<uuid::Uuid>,
) {
    tracing::info!("Instant dispatch worker started");

    while let Some(order_id) = rx.recv().await {
        tracing::info!(order_id = %order_id, "Processing instant dispatch");
        if let Err(e) = dispatch_service.dispatch_instant_order(order_id).await {
            tracing::error!(order_id = %order_id, error = %e, "Instant dispatch failed");
        }
    }

    tracing::warn!("Instant dispatch worker channel closed");
}
