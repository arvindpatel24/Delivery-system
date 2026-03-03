use std::sync::Arc;

use sqlx::PgPool;

pub async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://samagri:samagri@localhost:5432/samagri_delivery_test".to_string());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    let migrations = [
        include_str!("../../migrations/001_enable_extensions.sql"),
        include_str!("../../migrations/002_create_shops.sql"),
        include_str!("../../migrations/003_create_drivers.sql"),
        include_str!("../../migrations/004_create_driver_locations.sql"),
        include_str!("../../migrations/005_create_orders.sql"),
        include_str!("../../migrations/006_create_batch_tables.sql"),
        include_str!("../../migrations/007_create_webhook_outbox.sql"),
        include_str!("../../migrations/008_create_dispatch_offers.sql"),
    ];

    for sql in &migrations {
        sqlx::raw_sql(sql).execute(&pool).await.ok();
    }

    pool
}

pub async fn cleanup_test_db(pool: &PgPool) {
    let tables = [
        "driver_dispatch_offers",
        "webhook_outbox",
        "batch_clusters",
        "batch_runs",
        "driver_locations",
        "orders",
        "drivers",
        "shops",
    ];

    for table in &tables {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(pool)
            .await
            .ok();
    }
}
