use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::value_objects::Location;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRun {
    pub id: Uuid,
    pub scheduled_hour: i32,
    pub total_orders: i32,
    pub total_clusters: i32,
    pub total_drivers_assigned: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCluster {
    pub id: Uuid,
    pub batch_run_id: Uuid,
    pub cluster_label: i32,
    pub driver_id: Option<Uuid>,
    pub centroid: Option<Location>,
    pub order_count: i32,
    pub total_distance_meters: Option<f64>,
    pub created_at: DateTime<Utc>,
}
