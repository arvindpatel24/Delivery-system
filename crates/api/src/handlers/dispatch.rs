use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use delivery_application::dto::RespondToOfferRequest;

use crate::app_state::AppState;
use crate::errors::ApiError;
use crate::middleware::auth::DriverIdentity;
use crate::openapi::RespondOfferBody;
use crate::response::ApiResponse;

/// List pending dispatch offers for the authenticated driver
#[utoipa::path(
    get,
    path = "/api/driver/offers",
    tag = "driver",
    security(("driver_jwt" = [])),
    responses(
        (status = 200, description = "List of pending offers"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_pending_offers(
    State(state): State<AppState>,
    axum::extract::Extension(identity): axum::extract::Extension<DriverIdentity>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let offers = state
        .dispatch_service
        .offer_repo
        .find_pending_for_driver(identity.driver_id)
        .await?;

    Ok(ApiResponse::success(offers))
}

/// Accept or reject a dispatch offer
#[utoipa::path(
    put,
    path = "/api/driver/offers/{offer_id}",
    tag = "driver",
    security(("driver_jwt" = [])),
    params(
        ("offer_id" = Uuid, Path, description = "Offer UUID")
    ),
    request_body = RespondOfferBody,
    responses(
        (status = 200, description = "Response recorded"),
        (status = 403, description = "Offer not assigned to this driver"),
        (status = 410, description = "Offer expired"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn respond_to_offer(
    State(state): State<AppState>,
    axum::extract::Extension(identity): axum::extract::Extension<DriverIdentity>,
    Path(offer_id): Path<Uuid>,
    Json(req): Json<RespondToOfferRequest>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    if req.accept {
        state
            .dispatch_service
            .accept_offer(offer_id, identity.driver_id)
            .await?;
    } else {
        state
            .dispatch_service
            .reject_offer(offer_id, identity.driver_id)
            .await?;
    }

    Ok(ApiResponse::success(serde_json::json!({
        "offer_id": offer_id,
        "accepted": req.accept
    })))
}
