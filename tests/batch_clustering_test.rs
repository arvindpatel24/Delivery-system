mod common;

#[cfg(test)]
mod tests {
    use super::common;

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_spatial_clustering() {
        let pool = common::setup_test_db().await;
        common::cleanup_test_db(&pool).await;

        let shop_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO shops (id, name, phone, api_key_hash, location, address)
               VALUES ($1, 'Shop', '5555555555', 'hash', ST_SetSRID(ST_MakePoint(77.0, 23.0), 4326), 'Addr')"#,
        )
        .bind(shop_id)
        .execute(&pool)
        .await
        .unwrap();

        // Create 10 orders clustered in 2 groups
        // Group 1: around (77.0, 23.0)
        for i in 0..5 {
            let id = uuid::Uuid::new_v4();
            let lng = 77.0 + (i as f64 * 0.001);
            let lat = 23.0 + (i as f64 * 0.001);
            sqlx::query(
                r#"INSERT INTO orders (id, shop_id, status, routing_mode, pickup_address, pickup_location, dropoff_address, dropoff_location, distance_meters)
                   VALUES ($1, $2, 'pending'::order_status, 'batched'::routing_mode, 'P', ST_SetSRID(ST_MakePoint(77.0, 23.0), 4326), 'D', ST_SetSRID(ST_MakePoint($3, $4), 4326), 5000)"#,
            )
            .bind(id)
            .bind(shop_id)
            .bind(lng)
            .bind(lat)
            .execute(&pool)
            .await
            .unwrap();
        }

        // Group 2: around (77.1, 23.1) — 10km+ away
        for i in 0..5 {
            let id = uuid::Uuid::new_v4();
            let lng = 77.1 + (i as f64 * 0.001);
            let lat = 23.1 + (i as f64 * 0.001);
            sqlx::query(
                r#"INSERT INTO orders (id, shop_id, status, routing_mode, pickup_address, pickup_location, dropoff_address, dropoff_location, distance_meters)
                   VALUES ($1, $2, 'pending'::order_status, 'batched'::routing_mode, 'P', ST_SetSRID(ST_MakePoint(77.0, 23.0), 4326), 'D', ST_SetSRID(ST_MakePoint($3, $4), 4326), 15000)"#,
            )
            .bind(id)
            .bind(shop_id)
            .bind(lng)
            .bind(lat)
            .execute(&pool)
            .await
            .unwrap();
        }

        // Run clustering query
        let clusters = sqlx::query_as::<_, (i32, i64)>(
            r#"WITH clustered AS (
                SELECT
                    ST_ClusterDBSCAN(ST_Transform(dropoff_location, 32643), eps := 1500, minpoints := 1) OVER () as cluster_label
                FROM orders
                WHERE status = 'pending' AND routing_mode = 'batched'
            )
            SELECT cluster_label, COUNT(*) as cnt
            FROM clustered
            WHERE cluster_label IS NOT NULL
            GROUP BY cluster_label
            ORDER BY cluster_label"#,
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        // Should have 2 clusters
        assert!(clusters.len() >= 2, "Expected at least 2 clusters, got {}", clusters.len());

        common::cleanup_test_db(&pool).await;
    }
}
