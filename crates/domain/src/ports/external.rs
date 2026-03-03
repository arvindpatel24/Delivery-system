use async_trait::async_trait;

use crate::errors::DomainResult;
use crate::value_objects::Location;

#[async_trait]
pub trait Geocoder: Send + Sync {
    async fn geocode(&self, address: &str) -> DomainResult<Location>;
    async fn reverse_geocode(&self, location: Location) -> DomainResult<String>;
}

#[async_trait]
pub trait WebhookSender: Send + Sync {
    async fn send(
        &self,
        url: &str,
        payload: &serde_json::Value,
        timeout_secs: u64,
    ) -> Result<(), String>;
}

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn notify_driver(&self, driver_id: uuid::Uuid, message: &str) -> DomainResult<()>;
}
