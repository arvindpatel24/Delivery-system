use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use delivery_domain::entities::Shop;
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::ShopRepository;

use super::models::ShopRow;

pub struct PgShopRepo {
    pub pool: PgPool,
}

#[async_trait]
impl ShopRepository for PgShopRepo {
    async fn create(&self, shop: &Shop) -> DomainResult<Shop> {
        let row = sqlx::query_as::<_, ShopRow>(
            r#"
            INSERT INTO shops (id, name, phone, api_key_hash, location, address, webhook_url, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, ST_SetSRID(ST_MakePoint($5, $6), 4326), $7, $8, $9, $10, $11)
            RETURNING id, name, phone, api_key_hash, ST_Y(location) as latitude, ST_X(location) as longitude, address, webhook_url, is_active, created_at, updated_at
            "#,
        )
        .bind(shop.id)
        .bind(&shop.name)
        .bind(&shop.phone)
        .bind(&shop.api_key_hash)
        .bind(shop.location.longitude)
        .bind(shop.location.latitude)
        .bind(&shop.address)
        .bind(&shop.webhook_url)
        .bind(shop.is_active)
        .bind(shop.created_at)
        .bind(shop.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.into())
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Shop>> {
        let row = sqlx::query_as::<_, ShopRow>(
            r#"
            SELECT id, name, phone, api_key_hash, ST_Y(location) as latitude, ST_X(location) as longitude, address, webhook_url, is_active, created_at, updated_at
            FROM shops WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_by_api_key_hash(&self, hash: &str) -> DomainResult<Option<Shop>> {
        let row = sqlx::query_as::<_, ShopRow>(
            r#"
            SELECT id, name, phone, api_key_hash, ST_Y(location) as latitude, ST_X(location) as longitude, address, webhook_url, is_active, created_at, updated_at
            FROM shops WHERE api_key_hash = $1 AND is_active = TRUE
            "#,
        )
        .bind(hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_by_phone(&self, phone: &str) -> DomainResult<Option<Shop>> {
        let row = sqlx::query_as::<_, ShopRow>(
            r#"
            SELECT id, name, phone, api_key_hash, ST_Y(location) as latitude, ST_X(location) as longitude, address, webhook_url, is_active, created_at, updated_at
            FROM shops WHERE phone = $1
            "#,
        )
        .bind(phone)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn update(&self, shop: &Shop) -> DomainResult<Shop> {
        let row = sqlx::query_as::<_, ShopRow>(
            r#"
            UPDATE shops SET name = $2, phone = $3, location = ST_SetSRID(ST_MakePoint($4, $5), 4326),
                address = $6, webhook_url = $7, is_active = $8, updated_at = NOW()
            WHERE id = $1
            RETURNING id, name, phone, api_key_hash, ST_Y(location) as latitude, ST_X(location) as longitude, address, webhook_url, is_active, created_at, updated_at
            "#,
        )
        .bind(shop.id)
        .bind(&shop.name)
        .bind(&shop.phone)
        .bind(shop.location.longitude)
        .bind(shop.location.latitude)
        .bind(&shop.address)
        .bind(&shop.webhook_url)
        .bind(shop.is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.into())
    }
}
