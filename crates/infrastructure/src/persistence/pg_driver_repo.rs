use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use delivery_domain::entities::Driver;
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::DriverRepository;
use delivery_domain::value_objects::Location;

use super::models::DriverRow;

pub struct PgDriverRepo {
    pub pool: PgPool,
}

const DRIVER_SELECT: &str = r#"
    id, name, phone, password_hash, vehicle_type,
    ST_Y(current_location) as current_lat, ST_X(current_location) as current_lng,
    is_available, is_active, created_at, updated_at
"#;

#[async_trait]
impl DriverRepository for PgDriverRepo {
    async fn create(&self, driver: &Driver) -> DomainResult<Driver> {
        let row = sqlx::query_as::<_, DriverRow>(&format!(
            r#"
            INSERT INTO drivers (id, name, phone, password_hash, vehicle_type, is_available, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING {DRIVER_SELECT}
            "#
        ))
        .bind(driver.id)
        .bind(&driver.name)
        .bind(&driver.phone)
        .bind(&driver.password_hash)
        .bind(&driver.vehicle_type)
        .bind(driver.is_available)
        .bind(driver.is_active)
        .bind(driver.created_at)
        .bind(driver.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.into())
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Driver>> {
        let row = sqlx::query_as::<_, DriverRow>(&format!(
            "SELECT {DRIVER_SELECT} FROM drivers WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_by_phone(&self, phone: &str) -> DomainResult<Option<Driver>> {
        let row = sqlx::query_as::<_, DriverRow>(&format!(
            "SELECT {DRIVER_SELECT} FROM drivers WHERE phone = $1"
        ))
        .bind(phone)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn update(&self, driver: &Driver) -> DomainResult<Driver> {
        let row = sqlx::query_as::<_, DriverRow>(&format!(
            r#"
            UPDATE drivers SET name = $2, phone = $3, vehicle_type = $4, is_available = $5, is_active = $6, updated_at = NOW()
            WHERE id = $1
            RETURNING {DRIVER_SELECT}
            "#
        ))
        .bind(driver.id)
        .bind(&driver.name)
        .bind(&driver.phone)
        .bind(&driver.vehicle_type)
        .bind(driver.is_available)
        .bind(driver.is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.into())
    }

    async fn update_location(&self, driver_id: Uuid, location: Location) -> DomainResult<()> {
        sqlx::query(
            r#"
            UPDATE drivers SET current_location = ST_SetSRID(ST_MakePoint($2, $3), 4326), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(driver_id)
        .bind(location.longitude)
        .bind(location.latitude)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn set_availability(&self, driver_id: Uuid, available: bool) -> DomainResult<()> {
        sqlx::query("UPDATE drivers SET is_available = $2, updated_at = NOW() WHERE id = $1")
            .bind(driver_id)
            .bind(available)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }
}
