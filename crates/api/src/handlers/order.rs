use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use uuid::Uuid;

use delivery_application::dto::CreateOrderRequest;
use delivery_domain::entities::order::OrderStatus;

use crate::app_state::AppState;
use crate::errors::ApiError;
use crate::middleware::auth::{DriverIdentity, ShopIdentity};
use crate::openapi::{CreateOrderBody, CreateOrderResponse, OrderResponse};
use crate::response::ApiResponse;

/// Create a new delivery order (auto-classifies as instant or batched)
#[utoipa::path(
    post,
    path = "/api/shop/orders",
    tag = "shop",
    security(("shop_api_key" = [])),
    request_body = CreateOrderBody,
    responses(
        (status = 201, description = "Order created", body = CreateOrderResponse),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn create_order(
    State(state): State<AppState>,
    axum::extract::Extension(identity): axum::extract::Extension<ShopIdentity>,
    Json(req): Json<CreateOrderRequest>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let resp = state
        .order_service
        .create_order(identity.shop_id, req)
        .await?;

    if resp.routing_mode == "instant" {
        let dispatch = state.dispatch_service.clone();
        let order_id = resp.id;
        tokio::spawn(async move {
            if let Err(e) = dispatch.dispatch_instant_order(order_id).await {
                tracing::error!(order_id = %order_id, error = %e, "Failed to dispatch instant order");
            }
        });
    }

    Ok(ApiResponse::created(resp))
}

/// Get a single order by ID
#[utoipa::path(
    get,
    path = "/api/shop/orders/{order_id}",
    tag = "shop",
    security(("shop_api_key" = [])),
    params(
        ("order_id" = Uuid, Path, description = "Order UUID")
    ),
    responses(
        (status = 200, description = "Order details", body = OrderResponse),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let resp = state.order_service.get_order(order_id).await?;
    Ok(ApiResponse::success(resp))
}

#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List all orders for the authenticated shop
#[utoipa::path(
    get,
    path = "/api/shop/orders",
    tag = "shop",
    security(("shop_api_key" = [])),
    params(
        ("limit" = Option<i64>, Query, description = "Page size (default 20)"),
        ("offset" = Option<i64>, Query, description = "Page offset (default 0)"),
    ),
    responses(
        (status = 200, description = "List of orders"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_shop_orders(
    State(state): State<AppState>,
    axum::extract::Extension(identity): axum::extract::Extension<ShopIdentity>,
    Query(query): Query<ListOrdersQuery>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let limit = query.limit.unwrap_or(20);
    let offset = query.offset.unwrap_or(0);
    let orders = state
        .order_service
        .list_shop_orders(identity.shop_id, limit, offset)
        .await?;
    Ok(ApiResponse::success(orders))
}

#[derive(Debug, Deserialize)]
pub struct DriverOrdersQuery {
    pub status: Option<String>,
}

/// List orders assigned to the authenticated driver
#[utoipa::path(
    get,
    path = "/api/driver/orders",
    tag = "driver",
    security(("driver_jwt" = [])),
    params(
        ("status" = Option<String>, Query, description = "Filter by status (e.g. assigned, picked_up)")
    ),
    responses(
        (status = 200, description = "List of driver orders"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_driver_orders(
    State(state): State<AppState>,
    axum::extract::Extension(identity): axum::extract::Extension<DriverIdentity>,
    Query(query): Query<DriverOrdersQuery>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let status = query
        .status
        .as_deref()
        .map(OrderStatus::from_str)
        .transpose()?;

    let orders = state
        .order_service
        .list_driver_orders(identity.driver_id, status)
        .await?;
    Ok(ApiResponse::success(orders))
}

/// Update the status of an order (driver workflow: picked_up → in_transit → delivered)
#[utoipa::path(
    put,
    path = "/api/driver/orders/{order_id}/status",
    tag = "driver",
    security(("driver_jwt" = [])),
    params(
        ("order_id" = Uuid, Path, description = "Order UUID")
    ),
    request_body(
        content_type = "application/json",
        description = r#"JSON body with `status` field. Valid transitions: assigned→picked_up, picked_up→in_transit, in_transit→delivered"#,
        content = serde_json::Value,
    ),
    responses(
        (status = 200, description = "Status updated"),
        (status = 422, description = "Invalid state transition"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn update_order_status(
    State(state): State<AppState>,
    axum::extract::Extension(_identity): axum::extract::Extension<DriverIdentity>,
    Path(order_id): Path<Uuid>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let status_str = body
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or(delivery_domain::errors::DomainError::Validation(
            "Missing 'status' field".to_string(),
        ))?;

    let new_status = OrderStatus::from_str(status_str)?;
    state
        .order_service
        .update_order_status(order_id, new_status)
        .await?;

    Ok(ApiResponse::success(serde_json::json!({
        "order_id": order_id,
        "status": status_str,
    })))
}
