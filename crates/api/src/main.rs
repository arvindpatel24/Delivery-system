use std::sync::Arc;

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use delivery_application::services::{
    batch_service::BatchService,
    dispatch_service::DispatchService,
    driver_service::DriverService,
    location_service::LocationService,
    order_service::OrderService,
    shop_service::ShopService,
    webhook_service::WebhookService,
};
use delivery_infrastructure::{
    auth::JwtManager,
    external::{HttpWebhookSender, MockGeocoder},
    geospatial::PostgisEngine,
    persistence::{
        pg_batch_repo::PgBatchRepo,
        pg_dispatch_offer_repo::PgDispatchOfferRepo,
        pg_driver_repo::PgDriverRepo,
        pg_location_repo::PgLocationRepo,
        pg_order_repo::PgOrderRepo,
        pg_shop_repo::PgShopRepo,
        pg_webhook_outbox_repo::PgWebhookOutboxRepo,
    },
};

mod app_state;
mod config;
mod errors;
mod extractors;
mod handlers;
mod middleware;
mod openapi;
mod response;
mod router;
mod workers;

use app_state::AppState;
use config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env
    dotenvy::dotenv().ok();

    // Tracing
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "info,delivery_api=debug,delivery_infrastructure=debug".into()
        }))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let config = AppConfig::from_env();
    tracing::info!(host = %config.host, port = config.port, auth_mode = %config.auth_mode, "Starting delivery system");

    // Main connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await?;

    tracing::info!("Connected to database");

    // Run migrations
    run_migrations(&db_pool).await?;

    // Separate pool for location writes
    let location_pool = PgPoolOptions::new()
        .max_connections(config.location_pool_max_connections)
        .connect(&config.database_url)
        .await?;

    // Repositories
    let shop_repo = Arc::new(PgShopRepo { pool: db_pool.clone() });
    let driver_repo = Arc::new(PgDriverRepo { pool: db_pool.clone() });
    let order_repo = Arc::new(PgOrderRepo { pool: db_pool.clone() });
    let location_repo = Arc::new(PgLocationRepo { pool: location_pool });
    let offer_repo = Arc::new(PgDispatchOfferRepo { pool: db_pool.clone() });
    let webhook_repo = Arc::new(PgWebhookOutboxRepo { pool: db_pool.clone() });
    let batch_repo = Arc::new(PgBatchRepo { pool: db_pool.clone() });

    // Infrastructure
    let geospatial = Arc::new(PostgisEngine { pool: db_pool.clone() });
    let geocoder = Arc::new(MockGeocoder);
    let webhook_sender = Arc::new(HttpWebhookSender::new());
    let jwt_manager = Arc::new(JwtManager::new(&config.jwt_secret, config.jwt_expiry_hours));

    // Services
    let shop_service = Arc::new(ShopService {
        shop_repo: shop_repo.clone(),
        geocoder: geocoder.clone(),
    });

    let driver_service = Arc::new(DriverService {
        driver_repo: driver_repo.clone(),
    });

    let order_service = Arc::new(OrderService {
        order_repo: order_repo.clone(),
        shop_repo: shop_repo.clone(),
        geospatial: geospatial.clone(),
        geocoder: geocoder.clone(),
    });

    let dispatch_service = Arc::new(DispatchService {
        order_repo: order_repo.clone(),
        driver_repo: driver_repo.clone(),
        offer_repo: offer_repo.clone(),
        geospatial: geospatial.clone(),
        dispatch_timeout_secs: config.instant_dispatch_timeout_secs,
        dispatch_radius_meters: config.instant_dispatch_radius_meters,
    });

    let location_service = Arc::new(LocationService {
        location_repo: location_repo.clone(),
        driver_repo: driver_repo.clone(),
    });

    let webhook_service = Arc::new(WebhookService {
        outbox_repo: webhook_repo.clone(),
        sender: webhook_sender,
        max_retries: config.webhook_max_retries,
        timeout_secs: config.webhook_timeout_secs,
    });

    let batch_service = Arc::new(BatchService {
        order_repo: order_repo.clone(),
        batch_repo: batch_repo.clone(),
        driver_repo: driver_repo.clone(),
        geospatial: geospatial.clone(),
        cluster_eps_meters: config.batch_cluster_eps_meters,
        max_orders_per_cluster: config.batch_max_orders_per_cluster,
    });

    // App state
    let state = AppState {
        config: config.clone(),
        shop_service,
        driver_service,
        order_service,
        dispatch_service: dispatch_service.clone(),
        location_service: location_service.clone(),
        webhook_service: webhook_service.clone(),
        batch_service: batch_service.clone(),
        jwt_manager,
        db_pool: db_pool.clone(),
    };

    // Spawn background workers
    let schedule_hours = config.batch_schedule_hours.clone();
    tokio::spawn(workers::webhook_dispatcher::webhook_dispatcher_worker(
        webhook_service,
    ));
    tokio::spawn(workers::location_cleanup::location_cleanup_worker(
        location_service,
    ));
    tokio::spawn(workers::stale_order_checker::stale_order_checker_worker(
        dispatch_service,
    ));
    tokio::spawn(workers::batch_scheduler::batch_scheduler_worker(
        batch_service,
        db_pool.clone(),
        schedule_hours,
    ));

    // Build router
    let app = router::build_router(state).layer(TraceLayer::new_for_http());

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    tracing::info!(addr = %addr, "Server listening");
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn run_migrations(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    tracing::info!("Running database migrations");

    let migration_files = [
        include_str!("../../../migrations/001_enable_extensions.sql"),
        include_str!("../../../migrations/002_create_shops.sql"),
        include_str!("../../../migrations/003_create_drivers.sql"),
        include_str!("../../../migrations/004_create_driver_locations.sql"),
        include_str!("../../../migrations/005_create_orders.sql"),
        include_str!("../../../migrations/006_create_batch_tables.sql"),
        include_str!("../../../migrations/007_create_webhook_outbox.sql"),
        include_str!("../../../migrations/008_create_dispatch_offers.sql"),
    ];

    for (i, sql) in migration_files.iter().enumerate() {
        tracing::debug!(migration = i + 1, "Applying migration");
        // Use IF NOT EXISTS patterns in SQL, so re-running is safe
        sqlx::raw_sql(sql).execute(pool).await.map_err(|e| {
            tracing::warn!(migration = i + 1, error = %e, "Migration may have already been applied");
            e
        }).ok();
    }

    tracing::info!("Migrations complete");
    Ok(())
}
