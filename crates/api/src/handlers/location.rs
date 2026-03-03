use axum::extract::State;
use axum::Json;

use delivery_application::dto::{BulkLocationRequest, LocationPingRequest};
use delivery_domain::errors::DomainError;

use crate::app_state::AppState;
use crate::errors::ApiError;
use crate::middleware::auth::DriverIdentity;
use crate::openapi::{BulkLocationBody, LocationPingBody};
use crate::response::ApiResponse;

/// Send a single GPS location ping
#[utoipa::path(
    post,
    path = "/api/driver/location",
    tag = "driver",
    security(("driver_jwt" = [])),
    request_body = LocationPingBody,
    responses(
        (status = 200, description = "Location recorded"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn record_location(
    State(state): State<AppState>,
    axum::extract::Extension(identity): axum::extract::Extension<DriverIdentity>,
    Json(req): Json<LocationPingRequest>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    state
        .location_service
        .record_ping(identity.driver_id, req)
        .await?;

    Ok(ApiResponse::success(serde_json::json!({ "recorded": true })))
}

/// Bulk upload up to 500 GPS pings (for offline sync)
#[utoipa::path(
    post,
    path = "/api/driver/location/bulk",
    tag = "driver",
    security(("driver_jwt" = [])),
    request_body = BulkLocationBody,
    responses(
        (status = 200, description = "Pings inserted", body = serde_json::Value),
        (status = 400, description = "More than 500 pings"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn record_bulk_location(
    State(state): State<AppState>,
    axum::extract::Extension(identity): axum::extract::Extension<DriverIdentity>,
    Json(req): Json<BulkLocationRequest>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let ping_count = req.pings.len();
    if ping_count > 500 {
        return Err(ApiError(DomainError::Validation(
            "Maximum 500 pings per bulk request".to_string(),
        )));
    }

    let inserted = state
        .location_service
        .record_bulk(identity.driver_id, req)
        .await?;

    Ok(ApiResponse::success(serde_json::json!({
        "inserted": inserted,
        "received": ping_count
    })))
}
