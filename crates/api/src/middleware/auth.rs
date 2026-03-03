use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use uuid::Uuid;

use delivery_infrastructure::auth::ApiKeyManager;

use crate::app_state::AppState;
use crate::response::ApiResponse;

/// Extract shop identity from X-Api-Key header.
pub async fn shop_auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<()>>)> {
    if state.config.is_mock_auth() {
        // In mock mode, use a fixed shop ID from header or default
        let shop_id = req
            .headers()
            .get("X-Mock-Shop-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::nil);
        req.extensions_mut().insert(ShopIdentity { shop_id });
        return Ok(next.run(req).await);
    }

    let api_key = req
        .headers()
        .get("X-Api-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let api_key = match api_key {
        Some(k) => k,
        None => {
            return Err(ApiResponse::<()>::error(
                StatusCode::UNAUTHORIZED,
                "Missing X-Api-Key header".to_string(),
            ));
        }
    };

    let hash = ApiKeyManager::hash_api_key(&api_key);
    let shop = state.shop_service.shop_repo.find_by_api_key_hash(&hash).await;

    match shop {
        Ok(Some(shop)) => {
            req.extensions_mut().insert(ShopIdentity { shop_id: shop.id });
            Ok(next.run(req).await)
        }
        _ => Err(ApiResponse::<()>::error(
            StatusCode::UNAUTHORIZED,
            "Invalid API key".to_string(),
        )),
    }
}

/// Extract driver identity from Authorization: Bearer <jwt> header.
pub async fn driver_auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ApiResponse<()>>)> {
    if state.config.is_mock_auth() {
        let driver_id = req
            .headers()
            .get("X-Mock-Driver-Id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| Uuid::parse_str(s).ok())
            .unwrap_or_else(Uuid::nil);
        req.extensions_mut().insert(DriverIdentity { driver_id });
        return Ok(next.run(req).await);
    }

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    let token = match token {
        Some(t) => t,
        None => {
            return Err(ApiResponse::<()>::error(
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Authorization header".to_string(),
            ));
        }
    };

    match state.jwt_manager.validate_token(&token) {
        Ok(claims) => {
            req.extensions_mut().insert(DriverIdentity {
                driver_id: claims.sub,
            });
            Ok(next.run(req).await)
        }
        Err(e) => Err(ApiResponse::<()>::error(
            StatusCode::UNAUTHORIZED,
            format!("Invalid token: {e}"),
        )),
    }
}

#[derive(Debug, Clone)]
pub struct ShopIdentity {
    pub shop_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct DriverIdentity {
    pub driver_id: Uuid,
}
