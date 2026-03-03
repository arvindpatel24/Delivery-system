use std::sync::Arc;

use delivery_application::services::{
    batch_service::BatchService,
    dispatch_service::DispatchService,
    driver_service::DriverService,
    location_service::LocationService,
    order_service::OrderService,
    shop_service::ShopService,
    webhook_service::WebhookService,
};
use delivery_infrastructure::auth::JwtManager;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub shop_service: Arc<ShopService>,
    pub driver_service: Arc<DriverService>,
    pub order_service: Arc<OrderService>,
    pub dispatch_service: Arc<DispatchService>,
    pub location_service: Arc<LocationService>,
    pub webhook_service: Arc<WebhookService>,
    pub batch_service: Arc<BatchService>,
    pub jwt_manager: Arc<JwtManager>,
    pub db_pool: sqlx::PgPool,
}
