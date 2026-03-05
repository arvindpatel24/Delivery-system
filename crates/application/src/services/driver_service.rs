use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use delivery_domain::entities::Driver;
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::DriverRepository;

use crate::dto::{RegisterDriverRequest, RegisterDriverResponse, LoginDriverRequest, LoginDriverResponse};

pub struct DriverService {
    pub driver_repo: Arc<dyn DriverRepository>,
}

impl DriverService {
    pub async fn register(
        &self,
        id: Uuid,
        req: RegisterDriverRequest,
        password_hash: String,
        token: String,
    ) -> DomainResult<RegisterDriverResponse> {
        if let Some(_existing) = self.driver_repo.find_by_phone(&req.phone).await? {
            return Err(DomainError::Duplicate(format!(
                "Driver with phone {} already exists",
                req.phone
            )));
        }

        let now = Utc::now();
        let driver = Driver {
            id,
            name: req.name.clone(),
            phone: req.phone,
            password_hash,
            vehicle_type: req.vehicle_type.unwrap_or_else(|| "motorcycle".to_string()),
            current_location: None,
            is_available: false,
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        let created = self.driver_repo.create(&driver).await?;

        Ok(RegisterDriverResponse {
            id: created.id,
            name: created.name,
            token,
        })
    }

    pub async fn get_driver(&self, id: Uuid) -> DomainResult<Driver> {
        self.driver_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::NotFound {
                entity: "Driver",
                id,
            })
    }

    pub async fn set_availability(&self, driver_id: Uuid, available: bool) -> DomainResult<()> {
        self.driver_repo.set_availability(driver_id, available).await
    }
}
