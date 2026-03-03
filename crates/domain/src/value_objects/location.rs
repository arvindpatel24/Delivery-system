use serde::{Deserialize, Serialize};

use crate::errors::{DomainError, DomainResult};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    pub fn new(latitude: f64, longitude: f64) -> DomainResult<Self> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(DomainError::Validation(format!(
                "Latitude must be between -90 and 90, got {latitude}"
            )));
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(DomainError::Validation(format!(
                "Longitude must be between -180 and 180, got {longitude}"
            )));
        }
        Ok(Self { latitude, longitude })
    }
}
