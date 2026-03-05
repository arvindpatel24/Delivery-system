use axum::extract::State;
use axum::Json;

use delivery_application::dto::{LoginDriverRequest, LoginDriverResponse, RegisterDriverRequest};
use delivery_domain::errors::DomainError;

use crate::app_state::AppState;
use crate::errors::ApiError;
use crate::middleware::auth::DriverIdentity;
use crate::openapi::{AvailabilityBody, LoginDriverBody, RegisterDriverBody, RegisterDriverResponse};
use crate::response::ApiResponse;

/// Register a new driver account
#[utoipa::path(
    post,
    path = "/drivers/register",
    tag = "public",
    request_body = RegisterDriverBody,
    responses(
        (status = 201, description = "Driver registered, JWT token returned", body = RegisterDriverResponse),
        (status = 409, description = "Phone already registered"),
    )
)]
pub async fn register_driver(
    State(state): State<AppState>,
    Json(req): Json<RegisterDriverRequest>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let password_hash = hash_password(&req.password)?;

    let driver_id = uuid::Uuid::now_v7();
    let token = state
        .jwt_manager
        .generate_token(driver_id, "driver")
        .map_err(|e| ApiError(DomainError::Infrastructure(e)))?;

    // driver_id is passed so the DB record and the JWT contain the same UUID
    let resp = state.driver_service.register(driver_id, req, password_hash, token).await?;
    Ok(ApiResponse::created(resp))
}

/// Login as a driver and receive a JWT token
#[utoipa::path(
    post,
    path = "/drivers/login",
    tag = "public",
    request_body = LoginDriverBody,
    responses(
        (status = 200, description = "Login successful, JWT returned"),
        (status = 401, description = "Invalid credentials"),
    )
)]
pub async fn login_driver(
    State(state): State<AppState>,
    Json(req): Json<LoginDriverRequest>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let driver = state
        .driver_service
        .driver_repo
        .find_by_phone(&req.phone)
        .await?
        .ok_or(DomainError::AuthenticationFailed(
            "Invalid credentials".to_string(),
        ))?;

    verify_password(&req.password, &driver.password_hash)?;

    let token = state
        .jwt_manager
        .generate_token(driver.id, "driver")
        .map_err(|e| ApiError(DomainError::Infrastructure(e)))?;

    Ok(ApiResponse::success(LoginDriverResponse {
        token,
        driver_id: driver.id,
        name: driver.name,
    }))
}

/// Set driver online/offline availability
#[utoipa::path(
    put,
    path = "/api/driver/availability",
    tag = "driver",
    security(("driver_jwt" = [])),
    request_body = AvailabilityBody,
    responses(
        (status = 200, description = "Availability updated"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn set_availability(
    State(state): State<AppState>,
    axum::extract::Extension(identity): axum::extract::Extension<DriverIdentity>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl axum::response::IntoResponse, ApiError> {
    let available = body
        .get("available")
        .and_then(|v| v.as_bool())
        .ok_or(DomainError::Validation(
            "Missing 'available' boolean field".to_string(),
        ))?;

    state
        .driver_service
        .set_availability(identity.driver_id, available)
        .await?;

    Ok(ApiResponse::success(serde_json::json!({
        "driver_id": identity.driver_id,
        "is_available": available
    })))
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| ApiError(DomainError::Infrastructure(format!("Password hash error: {e}"))))?;

    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> Result<(), ApiError> {
    use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};

    let parsed = PasswordHash::new(hash)
        .map_err(|e| ApiError(DomainError::Infrastructure(format!("Invalid hash: {e}"))))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| {
            ApiError(DomainError::AuthenticationFailed(
                "Invalid credentials".to_string(),
            ))
        })
}
