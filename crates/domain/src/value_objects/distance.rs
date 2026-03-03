use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Distance {
    pub meters: f64,
}

impl Distance {
    pub fn from_meters(meters: f64) -> Self {
        Self { meters }
    }

    pub fn kilometers(&self) -> f64 {
        self.meters / 1000.0
    }

    pub fn is_instant_eligible(&self) -> bool {
        self.meters <= crate::entities::order::INSTANT_DISTANCE_THRESHOLD_METERS
    }
}
