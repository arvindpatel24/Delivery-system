use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::{LocationEntry, LocationRepository};
use delivery_domain::value_objects::Location;

pub struct PgLocationRepo {
    pub pool: PgPool,
}

#[async_trait]
impl LocationRepository for PgLocationRepo {
    async fn insert(
        &self,
        driver_id: Uuid,
        location: Location,
        accuracy: Option<f64>,
        speed: Option<f64>,
        heading: Option<f64>,
        is_offline_sync: bool,
        recorded_at: DateTime<Utc>,
    ) -> DomainResult<()> {
        sqlx::query(
            r#"
            INSERT INTO driver_locations (driver_id, location, accuracy_meters, speed_kmh, heading, is_offline_sync, recorded_at)
            VALUES ($1, ST_SetSRID(ST_MakePoint($2, $3), 4326), $4, $5, $6, $7, $8)
            "#,
        )
        .bind(driver_id)
        .bind(location.longitude)
        .bind(location.latitude)
        .bind(accuracy)
        .bind(speed)
        .bind(heading)
        .bind(is_offline_sync)
        .bind(recorded_at)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn bulk_insert(&self, entries: Vec<LocationEntry>) -> DomainResult<u64> {
        if entries.is_empty() {
            return Ok(0);
        }

        let len = entries.len();
        let mut driver_ids = Vec::with_capacity(len);
        let mut lngs = Vec::with_capacity(len);
        let mut lats = Vec::with_capacity(len);
        let mut accuracies = Vec::with_capacity(len);
        let mut speeds = Vec::with_capacity(len);
        let mut headings = Vec::with_capacity(len);
        let mut offline_flags = Vec::with_capacity(len);
        let mut recorded_ats = Vec::with_capacity(len);

        for e in &entries {
            driver_ids.push(e.driver_id);
            lngs.push(e.location.longitude);
            lats.push(e.location.latitude);
            accuracies.push(e.accuracy_meters);
            speeds.push(e.speed_kmh);
            headings.push(e.heading);
            offline_flags.push(e.is_offline_sync);
            recorded_ats.push(e.recorded_at);
        }

        let result = sqlx::query(
            r#"
            INSERT INTO driver_locations (driver_id, location, accuracy_meters, speed_kmh, heading, is_offline_sync, recorded_at)
            SELECT
                unnest($1::uuid[]),
                ST_SetSRID(ST_MakePoint(unnest($2::float8[]), unnest($3::float8[])), 4326),
                unnest($4::float8[]),
                unnest($5::float8[]),
                unnest($6::float8[]),
                unnest($7::bool[]),
                unnest($8::timestamptz[])
            "#,
        )
        .bind(&driver_ids)
        .bind(&lngs)
        .bind(&lats)
        .bind(&accuracies as &[Option<f64>])
        .bind(&speeds as &[Option<f64>])
        .bind(&headings as &[Option<f64>])
        .bind(&offline_flags)
        .bind(&recorded_ats)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(result.rows_affected())
    }

    async fn cleanup_older_than(&self, cutoff: DateTime<Utc>) -> DomainResult<u64> {
        let result = sqlx::query("DELETE FROM driver_locations WHERE created_at < $1")
            .bind(cutoff)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
