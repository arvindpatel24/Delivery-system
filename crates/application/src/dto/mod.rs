use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Shop DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateShopRequest {
    pub name: String,
    pub phone: String,
    pub address: String,
    pub webhook_url: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Serialize)]
pub struct CreateShopResponse {
    pub id: Uuid,
    pub name: String,
    pub api_key: String,
}

// --- Driver DTOs ---

#[derive(Debug, Deserialize)]
pub struct RegisterDriverRequest {
    pub name: String,
    pub phone: String,
    pub password: String,
    pub vehicle_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterDriverResponse {
    pub id: Uuid,
    pub name: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginDriverRequest {
    pub phone: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginDriverResponse {
    pub token: String,
    pub driver_id: Uuid,
    pub name: String,
}

// --- Order DTOs ---

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub pickup_address: String,
    pub dropoff_address: String,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub package_description: Option<String>,
    pub pickup_latitude: Option<f64>,
    pub pickup_longitude: Option<f64>,
    pub dropoff_latitude: Option<f64>,
    pub dropoff_longitude: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    pub id: Uuid,
    pub status: String,
    pub routing_mode: String,
    pub distance_meters: f64,
    pub estimated_delivery_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub shop_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub status: String,
    pub routing_mode: String,
    pub pickup_address: String,
    pub dropoff_address: String,
    pub distance_meters: f64,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub package_description: Option<String>,
    pub estimated_delivery_at: Option<DateTime<Utc>>,
    pub picked_up_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<delivery_domain::entities::Order> for OrderResponse {
    fn from(o: delivery_domain::entities::Order) -> Self {
        Self {
            id: o.id,
            shop_id: o.shop_id,
            driver_id: o.driver_id,
            status: o.status.as_str().to_string(),
            routing_mode: o.routing_mode.as_str().to_string(),
            pickup_address: o.pickup_address,
            dropoff_address: o.dropoff_address,
            distance_meters: o.distance_meters,
            customer_name: o.customer_name,
            customer_phone: o.customer_phone,
            package_description: o.package_description,
            estimated_delivery_at: o.estimated_delivery_at,
            picked_up_at: o.picked_up_at,
            delivered_at: o.delivered_at,
            created_at: o.created_at,
        }
    }
}

// --- Location DTOs ---

#[derive(Debug, Deserialize)]
pub struct LocationPingRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub speed_kmh: Option<f64>,
    pub heading: Option<f64>,
    pub recorded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct BulkLocationRequest {
    pub pings: Vec<LocationPingEntry>,
}

#[derive(Debug, Deserialize)]
pub struct LocationPingEntry {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub speed_kmh: Option<f64>,
    pub heading: Option<f64>,
    pub recorded_at: DateTime<Utc>,
    #[serde(default)]
    pub is_offline_sync: bool,
}

// --- Dispatch DTOs ---

#[derive(Debug, Deserialize)]
pub struct DispatchOfferResponse {
    pub offer_id: Uuid,
    pub order_id: Uuid,
    pub pickup_address: String,
    pub dropoff_address: String,
    pub distance_to_pickup_meters: f64,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct RespondToOfferRequest {
    pub accept: bool,
}
