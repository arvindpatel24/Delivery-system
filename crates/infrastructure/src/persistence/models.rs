use chrono::{DateTime, Utc};
use uuid::Uuid;

use delivery_domain::entities::*;
use delivery_domain::entities::order::{OrderStatus, RoutingMode};
use delivery_domain::entities::webhook::WebhookStatus;
use delivery_domain::entities::dispatch_offer::OfferStatus;
use delivery_domain::value_objects::Location;

// Row types for sqlx queries. We read lat/lng from PostGIS via ST_Y/ST_X.

#[derive(Debug, sqlx::FromRow)]
pub struct ShopRow {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub api_key_hash: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address: String,
    pub webhook_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ShopRow> for Shop {
    fn from(r: ShopRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            phone: r.phone,
            api_key_hash: r.api_key_hash,
            location: Location {
                latitude: r.latitude,
                longitude: r.longitude,
            },
            address: r.address,
            webhook_url: r.webhook_url,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DriverRow {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub password_hash: String,
    pub vehicle_type: String,
    pub current_lat: Option<f64>,
    pub current_lng: Option<f64>,
    pub is_available: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DriverRow> for Driver {
    fn from(r: DriverRow) -> Self {
        let current_location = match (r.current_lat, r.current_lng) {
            (Some(lat), Some(lng)) => Some(Location {
                latitude: lat,
                longitude: lng,
            }),
            _ => None,
        };
        Self {
            id: r.id,
            name: r.name,
            phone: r.phone,
            password_hash: r.password_hash,
            vehicle_type: r.vehicle_type,
            current_location,
            is_available: r.is_available,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct OrderRow {
    pub id: Uuid,
    pub shop_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub status: String,
    pub routing_mode: String,
    pub pickup_address: String,
    pub pickup_lat: f64,
    pub pickup_lng: f64,
    pub dropoff_address: String,
    pub dropoff_lat: f64,
    pub dropoff_lng: f64,
    pub distance_meters: f64,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub package_description: Option<String>,
    pub estimated_delivery_at: Option<DateTime<Utc>>,
    pub picked_up_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub batch_cluster_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<OrderRow> for Order {
    type Error = delivery_domain::errors::DomainError;

    fn try_from(r: OrderRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: r.id,
            shop_id: r.shop_id,
            driver_id: r.driver_id,
            status: OrderStatus::from_str(&r.status)?,
            routing_mode: RoutingMode::from_str(&r.routing_mode)?,
            pickup_address: r.pickup_address,
            pickup_location: Location {
                latitude: r.pickup_lat,
                longitude: r.pickup_lng,
            },
            dropoff_address: r.dropoff_address,
            dropoff_location: Location {
                latitude: r.dropoff_lat,
                longitude: r.dropoff_lng,
            },
            distance_meters: r.distance_meters,
            customer_name: r.customer_name,
            customer_phone: r.customer_phone,
            package_description: r.package_description,
            estimated_delivery_at: r.estimated_delivery_at,
            picked_up_at: r.picked_up_at,
            delivered_at: r.delivered_at,
            batch_cluster_id: r.batch_cluster_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DispatchOfferRow {
    pub id: Uuid,
    pub order_id: Uuid,
    pub driver_id: Uuid,
    pub status: String,
    pub distance_to_pickup_meters: f64,
    pub expires_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<DispatchOfferRow> for DispatchOffer {
    fn from(r: DispatchOfferRow) -> Self {
        let status = match r.status.as_str() {
            "accepted" => OfferStatus::Accepted,
            "rejected" => OfferStatus::Rejected,
            "expired" => OfferStatus::Expired,
            _ => OfferStatus::Pending,
        };
        Self {
            id: r.id,
            order_id: r.order_id,
            driver_id: r.driver_id,
            status,
            distance_to_pickup_meters: r.distance_to_pickup_meters,
            expires_at: r.expires_at,
            responded_at: r.responded_at,
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct WebhookOutboxRow {
    pub id: Uuid,
    pub order_id: Uuid,
    pub shop_id: Uuid,
    pub webhook_url: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub next_retry_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

impl From<WebhookOutboxRow> for WebhookOutbox {
    fn from(r: WebhookOutboxRow) -> Self {
        let status = match r.status.as_str() {
            "delivered" => WebhookStatus::Delivered,
            "failed" => WebhookStatus::Failed,
            "dead" => WebhookStatus::Dead,
            _ => WebhookStatus::Pending,
        };
        Self {
            id: r.id,
            order_id: r.order_id,
            shop_id: r.shop_id,
            webhook_url: r.webhook_url,
            payload: r.payload,
            status,
            attempts: r.attempts,
            last_error: r.last_error,
            next_retry_at: r.next_retry_at,
            created_at: r.created_at,
            delivered_at: r.delivered_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct BatchRunRow {
    pub id: Uuid,
    pub scheduled_hour: i32,
    pub total_orders: i32,
    pub total_clusters: i32,
    pub total_drivers_assigned: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<BatchRunRow> for delivery_domain::entities::BatchRun {
    fn from(r: BatchRunRow) -> Self {
        Self {
            id: r.id,
            scheduled_hour: r.scheduled_hour,
            total_orders: r.total_orders,
            total_clusters: r.total_clusters,
            total_drivers_assigned: r.total_drivers_assigned,
            started_at: r.started_at,
            completed_at: r.completed_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct BatchClusterRow {
    pub id: Uuid,
    pub batch_run_id: Uuid,
    pub cluster_label: i32,
    pub driver_id: Option<Uuid>,
    pub centroid_lat: Option<f64>,
    pub centroid_lng: Option<f64>,
    pub order_count: i32,
    pub total_distance_meters: Option<f64>,
    pub created_at: DateTime<Utc>,
}

impl From<BatchClusterRow> for delivery_domain::entities::BatchCluster {
    fn from(r: BatchClusterRow) -> Self {
        let centroid = match (r.centroid_lat, r.centroid_lng) {
            (Some(lat), Some(lng)) => Some(Location {
                latitude: lat,
                longitude: lng,
            }),
            _ => None,
        };
        Self {
            id: r.id,
            batch_run_id: r.batch_run_id,
            cluster_label: r.cluster_label,
            driver_id: r.driver_id,
            centroid,
            order_count: r.order_count,
            total_distance_meters: r.total_distance_meters,
            created_at: r.created_at,
        }
    }
}
