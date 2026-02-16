//! # Route Guards
//!
//! Provides a guard system for protecting routes with custom logic.
//! Guards are functions that run before the route handler and can reject
//! requests by returning an error.
//!
//! ## Example
//!
//! ```rust
//! use nexus_framework::prelude::*;
//!
//! // Define a guard function
//! async fn auth_guard(req: Request<Body>) -> GuardResult {
//!     if req.headers().get("Authorization").is_some() {
//!         Ok(req)
//!     } else {
//!         Err(AppError::unauthorized("Missing authorization header"))
//!     }
//! }
//!
//! // Use it on a route
//! #[route(GET, "/admin", guard = "auth_guard")]
//! async fn admin_panel() -> impl IntoResponse {
//!     "Welcome, admin!"
//! }
//! ```

use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;

use crate::error::AppError;

/// The result type for guard functions.
///
/// A guard receives the incoming request and must return either:
/// - `Ok(request)` to allow the request to proceed to the handler
/// - `Err(AppError)` to reject the request with an error response
pub type GuardResult = Result<Request<Body>, AppError>;

/// A guard function signature that can be used with the `#[route]` macro.
///
/// Guard functions are async functions that take a `Request<Body>` and return a `GuardResult`.
/// They are executed as Axum middleware layers on the specific route they protect.
///
/// # Example
///
/// ```rust
/// use nexus_framework::prelude::*;
///
/// async fn rate_limit_guard(req: Request<Body>) -> GuardResult {
///     // Check rate limit logic...
///     Ok(req)
/// }
/// ```
pub type GuardFn =
    fn(Request<Body>) -> std::pin::Pin<Box<dyn std::future::Future<Output = GuardResult> + Send>>;

/// Middleware adapter that wraps a guard function into an Axum middleware.
///
/// This is used internally by the `#[route]` macro when a `guard` parameter is specified.
/// You typically don't need to call this directly.
pub async fn guard_middleware<F, Fut>(
    req: Request<Body>,
    next: Next,
    guard_fn: F,
) -> impl IntoResponse
where
    F: FnOnce(Request<Body>) -> Fut,
    Fut: std::future::Future<Output = GuardResult>,
{
    match guard_fn(req).await {
        Ok(req) => next.run(req).await.into_response(),
        Err(err) => err.into_response(),
    }
}
