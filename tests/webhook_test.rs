mod common;

#[cfg(test)]
mod tests {
    use super::common;

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_webhook_outbox_lifecycle() {
        let pool = common::setup_test_db().await;
        common::cleanup_test_db(&pool).await;

        // Create shop and order first
        let shop_id = uuid::Uuid::new_v4();
        let order_id = uuid::Uuid::new_v4();

        sqlx::query(
            r#"INSERT INTO shops (id, name, phone, api_key_hash, location, address, webhook_url)
               VALUES ($1, 'Shop', '4444444444', 'hash', ST_SetSRID(ST_MakePoint(77.0, 23.0), 4326), 'Addr', 'https://example.com/webhook')"#,
        )
        .bind(shop_id)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"INSERT INTO orders (id, shop_id, status, routing_mode, pickup_address, pickup_location, dropoff_address, dropoff_location, distance_meters)
               VALUES ($1, $2, 'assigned'::order_status, 'instant'::routing_mode, 'A', ST_SetSRID(ST_MakePoint(77.0, 23.0), 4326), 'B', ST_SetSRID(ST_MakePoint(77.01, 23.01), 4326), 1000)"#,
        )
        .bind(order_id)
        .bind(shop_id)
        .execute(&pool)
        .await
        .unwrap();

        // Insert webhook
        let webhook_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO webhook_outbox (id, order_id, shop_id, webhook_url, payload, status)
               VALUES ($1, $2, $3, 'https://example.com/webhook', '{"lat": 23.0, "lng": 77.0}'::jsonb, 'pending'::webhook_status)"#,
        )
        .bind(webhook_id)
        .bind(order_id)
        .bind(shop_id)
        .execute(&pool)
        .await
        .unwrap();

        // Fetch pending with FOR UPDATE SKIP LOCKED
        let pending = sqlx::query_as::<_, (uuid::Uuid,)>(
            r#"SELECT id FROM webhook_outbox
               WHERE (status = 'pending' OR status = 'failed') AND next_retry_at <= NOW()
               FOR UPDATE SKIP LOCKED LIMIT 10"#,
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, webhook_id);

        // Mark delivered
        sqlx::query("UPDATE webhook_outbox SET status = 'delivered'::webhook_status, delivered_at = NOW() WHERE id = $1")
            .bind(webhook_id)
            .execute(&pool)
            .await
            .unwrap();

        // Verify no pending
        let pending_after = sqlx::query_as::<_, (uuid::Uuid,)>(
            "SELECT id FROM webhook_outbox WHERE status = 'pending' OR status = 'failed'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert!(pending_after.is_empty());

        common::cleanup_test_db(&pool).await;
    }
}
