use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use delivery_domain::entities::Shop;
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::{Geocoder, ShopRepository};
use delivery_domain::value_objects::Location;

use crate::dto::{CreateShopRequest, CreateShopResponse};

pub struct ShopService {
    pub shop_repo: Arc<dyn ShopRepository>,
    pub geocoder: Arc<dyn Geocoder>,
}

impl ShopService {
    pub async fn create_shop(
        &self,
        req: CreateShopRequest,
        api_key_hash: String,
        raw_api_key: String,
    ) -> DomainResult<CreateShopResponse> {
        if let Some(_existing) = self.shop_repo.find_by_phone(&req.phone).await? {
            return Err(DomainError::Duplicate(format!(
                "Shop with phone {} already exists",
                req.phone
            )));
        }

        let location = Location::new(req.latitude, req.longitude)?;

        let now = Utc::now();
        let shop = Shop {
            id: Uuid::now_v7(),
            name: req.name.clone(),
            phone: req.phone,
            api_key_hash,
            location,
            address: req.address,
            webhook_url: req.webhook_url,
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        let created = self.shop_repo.create(&shop).await?;

        Ok(CreateShopResponse {
            id: created.id,
            name: created.name,
            api_key: raw_api_key,
        })
    }

    pub async fn get_shop(&self, id: Uuid) -> DomainResult<Shop> {
        self.shop_repo
            .find_by_id(id)
            .await?
            .ok_or(DomainError::NotFound {
                entity: "Shop",
                id,
            })
    }
}
