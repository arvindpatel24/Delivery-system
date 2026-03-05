use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use delivery_domain::entities::webhook::{WebhookOutbox, WebhookStatus};
use delivery_domain::errors::DomainResult;
use delivery_domain::ports::{WebhookOutboxRepository, WebhookSender};

pub struct WebhookService {
    pub outbox_repo: Arc<dyn WebhookOutboxRepository>,
    pub sender: Arc<dyn WebhookSender>,
    pub max_retries: i32,
    pub timeout_secs: u64,
}

impl WebhookService {
    pub async fn enqueue(
        &self,
        order_id: Uuid,
        shop_id: Uuid,
        webhook_url: String,
        payload: serde_json::Value,
    ) -> DomainResult<()> {
        let entry = WebhookOutbox {
            id: Uuid::now_v7(),
            order_id,
            shop_id,
            webhook_url,
            payload,
            status: WebhookStatus::Pending,
            attempts: 0,
            last_error: None,
            next_retry_at: Utc::now(),
            created_at: Utc::now(),
            delivered_at: None,
        };
        self.outbox_repo.insert(&entry).await
    }

    pub async fn process_pending(&self) -> DomainResult<u64> {
        let pending = self.outbox_repo.fetch_pending(50).await?;
        let mut processed = 0u64;

        for entry in pending {
            match self.sender.send(&entry.webhook_url, &entry.payload, self.timeout_secs).await {
                Ok(()) => {
                    self.outbox_repo.mark_delivered(entry.id).await?;
                    processed += 1;
                }
                Err(err) => {
                    let new_attempts = entry.attempts + 1;
                    if new_attempts >= self.max_retries {
                        self.outbox_repo.mark_dead(entry.id).await?;
                    } else {
                        let backoff_secs = 2i64.pow(new_attempts as u32);
                        let next_retry = Utc::now() + Duration::seconds(backoff_secs);
                        self.outbox_repo
                            .mark_failed(entry.id, &err, next_retry)
                            .await?;
                    }
                }
            }
        }

        Ok(processed)
    }
}
