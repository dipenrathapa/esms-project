//! Centralized error handling module
//! 
//! Provides unified error types and HTTP response mapping for the entire application.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;
use thiserror::Error;
use tracing::error;
use uuid::Uuid;

/// Application-wide error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Sensor data error: {0}")]
    SensorError(String),

    #[error("FHIR conversion error: {0}")]
    FhirError(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Bad request: {0}")]
    BadRequest(String),
}

/// Standardized error response format
#[derive(Serialize)]
pub struct ErrorResponse {
    /// Unique correlation ID for tracing
    pub correlation_id: String,
    /// Error type classification
    pub error_type: String,
    /// Human-readable error message (safe for clients)
    pub message: String,
    /// HTTP status code
    pub status_code: u16,
    /// Timestamp of the error
    pub timestamp: String,
}

impl ErrorResponse {
    pub fn new(error_type: &str, message: &str, status_code: StatusCode) -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            error_type: error_type.to_string(),
            message: message.to_string(),
            status_code: status_code.as_u16(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type, message) = match self {
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.as_str())
            }
            AppError::SensorError(msg) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "SENSOR_ERROR", msg.as_str())
            }
            AppError::FhirError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "FHIR_ERROR", msg.as_str())
            }
            AppError::InternalError(msg) => {
                // Log internal errors but return safe message to client
                error!(error = %msg, "Internal server error occurred");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "An internal error occurred")
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", msg.as_str())
            }
            AppError::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.as_str())
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.as_str())
            }
        };

        let error_response = ErrorResponse::new(error_type, message, status);
        
        error!(
            correlation_id = %error_response.correlation_id,
            error_type = %error_type,
            status_code = %status.as_u16(),
            "Error response generated"
        );

        HttpResponse::build(status).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::SensorError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::FhirError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }
}

/// Result type alias for application errors
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_status() {
        let error = AppError::ValidationError("Invalid temperature".to_string());
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_not_found_error_status() {
        let error = AppError::NotFound("Resource not found".to_string());
        assert_eq!(error.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_error_response_format() {
        let response = ErrorResponse::new("TEST_ERROR", "Test message", StatusCode::BAD_REQUEST);
        assert!(!response.correlation_id.is_empty());
        assert_eq!(response.error_type, "TEST_ERROR");
        assert_eq!(response.message, "Test message");
        assert_eq!(response.status_code, 400);
    }
}
