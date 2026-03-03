use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookStatus {
    Pending,
    Delivered,
    Failed,
    Dead,
}

impl WebhookStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Delivered => "delivered",
            Self::Failed => "failed",
            Self::Dead => "dead",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookOutbox {
    pub id: Uuid,
    pub order_id: Uuid,
    pub shop_id: Uuid,
    pub webhook_url: String,
    pub payload: serde_json::Value,
    pub status: WebhookStatus,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub next_retry_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}
