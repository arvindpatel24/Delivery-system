use std::sync::Arc;

use chrono::{Duration, Utc};
use uuid::Uuid;

use delivery_domain::entities::dispatch_offer::OfferStatus;
use delivery_domain::entities::order::OrderStatus;
use delivery_domain::entities::DispatchOffer;
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::{
    DispatchOfferRepository, DriverRepository, GeospatialEngine, OrderRepository,
};
use delivery_domain::value_objects::Location;

pub struct DispatchService {
    pub order_repo: Arc<dyn OrderRepository>,
    pub driver_repo: Arc<dyn DriverRepository>,
    pub offer_repo: Arc<dyn DispatchOfferRepository>,
    pub geospatial: Arc<dyn GeospatialEngine>,
    pub dispatch_timeout_secs: i64,
    pub dispatch_radius_meters: f64,
}

impl DispatchService {
    pub async fn dispatch_instant_order(&self, order_id: Uuid) -> DomainResult<()> {
        let order = self
            .order_repo
            .find_by_id(order_id)
            .await?
            .ok_or(DomainError::NotFound {
                entity: "Order",
                id: order_id,
            })?;

        self.order_repo
            .update_status(order_id, OrderStatus::Dispatching, None)
            .await?;

        let nearby = self
            .geospatial
            .find_nearby_drivers(order.pickup_location, self.dispatch_radius_meters, 5)
            .await?;

        if nearby.is_empty() {
            tracing::warn!(order_id = %order_id, "No drivers nearby for instant dispatch");
            return Err(DomainError::NoDriversAvailable);
        }

        let expires_at = Utc::now() + Duration::seconds(self.dispatch_timeout_secs);

        for driver in &nearby {
            let offer = DispatchOffer {
                id: Uuid::now_v7(),
                order_id,
                driver_id: driver.driver_id,
                status: OfferStatus::Pending,
                distance_to_pickup_meters: driver.distance_meters,
                expires_at,
                responded_at: None,
                created_at: Utc::now(),
            };
            self.offer_repo.create(&offer).await?;
        }

        tracing::info!(
            order_id = %order_id,
            driver_count = nearby.len(),
            "Created dispatch offers for instant order"
        );

        Ok(())
    }

    pub async fn accept_offer(&self, offer_id: Uuid, driver_id: Uuid) -> DomainResult<()> {
        let offer = self
            .offer_repo
            .find_by_id(offer_id)
            .await?
            .ok_or(DomainError::NotFound {
                entity: "DispatchOffer",
                id: offer_id,
            })?;

        if offer.driver_id != driver_id {
            return Err(DomainError::AuthorizationDenied(
                "This offer is not for you".to_string(),
            ));
        }

        if offer.status != OfferStatus::Pending {
            return Err(DomainError::InvalidStateTransition(
                "Offer is no longer pending".to_string(),
            ));
        }

        if offer.expires_at < Utc::now() {
            return Err(DomainError::OfferExpired);
        }

        self.offer_repo
            .update_status(offer_id, OfferStatus::Accepted)
            .await?;

        self.order_repo
            .update_status(offer.order_id, OrderStatus::Assigned, Some(driver_id))
            .await?;

        // Expire remaining offers for this order
        let pending = self
            .offer_repo
            .find_pending_for_order(offer.order_id)
            .await?;
        for other in pending {
            if other.id != offer_id {
                self.offer_repo
                    .update_status(other.id, OfferStatus::Expired)
                    .await?;
            }
        }

        tracing::info!(
            order_id = %offer.order_id,
            driver_id = %driver_id,
            "Driver accepted dispatch offer"
        );

        Ok(())
    }

    pub async fn reject_offer(&self, offer_id: Uuid, driver_id: Uuid) -> DomainResult<()> {
        let offer = self
            .offer_repo
            .find_by_id(offer_id)
            .await?
            .ok_or(DomainError::NotFound {
                entity: "DispatchOffer",
                id: offer_id,
            })?;

        if offer.driver_id != driver_id {
            return Err(DomainError::AuthorizationDenied(
                "This offer is not for you".to_string(),
            ));
        }

        self.offer_repo
            .update_status(offer_id, OfferStatus::Rejected)
            .await?;

        Ok(())
    }

    pub async fn handle_stale_offers(&self) -> DomainResult<Vec<Uuid>> {
        let expired_order_ids = self.offer_repo.expire_stale_offers(Utc::now()).await?;

        for order_id in &expired_order_ids {
            tracing::info!(order_id = %order_id, "Re-dispatching order with expired offers");
            // Try widened radius or fail
            let order = self.order_repo.find_by_id(*order_id).await?;
            if let Some(order) = order {
                let wider_radius = self.dispatch_radius_meters * 1.5;
                let nearby = self
                    .geospatial
                    .find_nearby_drivers(order.pickup_location, wider_radius, 5)
                    .await?;

                if nearby.is_empty() {
                    self.order_repo
                        .update_status(*order_id, OrderStatus::Failed, None)
                        .await?;
                } else {
                    let expires_at = Utc::now() + Duration::seconds(self.dispatch_timeout_secs);
                    for driver in &nearby {
                        let offer = DispatchOffer {
                            id: Uuid::now_v7(),
                            order_id: *order_id,
                            driver_id: driver.driver_id,
                            status: OfferStatus::Pending,
                            distance_to_pickup_meters: driver.distance_meters,
                            expires_at,
                            responded_at: None,
                            created_at: Utc::now(),
                        };
                        self.offer_repo.create(&offer).await?;
                    }
                }
            }
        }

        Ok(expired_order_ids)
    }
}
