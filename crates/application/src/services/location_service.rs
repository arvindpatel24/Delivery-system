use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use delivery_domain::errors::DomainResult;
use delivery_domain::ports::{DriverRepository, LocationRepository, LocationEntry};
use delivery_domain::value_objects::Location;

use crate::dto::{LocationPingRequest, BulkLocationRequest};

pub struct LocationService {
    pub location_repo: Arc<dyn LocationRepository>,
    pub driver_repo: Arc<dyn DriverRepository>,
}

impl LocationService {
    pub async fn record_ping(
        &self,
        driver_id: Uuid,
        req: LocationPingRequest,
    ) -> DomainResult<()> {
        let location = Location::new(req.latitude, req.longitude)?;
        let recorded_at = req.recorded_at.unwrap_or_else(Utc::now);

        self.location_repo
            .insert(
                driver_id,
                location,
                req.accuracy_meters,
                req.speed_kmh,
                req.heading,
                false,
                recorded_at,
            )
            .await?;

        self.driver_repo
            .update_location(driver_id, location)
            .await?;

        Ok(())
    }

    pub async fn record_bulk(
        &self,
        driver_id: Uuid,
        req: BulkLocationRequest,
    ) -> DomainResult<u64> {
        // Find the latest ping before consuming the vec
        let latest_ping = req
            .pings
            .iter()
            .max_by_key(|p| p.recorded_at)
            .map(|p| (p.latitude, p.longitude));

        let entries: Vec<LocationEntry> = req
            .pings
            .into_iter()
            .filter_map(|p| {
                let loc = Location::new(p.latitude, p.longitude).ok()?;
                Some(LocationEntry {
                    driver_id,
                    location: loc,
                    accuracy_meters: p.accuracy_meters,
                    speed_kmh: p.speed_kmh,
                    heading: p.heading,
                    is_offline_sync: p.is_offline_sync,
                    recorded_at: p.recorded_at,
                })
            })
            .collect();

        let count = self.location_repo.bulk_insert(entries).await?;

        // Update driver's current location to the most recent ping
        if let Some((lat, lng)) = latest_ping {
            if let Ok(loc) = Location::new(lat, lng) {
                self.driver_repo.update_location(driver_id, loc).await?;
            }
        }

        Ok(count)
    }

    pub async fn cleanup_old_locations(&self) -> DomainResult<u64> {
        let cutoff = Utc::now() - Duration::hours(48);
        self.location_repo.cleanup_older_than(cutoff).await
    }
}
