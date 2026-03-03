use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use crate::app_state::AppState;
use crate::openapi::ApiHealthResponse;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

/// Check server and database health
#[utoipa::path(
    get,
    path = "/health",
    tag = "public",
    responses(
        (status = 200, description = "Service healthy", body = ApiHealthResponse),
        (status = 503, description = "Service unhealthy", body = ApiHealthResponse),
    )
)]
pub async fn health_check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let db_status = match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db_pool)
        .await
    {
        Ok(_) => "connected".to_string(),
        Err(e) => format!("error: {e}"),
    };

    let status = if db_status == "connected" {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(HealthResponse {
            status: if status == StatusCode::OK {
                "healthy".to_string()
            } else {
                "unhealthy".to_string()
            },
            database: db_status,
        }),
    )
}
