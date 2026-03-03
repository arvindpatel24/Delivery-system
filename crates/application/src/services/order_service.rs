use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use delivery_domain::entities::order::{Order, OrderStatus, RoutingMode, INSTANT_DISTANCE_THRESHOLD_METERS};
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::{GeospatialEngine, Geocoder, OrderRepository, ShopRepository};
use delivery_domain::value_objects::Location;

use crate::dto::{CreateOrderRequest, CreateOrderResponse, OrderResponse};

pub struct OrderService {
    pub order_repo: Arc<dyn OrderRepository>,
    pub shop_repo: Arc<dyn ShopRepository>,
    pub geospatial: Arc<dyn GeospatialEngine>,
    pub geocoder: Arc<dyn Geocoder>,
}

impl OrderService {
    pub async fn create_order(
        &self,
        shop_id: Uuid,
        req: CreateOrderRequest,
    ) -> DomainResult<CreateOrderResponse> {
        let shop = self
            .shop_repo
            .find_by_id(shop_id)
            .await?
            .ok_or(DomainError::NotFound {
                entity: "Shop",
                id: shop_id,
            })?;

        let pickup_location = match (req.pickup_latitude, req.pickup_longitude) {
            (Some(lat), Some(lng)) => Location::new(lat, lng)?,
            _ => shop.location,
        };

        let dropoff_location = match (req.dropoff_latitude, req.dropoff_longitude) {
            (Some(lat), Some(lng)) => Location::new(lat, lng)?,
            _ => self.geocoder.geocode(&req.dropoff_address).await?,
        };

        let distance = self
            .geospatial
            .compute_distance(pickup_location, dropoff_location)
            .await?;

        let routing_mode = if distance <= INSTANT_DISTANCE_THRESHOLD_METERS {
            RoutingMode::Instant
        } else {
            RoutingMode::Batched
        };

        let now = Utc::now();
        let order = Order {
            id: Uuid::new_v4(),
            shop_id,
            driver_id: None,
            status: OrderStatus::Pending,
            routing_mode,
            pickup_address: req.pickup_address,
            pickup_location,
            dropoff_address: req.dropoff_address,
            dropoff_location,
            distance_meters: distance,
            customer_name: req.customer_name,
            customer_phone: req.customer_phone,
            package_description: req.package_description,
            estimated_delivery_at: None,
            picked_up_at: None,
            delivered_at: None,
            batch_cluster_id: None,
            created_at: now,
            updated_at: now,
        };

        let created = self.order_repo.create(&order).await?;

        Ok(CreateOrderResponse {
            id: created.id,
            status: created.status.as_str().to_string(),
            routing_mode: created.routing_mode.as_str().to_string(),
            distance_meters: created.distance_meters,
            estimated_delivery_at: created.estimated_delivery_at,
        })
    }

    pub async fn get_order(&self, order_id: Uuid) -> DomainResult<OrderResponse> {
        let order = self
            .order_repo
            .find_by_id(order_id)
            .await?
            .ok_or(DomainError::NotFound {
                entity: "Order",
                id: order_id,
            })?;
        Ok(OrderResponse::from(order))
    }

    pub async fn list_shop_orders(
        &self,
        shop_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> DomainResult<Vec<OrderResponse>> {
        let orders = self.order_repo.find_by_shop(shop_id, limit, offset).await?;
        Ok(orders.into_iter().map(OrderResponse::from).collect())
    }

    pub async fn list_driver_orders(
        &self,
        driver_id: Uuid,
        status: Option<OrderStatus>,
    ) -> DomainResult<Vec<OrderResponse>> {
        let orders = self.order_repo.find_by_driver(driver_id, status).await?;
        Ok(orders.into_iter().map(OrderResponse::from).collect())
    }

    pub async fn update_order_status(
        &self,
        order_id: Uuid,
        new_status: OrderStatus,
    ) -> DomainResult<()> {
        let order = self
            .order_repo
            .find_by_id(order_id)
            .await?
            .ok_or(DomainError::NotFound {
                entity: "Order",
                id: order_id,
            })?;

        if !order.can_transition_to(new_status) {
            return Err(DomainError::InvalidStateTransition(format!(
                "Cannot transition order from {:?} to {:?}",
                order.status, new_status,
            )));
        }

        self.order_repo
            .update_status(order_id, new_status, order.driver_id)
            .await
    }
}
