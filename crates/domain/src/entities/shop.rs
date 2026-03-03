use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::value_objects::Location;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shop {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub api_key_hash: String,
    pub location: Location,
    pub address: String,
    pub webhook_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
