use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::app_state::AppState;
use crate::handlers;
use crate::middleware::auth::{driver_auth_middleware, shop_auth_middleware};
use crate::openapi::ApiDoc;

pub fn build_router(state: AppState) -> Router {
    let shop_routes = Router::new()
        .route("/orders", post(handlers::order::create_order))
        .route("/orders", get(handlers::order::list_shop_orders))
        .route("/orders/{order_id}", get(handlers::order::get_order))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            shop_auth_middleware,
        ));

    let driver_routes = Router::new()
        .route("/availability", put(handlers::driver::set_availability))
        .route("/location", post(handlers::location::record_location))
        .route(
            "/location/bulk",
            post(handlers::location::record_bulk_location),
        )
        .route("/orders", get(handlers::order::list_driver_orders))
        .route(
            "/orders/{order_id}/status",
            put(handlers::order::update_order_status),
        )
        .route("/offers", get(handlers::dispatch::get_pending_offers))
        .route(
            "/offers/{offer_id}",
            put(handlers::dispatch::respond_to_offer),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            driver_auth_middleware,
        ));

    let public_routes = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/shops/register", post(handlers::shop::register_shop))
        .route("/drivers/register", post(handlers::driver::register_driver))
        .route("/drivers/login", post(handlers::driver::login_driver));

    // Serve OpenAPI JSON at /api-docs/openapi.json
    // Serve Swagger UI at /docs  (or /docs/)
    let swagger = SwaggerUi::new("/docs")
        .url("/api-docs/openapi.json", ApiDoc::openapi());

    // with_state() collapses the type to Router<()>; swagger is also Router<()>
    Router::new()
        .nest("/api/shop", shop_routes)
        .nest("/api/driver", driver_routes)
        .merge(public_routes)
        .with_state(state)
        .merge(swagger)
}
