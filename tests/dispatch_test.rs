mod common;

#[cfg(test)]
mod tests {
    use super::common;

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_dispatch_offer_lifecycle() {
        let pool = common::setup_test_db().await;
        common::cleanup_test_db(&pool).await;

        // Setup shop, driver, order
        let shop_id = uuid::Uuid::new_v4();
        let driver_id = uuid::Uuid::new_v4();
        let order_id = uuid::Uuid::new_v4();
        let offer_id = uuid::Uuid::new_v4();

        sqlx::query(
            r#"INSERT INTO shops (id, name, phone, api_key_hash, location, address)
               VALUES ($1, 'Shop', '1111111111', 'hash', ST_SetSRID(ST_MakePoint(77.0, 23.0), 4326), 'Addr')"#,
        )
        .bind(shop_id)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"INSERT INTO drivers (id, name, phone, password_hash, current_location, is_available)
               VALUES ($1, 'Driver', '2222222222', 'hash', ST_SetSRID(ST_MakePoint(77.0, 23.0), 4326), TRUE)"#,
        )
        .bind(driver_id)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"INSERT INTO orders (id, shop_id, status, routing_mode, pickup_address, pickup_location, dropoff_address, dropoff_location, distance_meters)
               VALUES ($1, $2, 'dispatching'::order_status, 'instant'::routing_mode, 'A', ST_SetSRID(ST_MakePoint(77.0, 23.0), 4326), 'B', ST_SetSRID(ST_MakePoint(77.01, 23.01), 4326), 1000)"#,
        )
        .bind(order_id)
        .bind(shop_id)
        .execute(&pool)
        .await
        .unwrap();

        // Create dispatch offer
        sqlx::query(
            r#"INSERT INTO driver_dispatch_offers (id, order_id, driver_id, status, distance_to_pickup_meters, expires_at)
               VALUES ($1, $2, $3, 'pending'::offer_status, 500, NOW() + INTERVAL '90 seconds')"#,
        )
        .bind(offer_id)
        .bind(order_id)
        .bind(driver_id)
        .execute(&pool)
        .await
        .unwrap();

        // Accept offer
        sqlx::query("UPDATE driver_dispatch_offers SET status = 'accepted'::offer_status WHERE id = $1")
            .bind(offer_id)
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("UPDATE orders SET status = 'assigned'::order_status, driver_id = $2 WHERE id = $1")
            .bind(order_id)
            .bind(driver_id)
            .execute(&pool)
            .await
            .unwrap();

        // Verify
        let (status,): (String,) = sqlx::query_as("SELECT status::text FROM orders WHERE id = $1")
            .bind(order_id)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(status, "assigned");

        common::cleanup_test_db(&pool).await;
    }
}
