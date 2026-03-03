use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("Entity not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: Uuid },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate entry: {0}")]
    Duplicate(String),

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("No available drivers within range")]
    NoDriversAvailable,

    #[error("Dispatch offer expired")]
    OfferExpired,

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
}

pub type DomainResult<T> = Result<T, DomainError>;
