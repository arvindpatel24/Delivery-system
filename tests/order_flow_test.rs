mod common;

/// Integration test: Full order flow
/// Requires a running PostgreSQL+PostGIS instance.
/// Set TEST_DATABASE_URL env var to run.
#[cfg(test)]
mod tests {
    use super::common;

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_create_order_instant_flow() {
        let pool = common::setup_test_db().await;
        common::cleanup_test_db(&pool).await;

        // 1. Create a shop
        let shop_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO shops (id, name, phone, api_key_hash, location, address)
               VALUES ($1, 'Test Shop', '9999999999', 'testhash123',
                       ST_SetSRID(ST_MakePoint(77.4126, 23.2599), 4326), 'Test Address')"#,
        )
        .bind(shop_id)
        .execute(&pool)
        .await
        .unwrap();

        // 2. Create a driver nearby
        let driver_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO drivers (id, name, phone, password_hash, current_location, is_available)
               VALUES ($1, 'Test Driver', '8888888888', 'hash',
                       ST_SetSRID(ST_MakePoint(77.413, 23.260), 4326), TRUE)"#,
        )
        .bind(driver_id)
        .execute(&pool)
        .await
        .unwrap();

        // 3. Create an instant order (within 2km)
        let order_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO orders (id, shop_id, status, routing_mode,
                pickup_address, pickup_location,
                dropoff_address, dropoff_location, distance_meters)
               VALUES ($1, $2, 'pending'::order_status, 'instant'::routing_mode,
                       'Pickup St', ST_SetSRID(ST_MakePoint(77.4126, 23.2599), 4326),
                       'Dropoff St', ST_SetSRID(ST_MakePoint(77.4150, 23.2610), 4326), 500)"#,
        )
        .bind(order_id)
        .bind(shop_id)
        .execute(&pool)
        .await
        .unwrap();

        // 4. Verify nearby driver query works
        let nearby = sqlx::query_as::<_, (uuid::Uuid, f64)>(
            r#"SELECT d.id, ST_Distance(d.current_location::geography,
                   ST_SetSRID(ST_MakePoint(77.4126, 23.2599), 4326)::geography) as dist
               FROM drivers d
               WHERE d.is_available = TRUE
                 AND ST_DWithin(d.current_location::geography,
                     ST_SetSRID(ST_MakePoint(77.4126, 23.2599), 4326)::geography, 2000)"#,
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert!(!nearby.is_empty(), "Should find nearby driver");
        assert_eq!(nearby[0].0, driver_id);

        common::cleanup_test_db(&pool).await;
    }
}
