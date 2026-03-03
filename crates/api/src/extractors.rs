use axum::extract::Extension;
use uuid::Uuid;

use crate::middleware::auth::{ShopIdentity, DriverIdentity};

pub type ShopId = Extension<ShopIdentity>;
pub type DriverId = Extension<DriverIdentity>;

pub fn extract_shop_id(ext: &ShopIdentity) -> Uuid {
    ext.shop_id
}

pub fn extract_driver_id(ext: &DriverIdentity) -> Uuid {
    ext.driver_id
}
