use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use delivery_domain::entities::WebhookOutbox;
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::WebhookOutboxRepository;

use super::models::WebhookOutboxRow;

pub struct PgWebhookOutboxRepo {
    pub pool: PgPool,
}

#[async_trait]
impl WebhookOutboxRepository for PgWebhookOutboxRepo {
    async fn insert(&self, entry: &WebhookOutbox) -> DomainResult<()> {
        sqlx::query(
            r#"
            INSERT INTO webhook_outbox (id, order_id, shop_id, webhook_url, payload, status, attempts, next_retry_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6::webhook_status, $7, $8, $9)
            "#,
        )
        .bind(entry.id)
        .bind(entry.order_id)
        .bind(entry.shop_id)
        .bind(&entry.webhook_url)
        .bind(&entry.payload)
        .bind(entry.status.as_str())
        .bind(entry.attempts)
        .bind(entry.next_retry_at)
        .bind(entry.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn fetch_pending(&self, limit: i64) -> DomainResult<Vec<WebhookOutbox>> {
        let rows = sqlx::query_as::<_, WebhookOutboxRow>(
            r#"
            SELECT id, order_id, shop_id, webhook_url, payload, status::text, attempts, last_error, next_retry_at, created_at, delivered_at
            FROM webhook_outbox
            WHERE (status = 'pending' OR status = 'failed') AND next_retry_at <= NOW()
            ORDER BY next_retry_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn mark_delivered(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query(
            "UPDATE webhook_outbox SET status = 'delivered'::webhook_status, delivered_at = NOW(), attempts = attempts + 1 WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn mark_failed(&self, id: Uuid, error: &str, next_retry_at: DateTime<Utc>) -> DomainResult<()> {
        sqlx::query(
            r#"
            UPDATE webhook_outbox
            SET status = 'failed'::webhook_status, last_error = $2, attempts = attempts + 1, next_retry_at = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(error)
        .bind(next_retry_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn mark_dead(&self, id: Uuid) -> DomainResult<()> {
        sqlx::query(
            "UPDATE webhook_outbox SET status = 'dead'::webhook_status, attempts = attempts + 1 WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }
}
