//! # Nexus Framework
//!
//! A lightweight, modular web application framework for Rust built on top of Axum.
//!
//! Nexus Framework provides a declarative API for building web applications with
//! dependency injection, auto-discovery of services and controllers, and built-in
//! tracing and logging capabilities.
//!
//! ## Key Features
//!
//! - **Dependency Injection**: Automatic service discovery and injection with inter-service dependencies
//! - **Declarative API**: Simple attribute macros for defining services, controllers, and routes
//! - **Tracing**: Built-in tracing with a customizable formatter
//! - **Auto-Discovery**: Services and controllers are automatically discovered and registered
//! - **Error Handling**: Centralized `AppError` type with automatic JSON error responses
//! - **Configuration**: Layered configuration system (TOML files and environment variables)
//! - **Route Guards**: Protect routes with custom guard functions
//! - **Scheduled Jobs**: Cron-based job scheduling with `#[scheduled]`
//!
//! ## Example
//!
//! ```rust
//! use nexus_framework::prelude::*;
//!
//! #[service(inject)]
//! pub struct UserService;
//!
//! #[service_impl]
//! impl UserService {
//!     pub fn new(_container: &DependencyContainer) -> Self { Self }
//!     pub fn find_user(&self, id: u64) -> String { format!("User {}", id) }
//! }
//!
//! pub struct UserController {
//!     user_service: Arc<UserService>,
//! }
//!
//! #[controller(path = "/users")]
//! impl UserController {
//!     pub fn new(container: &DependencyContainer) -> Self {
//!         Self { user_service: container.get() }
//!     }
//!
//!     #[route(GET, "/:id")]
//!     async fn get_user(
//!         State(state): State<Arc<Self>>,
//!         Path(id): Path<u64>,
//!     ) -> impl IntoResponse {
//!         Json(state.user_service.find_user(id))
//!     }
//! }
//!
//! #[nexus_app(port = 8080)]
//! async fn main() {}
//! ```
//!
//! ## Module Organization
//!
//! - [`tracing`]: Custom formatter for structured, colorized log output
//! - [`factory`]: Auto-discovery factories for services, controllers, and scheduled jobs
//! - [`container`]: Dependency injection container with type-safe service resolution
//! - [`handlers`]: Built-in utility HTTP handlers (health check, ping)
//! - [`diagnostics`]: System resource scanning and logging
//! - [`config`]: Layered configuration system via `NexusConfig`
//! - [`error`]: Centralized `AppError` type for consistent error responses
//! - [`guard`]: Route guard functions for route protection
//! - [`middleware`]: Request logging middleware
//! - [`extractors`]: Enhanced extractors with automatic error handling
//! - [`prelude`]: Common imports for using the framework

// Re-export inventory for use by generated code
pub use inventory;

// ─── Modules ─────────────────────────────────────────────────────────────────

pub mod config;
pub mod container;
pub mod diagnostics;
pub mod error;
pub mod extractors;
pub mod factory;
pub mod guard;
pub mod handlers;
pub mod middleware;
pub mod prelude;
pub mod tracing;

// ─── Public Re-exports ───────────────────────────────────────────────────────

// Re-export key types at the crate root for convenience and backward compatibility
pub use self::container::DependencyContainer;
pub use self::diagnostics::log_system_resources;
pub use self::factory::{ControllerFactory, ControllerInfo, ScheduledJobFactory, ServiceFactory};
pub use self::handlers::{health_handler, ping_handler};
pub use self::tracing::CustomFormatter;
