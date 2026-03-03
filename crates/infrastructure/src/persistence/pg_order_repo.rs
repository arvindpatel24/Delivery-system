use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use delivery_domain::entities::Order;
use delivery_domain::entities::order::{OrderStatus, RoutingMode};
use delivery_domain::errors::{DomainError, DomainResult};
use delivery_domain::ports::OrderRepository;

use super::models::OrderRow;

pub struct PgOrderRepo {
    pub pool: PgPool,
}

const ORDER_SELECT: &str = r#"
    id, shop_id, driver_id, status::text, routing_mode::text,
    pickup_address, ST_Y(pickup_location) as pickup_lat, ST_X(pickup_location) as pickup_lng,
    dropoff_address, ST_Y(dropoff_location) as dropoff_lat, ST_X(dropoff_location) as dropoff_lng,
    distance_meters, customer_name, customer_phone, package_description,
    estimated_delivery_at, picked_up_at, delivered_at, batch_cluster_id,
    created_at, updated_at
"#;

#[async_trait]
impl OrderRepository for PgOrderRepo {
    async fn create(&self, order: &Order) -> DomainResult<Order> {
        let row = sqlx::query_as::<_, OrderRow>(&format!(
            r#"
            INSERT INTO orders (id, shop_id, status, routing_mode, pickup_address, pickup_location, dropoff_address, dropoff_location,
                distance_meters, customer_name, customer_phone, package_description, created_at, updated_at)
            VALUES ($1, $2, $3::order_status, $4::routing_mode,
                $5, ST_SetSRID(ST_MakePoint($6, $7), 4326),
                $8, ST_SetSRID(ST_MakePoint($9, $10), 4326),
                $11, $12, $13, $14, $15, $16)
            RETURNING {ORDER_SELECT}
            "#
        ))
        .bind(order.id)
        .bind(order.shop_id)
        .bind(order.status.as_str())
        .bind(order.routing_mode.as_str())
        .bind(&order.pickup_address)
        .bind(order.pickup_location.longitude)
        .bind(order.pickup_location.latitude)
        .bind(&order.dropoff_address)
        .bind(order.dropoff_location.longitude)
        .bind(order.dropoff_location.latitude)
        .bind(order.distance_meters)
        .bind(&order.customer_name)
        .bind(&order.customer_phone)
        .bind(&order.package_description)
        .bind(order.created_at)
        .bind(order.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        row.try_into()
    }

    async fn find_by_id(&self, id: Uuid) -> DomainResult<Option<Order>> {
        let row = sqlx::query_as::<_, OrderRow>(&format!(
            "SELECT {ORDER_SELECT} FROM orders WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(r.try_into()?)),
            None => Ok(None),
        }
    }

    async fn find_by_shop(&self, shop_id: Uuid, limit: i64, offset: i64) -> DomainResult<Vec<Order>> {
        let rows = sqlx::query_as::<_, OrderRow>(&format!(
            "SELECT {ORDER_SELECT} FROM orders WHERE shop_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        ))
        .bind(shop_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn find_by_driver(&self, driver_id: Uuid, status: Option<OrderStatus>) -> DomainResult<Vec<Order>> {
        let rows = match status {
            Some(s) => {
                sqlx::query_as::<_, OrderRow>(&format!(
                    "SELECT {ORDER_SELECT} FROM orders WHERE driver_id = $1 AND status = $2::order_status ORDER BY created_at DESC"
                ))
                .bind(driver_id)
                .bind(s.as_str())
                .fetch_all(&self.pool)
                .await
            }
            None => {
                sqlx::query_as::<_, OrderRow>(&format!(
                    "SELECT {ORDER_SELECT} FROM orders WHERE driver_id = $1 ORDER BY created_at DESC"
                ))
                .bind(driver_id)
                .fetch_all(&self.pool)
                .await
            }
        }
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn update_status(&self, id: Uuid, status: OrderStatus, driver_id: Option<Uuid>) -> DomainResult<()> {
        sqlx::query(
            r#"
            UPDATE orders SET status = $2::order_status, driver_id = COALESCE($3, driver_id), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status.as_str())
        .bind(driver_id)
        .execute(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }

    async fn find_pending_by_routing_mode(&self, mode: RoutingMode) -> DomainResult<Vec<Order>> {
        let rows = sqlx::query_as::<_, OrderRow>(&format!(
            "SELECT {ORDER_SELECT} FROM orders WHERE status = 'pending' AND routing_mode = $1::routing_mode"
        ))
        .bind(mode.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn assign_to_batch_cluster(&self, order_id: Uuid, cluster_id: Uuid) -> DomainResult<()> {
        sqlx::query("UPDATE orders SET batch_cluster_id = $2, updated_at = NOW() WHERE id = $1")
            .bind(order_id)
            .bind(cluster_id)
            .execute(&self.pool)
            .await
            .map_err(|e| DomainError::Infrastructure(e.to_string()))?;

        Ok(())
    }
}
