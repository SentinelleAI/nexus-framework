//! # Built-in Utility Handlers
//!
//! Provides built-in HTTP handlers that are automatically registered by the framework,
//! such as health check and ping endpoints.

/// A health check endpoint handler that returns a JSON response with the status "ok".
///
/// This handler is automatically added to the application at the `/health` path
/// by the `#[nexus_app]` macro. It can be used by load balancers, monitoring tools,
/// or other services to check if the application is running properly.
///
/// # Returns
///
/// Returns a tuple containing:
/// - An HTTP 200 OK status code
/// - A JSON response with `{ "status": "ok" }`
pub async fn health_handler() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    let health_status = serde_json::json!({ "status": "ok" });
    (axum::http::StatusCode::OK, axum::Json(health_status))
}

/// A simple ping endpoint handler that returns "pong".
///
/// This handler is automatically added to the application at the `/ping` path
/// by the `#[nexus_app]` macro. It can be used for simple connectivity tests
/// or as a lightweight health check.
///
/// # Returns
///
/// Returns the string "pong".
pub async fn ping_handler() -> &'static str {
    "pong"
}
