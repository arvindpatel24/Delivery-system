use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub database_max_connections: u32,
    pub location_pool_max_connections: u32,
    pub jwt_secret: String,
    pub jwt_expiry_hours: i64,
    pub auth_mode: String,
    pub rate_limit_shop_per_minute: u32,
    pub rate_limit_driver_per_second: u32,
    pub webhook_timeout_secs: u64,
    pub webhook_max_retries: i32,
    pub batch_schedule_hours: Vec<u32>,
    pub batch_cluster_eps_meters: f64,
    pub batch_max_orders_per_cluster: usize,
    pub instant_dispatch_radius_meters: f64,
    pub instant_dispatch_timeout_secs: i64,
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            database_max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("Invalid DATABASE_MAX_CONNECTIONS"),
            location_pool_max_connections: env::var("LOCATION_POOL_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Invalid LOCATION_POOL_MAX_CONNECTIONS"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("Invalid JWT_EXPIRY_HOURS"),
            auth_mode: env::var("AUTH_MODE").unwrap_or_else(|_| "real".to_string()),
            rate_limit_shop_per_minute: env::var("RATE_LIMIT_SHOP_PER_MINUTE")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .expect("Invalid RATE_LIMIT_SHOP_PER_MINUTE"),
            rate_limit_driver_per_second: env::var("RATE_LIMIT_DRIVER_PER_SECOND")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("Invalid RATE_LIMIT_DRIVER_PER_SECOND"),
            webhook_timeout_secs: env::var("WEBHOOK_TIMEOUT_SECS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Invalid WEBHOOK_TIMEOUT_SECS"),
            webhook_max_retries: env::var("WEBHOOK_MAX_RETRIES")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("Invalid WEBHOOK_MAX_RETRIES"),
            batch_schedule_hours: env::var("BATCH_SCHEDULE_HOURS")
                .unwrap_or_else(|_| "13,20".to_string())
                .split(',')
                .map(|s| s.trim().parse().expect("Invalid BATCH_SCHEDULE_HOURS"))
                .collect(),
            batch_cluster_eps_meters: env::var("BATCH_CLUSTER_EPS_METERS")
                .unwrap_or_else(|_| "1500".to_string())
                .parse()
                .expect("Invalid BATCH_CLUSTER_EPS_METERS"),
            batch_max_orders_per_cluster: env::var("BATCH_MAX_ORDERS_PER_CLUSTER")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("Invalid BATCH_MAX_ORDERS_PER_CLUSTER"),
            instant_dispatch_radius_meters: env::var("INSTANT_DISPATCH_RADIUS_METERS")
                .unwrap_or_else(|_| "2000".to_string())
                .parse()
                .expect("Invalid INSTANT_DISPATCH_RADIUS_METERS"),
            instant_dispatch_timeout_secs: env::var("INSTANT_DISPATCH_TIMEOUT_SECS")
                .unwrap_or_else(|_| "90".to_string())
                .parse()
                .expect("Invalid INSTANT_DISPATCH_TIMEOUT_SECS"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("Invalid PORT"),
        }
    }

    pub fn is_mock_auth(&self) -> bool {
        self.auth_mode == "mock"
    }
}
