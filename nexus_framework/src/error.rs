//! # Centralized Error Handling
//!
//! Provides a unified error type `AppError` that can be used across the application
//! to return consistent JSON error responses.
//!
//! ## Example
//!
//! ```rust
//! use nexus_framework::prelude::*;
//!
//! async fn handler() -> Result<Json<String>, AppError> {
//!     Err(AppError::not_found("User not found"))
//! }
//! ```

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// A structured error response returned as JSON.
#[derive(Debug, Serialize, Clone)]
pub struct ErrorResponse {
    /// HTTP status code as a number
    pub status: u16,
    /// Short error label (e.g., "Not Found", "Internal Server Error")
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Optional additional details about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// A unified application error type that automatically converts to an HTTP JSON response.
///
/// `AppError` wraps an HTTP status code and a message, and implements `IntoResponse`
/// so it can be returned directly from Axum handlers.
///
/// # Example
///
/// ```rust
/// use nexus_framework::error::AppError;
///
/// async fn get_user(id: u64) -> Result<String, AppError> {
///     if id == 0 {
///         return Err(AppError::bad_request("ID must be greater than 0"));
///     }
///     Ok(format!("User {}", id))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AppError {
    pub status: StatusCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl AppError {
    /// Creates a new `AppError` with the given status code and message.
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            details: None,
        }
    }

    /// Adds structured details to this error.
    ///
    /// # Example
    ///
    /// ```rust
    /// AppError::bad_request("Validation failed")
    ///     .with_details(serde_json::json!({
    ///         "field": "email",
    ///         "reason": "invalid format"
    ///     }))
    /// ```
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Creates a 400 Bad Request error.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    /// Creates a 401 Unauthorized error.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    /// Creates a 403 Forbidden error.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, message)
    }

    /// Creates a 404 Not Found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    /// Creates a 409 Conflict error.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }

    /// Creates a 422 Unprocessable Entity error.
    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }

    /// Creates a 429 Too Many Requests error.
    pub fn too_many_requests(message: impl Into<String>) -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, message)
    }

    /// Creates a 500 Internal Server Error.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    /// Creates a 503 Service Unavailable error.
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, message)
    }

    /// Creates a 504 Gateway Timeout error.
    pub fn gateway_timeout(message: impl Into<String>) -> Self {
        Self::new(StatusCode::GATEWAY_TIMEOUT, message)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.status, self.message)
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            status: self.status.as_u16(),
            error: self
                .status
                .canonical_reason()
                .unwrap_or("Unknown")
                .to_string(),
            message: self.message.clone(),
            details: self.details.clone(),
        };

        tracing::warn!(
            status = self.status.as_u16(),
            message = %self.message,
            "Application error returned"
        );

        (self.status, Json(body)).into_response()
    }
}

// Manual From implementations for common error types

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::internal(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::bad_request(err.to_string())
    }
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::internal(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::internal(msg.to_string())
    }
}
