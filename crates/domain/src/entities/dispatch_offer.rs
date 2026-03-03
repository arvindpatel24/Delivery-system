use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfferStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

impl OfferStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatchOffer {
    pub id: Uuid,
    pub order_id: Uuid,
    pub driver_id: Uuid,
    pub status: OfferStatus,
    pub distance_to_pickup_meters: f64,
    pub expires_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
