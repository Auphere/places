// src/errors.rs
// DOCUMENTATION: Custom error types and HTTP responses
// PURPOSE: Centralized error handling for entire application

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde_json::json;
use thiserror::Error;

/// Application-specific error types
/// DOCUMENTATION: Comprehensive error enum for all possible failures
/// Each variant maps to appropriate HTTP status code and error response
#[derive(Error, Debug)]
pub enum PlacesError {
    #[error("Place not found with id: {0}")]
    NotFound(String),

    #[error("Place already exists: {0}")]
    #[allow(dead_code)]
    AlreadyExists(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Forbidden access")]
    Forbidden,

    #[error("Internal server error")]
    #[allow(dead_code)]
    InternalError,

    #[error("External API error: {0}")]
    ExternalApiError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service temporarily unavailable")]
    #[allow(dead_code)]
    ServiceUnavailable,
}

/// Convert PlacesError to HTTP response
/// DOCUMENTATION: Maps error types to HTTP status codes and JSON responses
impl ResponseError for PlacesError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_code) = match self {
            PlacesError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            PlacesError::AlreadyExists(_) => (StatusCode::CONFLICT, "ALREADY_EXISTS"),
            PlacesError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            PlacesError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "INVALID_INPUT"),
            PlacesError::ValidationError(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            PlacesError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            PlacesError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            PlacesError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
            PlacesError::ExternalApiError(_) => (StatusCode::BAD_GATEWAY, "EXTERNAL_API_ERROR"),
            PlacesError::RateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED")
            }
            PlacesError::ServiceUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "SERVICE_UNAVAILABLE")
            }
        };

        let body = json!({
            "error": {
                "code": error_code,
                "message": self.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        });

        HttpResponse::build(status).json(body)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            PlacesError::NotFound(_) => StatusCode::NOT_FOUND,
            PlacesError::AlreadyExists(_) => StatusCode::CONFLICT,
            PlacesError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            PlacesError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            PlacesError::ValidationError(_) => StatusCode::BAD_REQUEST,
            PlacesError::Unauthorized => StatusCode::UNAUTHORIZED,
            PlacesError::Forbidden => StatusCode::FORBIDDEN,
            PlacesError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            PlacesError::ExternalApiError(_) => StatusCode::BAD_GATEWAY,
            PlacesError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            PlacesError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}
