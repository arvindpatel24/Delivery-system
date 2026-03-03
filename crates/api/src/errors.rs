use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use delivery_domain::errors::DomainError;

use crate::response::ApiResponse;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self.0 {
            DomainError::NotFound { entity, id } => (
                StatusCode::NOT_FOUND,
                format!("{entity} with id {id} not found"),
            ),
            DomainError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            DomainError::Duplicate(msg) => (StatusCode::CONFLICT, msg),
            DomainError::InvalidStateTransition(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            DomainError::AuthenticationFailed(msg) => (StatusCode::UNAUTHORIZED, msg),
            DomainError::AuthorizationDenied(msg) => (StatusCode::FORBIDDEN, msg),
            DomainError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
            ),
            DomainError::NoDriversAvailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "No drivers available nearby".to_string(),
            ),
            DomainError::OfferExpired => (StatusCode::GONE, "Dispatch offer expired".to_string()),
            DomainError::ExternalService(msg) => (StatusCode::BAD_GATEWAY, msg),
            DomainError::Infrastructure(msg) => {
                tracing::error!(error = %msg, "Infrastructure error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let (status, json) = ApiResponse::<()>::error(status, msg);
        (status, json).into_response()
    }
}

pub struct ApiError(pub DomainError);

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        Self(e)
    }
}
