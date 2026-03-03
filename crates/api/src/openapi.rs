use utoipa::OpenApi;

/// Root OpenAPI document for the Samagri Delivery System API.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Samagri Delivery System",
        version = "0.1.0",
        description = "B2B2C delivery logistics API for small towns and rural areas.
Supports instant dispatch (≤ 2 km) and batch scheduling (> 2 km).

## Authentication
- **Shops** — pass `X-Api-Key: <key>` header (key returned at registration)
- **Drivers** — pass `Authorization: Bearer <jwt>` header (token returned at login)
"
    ),
    tags(
        (name = "public", description = "No authentication required"),
        (name = "shop", description = "Shop-authenticated endpoints (X-Api-Key)"),
        (name = "driver", description = "Driver-authenticated endpoints (Bearer JWT)")
    ),
    paths(
        crate::handlers::health::health_check,
        crate::handlers::shop::register_shop,
        crate::handlers::driver::register_driver,
        crate::handlers::driver::login_driver,
        crate::handlers::driver::set_availability,
        crate::handlers::order::create_order,
        crate::handlers::order::get_order,
        crate::handlers::order::list_shop_orders,
        crate::handlers::order::list_driver_orders,
        crate::handlers::order::update_order_status,
        crate::handlers::location::record_location,
        crate::handlers::location::record_bulk_location,
        crate::handlers::dispatch::get_pending_offers,
        crate::handlers::dispatch::respond_to_offer,
    ),
    components(
        schemas(
            ApiHealthResponse,
            RegisterShopBody,
            RegisterShopResponse,
            RegisterDriverBody,
            RegisterDriverResponse,
            LoginDriverBody,
            LoginDriverResponse,
            AvailabilityBody,
            CreateOrderBody,
            CreateOrderResponse,
            OrderResponse,
            LocationPingBody,
            BulkLocationBody,
            BulkLocationPingEntry,
            RespondOfferBody,
        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};

        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "shop_api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Api-Key"))),
        );
        components.add_security_scheme(
            "driver_jwt",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

// ── Schema types (mirrors the DTOs but with utoipa derives) ──────────────────

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiHealthResponse {
    pub status: String,
    pub database: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterShopBody {
    pub name: String,
    pub phone: String,
    pub address: String,
    pub webhook_url: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterShopResponse {
    pub id: String,
    pub name: String,
    /// Raw API key — store it securely, shown only once.
    pub api_key: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterDriverBody {
    pub name: String,
    pub phone: String,
    pub password: String,
    pub vehicle_type: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct RegisterDriverResponse {
    pub id: String,
    pub name: String,
    pub token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginDriverBody {
    pub phone: String,
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginDriverResponse {
    pub token: String,
    pub driver_id: String,
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct AvailabilityBody {
    pub available: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateOrderBody {
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

#[derive(Serialize, ToSchema)]
pub struct CreateOrderResponse {
    pub id: String,
    /// `instant` or `batched`
    pub routing_mode: String,
    pub status: String,
    pub distance_meters: f64,
    pub estimated_delivery_at: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct OrderResponse {
    pub id: String,
    pub shop_id: String,
    pub driver_id: Option<String>,
    pub status: String,
    pub routing_mode: String,
    pub pickup_address: String,
    pub dropoff_address: String,
    pub distance_meters: f64,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub package_description: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize, ToSchema)]
pub struct LocationPingBody {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub speed_kmh: Option<f64>,
    pub heading: Option<f64>,
    /// RFC 3339. Defaults to server time if omitted.
    pub recorded_at: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct BulkLocationBody {
    /// Up to 500 pings per request.
    pub pings: Vec<BulkLocationPingEntry>,
}

#[derive(Deserialize, ToSchema)]
pub struct BulkLocationPingEntry {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy_meters: Option<f64>,
    pub speed_kmh: Option<f64>,
    pub heading: Option<f64>,
    pub recorded_at: String,
    #[serde(default)]
    pub is_offline_sync: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct RespondOfferBody {
    pub accept: bool,
}
