mod common;

#[cfg(test)]
mod tests {
    use super::common;

    #[tokio::test]
    #[ignore = "requires database"]
    async fn test_bulk_location_insert() {
        let pool = common::setup_test_db().await;
        common::cleanup_test_db(&pool).await;

        let driver_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"INSERT INTO drivers (id, name, phone, password_hash)
               VALUES ($1, 'Driver', '3333333333', 'hash')"#,
        )
        .bind(driver_id)
        .execute(&pool)
        .await
        .unwrap();

        // Bulk insert using UNNEST
        let driver_ids = vec![driver_id; 10];
        let lngs: Vec<f64> = (0..10).map(|i| 77.0 + i as f64 * 0.001).collect();
        let lats: Vec<f64> = (0..10).map(|i| 23.0 + i as f64 * 0.001).collect();
        let recorded_ats: Vec<chrono::DateTime<chrono::Utc>> =
            (0..10).map(|i| chrono::Utc::now() - chrono::Duration::seconds(i * 10)).collect();

        let result = sqlx::query(
            r#"INSERT INTO driver_locations (driver_id, location, is_offline_sync, recorded_at)
               SELECT unnest($1::uuid[]), ST_SetSRID(ST_MakePoint(unnest($2::float8[]), unnest($3::float8[])), 4326),
                      FALSE, unnest($4::timestamptz[])"#,
        )
        .bind(&driver_ids)
        .bind(&lngs)
        .bind(&lats)
        .bind(&recorded_ats)
        .execute(&pool)
        .await
        .unwrap();

        assert_eq!(result.rows_affected(), 10);

        // Verify count
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM driver_locations WHERE driver_id = $1")
                .bind(driver_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(count, 10);

        common::cleanup_test_db(&pool).await;
    }
}
