use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::value_objects::Location;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Driver {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub password_hash: String,
    pub vehicle_type: String,
    pub current_location: Option<Location>,
    pub is_available: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
