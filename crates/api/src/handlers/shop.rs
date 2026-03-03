use axum::extract::State;
use axum::Json;

use delivery_application::dto::CreateShopRequest;
use delivery_infrastructure::auth::ApiKeyManager;

use crate::app_state::AppState;
use crate::errors::ApiError;
use crate::openapi::{RegisterShopBody, RegisterShopResponse};
use crate::response::ApiResponse;

/// Register a new shopkeeper account and receive an API key
#[utoipa::path(
    post,
    path = "/shops/register",
    tag = "public",
    request_body = RegisterShopBody,
    responses(
        (status = 201, description = "Shop created", body = RegisterShopResponse),
        (status = 409, description = "Phone already registered"),
        (status = 400, description = "Validation error"),
    )
)]
pub async fn register_shop(
    State(state): State<AppState>,
    Json(req): Json<CreateShopRequest>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let raw_key = ApiKeyManager::generate_api_key();
    let hash = ApiKeyManager::hash_api_key(&raw_key);

    let resp = state.shop_service.create_shop(req, hash, raw_key).await?;
    Ok(ApiResponse::created(resp))
}
