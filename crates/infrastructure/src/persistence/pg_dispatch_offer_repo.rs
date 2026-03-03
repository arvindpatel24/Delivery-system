use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use delivery_domain::entities::DispatchOffer;
use delivery_domain::entities::dispatch_offer::OfferStatus;
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::DispatchOfferRepository;

use super::models::DispatchOfferRow;

pub struct PgDispatchOfferRepo {
    pub pool: PgPool,
}

#[async_trait]
impl DispatchOfferRepository for PgDispatchOfferRepo {
    async fn create(&self, offer: &DispatchOffer) -> DomainResult<DispatchOffer> {
        let row = sqlx::query_as::<_, DispatchOfferRow>(
            r#"
            INSERT INTO driver_dispatch_offers (id, order_id, driver_id, status, distance_to_pickup_meters, expires_at, created_at)
            VALUES ($1, $2, $3, $4::offer_status, $5, $6, $7)
            RETURNING id, order_id, driver_id, status::text, distance_to_pickup_meters, expires_at, responded_at, created_at
            "#,
        )
        .bind(offer.id)
        .bind(offer.order_id)
        .bind(offer.driver_id)
        .bind(offer.status.as_str())
        .bind(offer.distance_to_pickup_meters)
        .bind(offer.expires_at)
        .bind(offer.created_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.into())
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<DispatchOffer>> {
        let row = sqlx::query_as::<_, DispatchOfferRow>(
            r#"
            SELECT id, order_id, driver_id, status::text, distance_to_pickup_meters, expires_at, responded_at, created_at
            FROM driver_dispatch_offers WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    async fn find_pending_for_order(&self, order_id: Uuid) -> DomainResult<Vec<DispatchOffer>> {
        let rows = sqlx::query_as::<_, DispatchOfferRow>(
            r#"
            SELECT id, order_id, driver_id, status::text, distance_to_pickup_meters, expires_at, responded_at, created_at
            FROM driver_dispatch_offers WHERE order_id = $1 AND status = 'pending'
            "#,
        )
        .bind(order_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_pending_for_driver(&self, driver_id: Uuid) -> DomainResult<Vec<DispatchOffer>> {
        let rows = sqlx::query_as::<_, DispatchOfferRow>(
            r#"
            SELECT id, order_id, driver_id, status::text, distance_to_pickup_meters, expires_at, responded_at, created_at
            FROM driver_dispatch_offers WHERE driver_id = $1 AND status = 'pending'
            "#,
        )
        .bind(driver_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update_status(&self, id: Uuid, status: OfferStatus) -> DomainResult<()> {
        sqlx::query(
            "UPDATE driver_dispatch_offers SET status = $2::offer_status, responded_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .bind(status.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn expire_stale_offers(&self, before: DateTime<Utc>) -> DomainResult<Vec<Uuid>> {
        let rows = sqlx::query_as::<_, (Uuid,)>(
            r#"
            UPDATE driver_dispatch_offers
            SET status = 'expired'::offer_status, responded_at = NOW()
            WHERE status = 'pending' AND expires_at < $1
            RETURNING DISTINCT order_id
            "#,
        )
        .bind(before)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }
}
