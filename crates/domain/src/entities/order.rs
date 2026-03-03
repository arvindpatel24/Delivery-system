use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::{DomainError, DomainResult};
use crate::value_objects::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Dispatching,
    Assigned,
    PickedUp,
    InTransit,
    Delivered,
    Cancelled,
    Failed,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Dispatching => "dispatching",
            Self::Assigned => "assigned",
            Self::PickedUp => "picked_up",
            Self::InTransit => "in_transit",
            Self::Delivered => "delivered",
            Self::Cancelled => "cancelled",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> DomainResult<Self> {
        match s {
            "pending" => Ok(Self::Pending),
            "dispatching" => Ok(Self::Dispatching),
            "assigned" => Ok(Self::Assigned),
            "picked_up" => Ok(Self::PickedUp),
            "in_transit" => Ok(Self::InTransit),
            "delivered" => Ok(Self::Delivered),
            "cancelled" => Ok(Self::Cancelled),
            "failed" => Ok(Self::Failed),
            other => Err(DomainError::Validation(format!("Invalid order status: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingMode {
    Instant,
    Batched,
}

impl RoutingMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Instant => "instant",
            Self::Batched => "batched",
        }
    }

    pub fn from_str(s: &str) -> DomainResult<Self> {
        match s {
            "instant" => Ok(Self::Instant),
            "batched" => Ok(Self::Batched),
            other => Err(DomainError::Validation(format!("Invalid routing mode: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: Uuid,
    pub shop_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub status: OrderStatus,
    pub routing_mode: RoutingMode,
    pub pickup_address: String,
    pub pickup_location: Location,
    pub dropoff_address: String,
    pub dropoff_location: Location,
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

impl Order {
    pub fn can_transition_to(&self, new_status: OrderStatus) -> bool {
        use OrderStatus::*;
        matches!(
            (self.status, new_status),
            (Pending, Dispatching)
                | (Pending, Cancelled)
                | (Dispatching, Assigned)
                | (Dispatching, Pending) // re-dispatch
                | (Dispatching, Failed)
                | (Assigned, PickedUp)
                | (Assigned, Cancelled)
                | (PickedUp, InTransit)
                | (InTransit, Delivered)
        )
    }

    pub fn transition_to(&mut self, new_status: OrderStatus) -> DomainResult<()> {
        if !self.can_transition_to(new_status) {
            return Err(DomainError::InvalidStateTransition(format!(
                "Cannot transition from {:?} to {:?}",
                self.status, new_status
            )));
        }
        self.status = new_status;
        self.updated_at = Utc::now();
        Ok(())
    }
}

/// Distance threshold for instant vs batched routing (2km).
pub const INSTANT_DISTANCE_THRESHOLD_METERS: f64 = 2000.0;
