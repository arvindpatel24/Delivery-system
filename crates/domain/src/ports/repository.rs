use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::*;
use crate::entities::dispatch_offer::OfferStatus;
use crate::entities::order::{OrderStatus, RoutingMode};
use crate::entities::webhook::WebhookStatus;
use crate::errors::DomainResult;
use crate::value_objects::Location;

#[async_trait]
pub trait ShopRepository: Send + Sync {
    async fn create(&self, shop: &Shop) -> DomainResult<Shop>;
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Shop>>;
    async fn find_by_api_key_hash(&self, hash: &str) -> DomainResult<Option<Shop>>;
    async fn find_by_phone(&self, phone: &str) -> DomainResult<Option<Shop>>;
    async fn update(&self, shop: &Shop) -> DomainResult<Shop>;
}

#[async_trait]
pub trait DriverRepository: Send + Sync {
    async fn create(&self, driver: &Driver) -> DomainResult<Driver>;
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Driver>>;
    async fn find_by_phone(&self, phone: &str) -> DomainResult<Option<Driver>>;
    async fn update(&self, driver: &Driver) -> DomainResult<Driver>;
    async fn update_location(&self, driver_id: Uuid, location: Location) -> DomainResult<()>;
    async fn set_availability(&self, driver_id: Uuid, available: bool) -> DomainResult<()>;
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create(&self, order: &Order) -> DomainResult<Order>;
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Order>>;
    async fn find_by_shop(&self, shop_id: Uuid, limit: i64, offset: i64) -> DomainResult<Vec<Order>>;
    async fn find_by_driver(&self, driver_id: Uuid, status: Option<OrderStatus>) -> DomainResult<Vec<Order>>;
    async fn update_status(&self, id: Uuid, status: OrderStatus, driver_id: Option<Uuid>) -> DomainResult<()>;
    async fn find_pending_by_routing_mode(&self, mode: RoutingMode) -> DomainResult<Vec<Order>>;
    async fn assign_to_batch_cluster(&self, order_id: Uuid, cluster_id: Uuid) -> DomainResult<()>;
}

#[async_trait]
pub trait LocationRepository: Send + Sync {
    async fn insert(&self, driver_id: Uuid, location: Location, accuracy: Option<f64>, speed: Option<f64>, heading: Option<f64>, is_offline_sync: bool, recorded_at: DateTime<Utc>) -> DomainResult<()>;
    async fn bulk_insert(&self, entries: Vec<LocationEntry>) -> DomainResult<u64>;
    async fn cleanup_older_than(&self, cutoff: DateTime<Utc>) -> DomainResult<u64>;
}

#[derive(Debug, Clone)]
pub struct LocationEntry {
    pub driver_id: Uuid,
    pub location: Location,
    pub accuracy_meters: Option<f64>,
    pub speed_kmh: Option<f64>,
    pub heading: Option<f64>,
    pub is_offline_sync: bool,
    pub recorded_at: DateTime<Utc>,
}

#[async_trait]
pub trait DispatchOfferRepository: Send + Sync {
    async fn create(&self, offer: &DispatchOffer) -> DomainResult<DispatchOffer>;
    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DispatchOffer>>;
    async fn find_pending_for_order(&self, order_id: Uuid) -> DomainResult<Vec<DispatchOffer>>;
    async fn find_pending_for_driver(&self, driver_id: Uuid) -> DomainResult<Vec<DispatchOffer>>;
    async fn update_status(&self, id: Uuid, status: OfferStatus) -> DomainResult<()>;
    async fn expire_stale_offers(&self, before: DateTime<Utc>) -> DomainResult<Vec<Uuid>>;
}

#[async_trait]
pub trait WebhookOutboxRepository: Send + Sync {
    async fn insert(&self, entry: &WebhookOutbox) -> DomainResult<()>;
    async fn fetch_pending(&self, limit: i64) -> DomainResult<Vec<WebhookOutbox>>;
    async fn mark_delivered(&self, id: Uuid) -> DomainResult<()>;
    async fn mark_failed(&self, id: Uuid, error: &str, next_retry_at: DateTime<Utc>) -> DomainResult<()>;
    async fn mark_dead(&self, id: Uuid) -> DomainResult<()>;
}

#[async_trait]
pub trait BatchRepository: Send + Sync {
    async fn create_run(&self, run: &BatchRun) -> DomainResult<BatchRun>;
    async fn complete_run(&self, id: Uuid, clusters: i32, drivers: i32) -> DomainResult<()>;
    async fn create_cluster(&self, cluster: &BatchCluster) -> DomainResult<BatchCluster>;
}
