use std::sync::Arc;

use delivery_application::services::webhook_service::WebhookService;

pub async fn webhook_dispatcher_worker(webhook_service: Arc<WebhookService>) {
    tracing::info!("Webhook dispatcher worker started");

    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));

    loop {
        interval.tick().await;

        match webhook_service.process_pending().await {
            Ok(count) => {
                if count > 0 {
                    tracing::debug!(count, "Processed webhook deliveries");
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Webhook dispatcher error");
            }
        }
    }
}
