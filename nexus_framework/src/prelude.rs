//! # Prelude
//!
//! A collection of commonly used imports for working with the Nexus Framework.
//!
//! The prelude module re-exports the most frequently used types, traits, and functions
//! from the framework and its dependencies. By importing this module with a wildcard import
//! (`use nexus_framework::prelude::*`), you can reduce the number of import statements
//! needed in your application code.
//!
//! # Included Items
//!
//! ## Framework Macros
//! - `controller`: For defining controllers with routes
//! - `nexus_app`: For setting up the main application
//! - `route`: For defining HTTP endpoints
//! - `service`: For defining injectable services
//! - `service_impl`: For adding tracing to service methods
//! - `model`: For defining data models
//! - `scheduled`: For defining scheduled jobs
//!
//! ## Error Handling
//! - `AppError`: Centralized error type with JSON responses
//! - `GuardResult`: Result type for route guards
//!
//! ## Configuration
//! - `NexusConfig`: Layered configuration system
//!
//! ## Web Framework (Axum)
//! - Core Axum types and traits
//! - Extractors (Path, Query, State)
//! - Response types and traits
//! - HTTP status codes
//!
//! ## Async Runtime (Tokio)
//! - Tokio runtime and utilities
//!
//! ## Serialization (Serde)
//! - Serialize and Deserialize traits
//!
//! ## Tracing and Logging
//! - Tracing macros and types
//! - Tracing subscriber configuration
//!
//! ## Utilities
//! - Arc for shared ownership
//! - Inventory for type registration
//! - Chrono for date/time handling
//! - Colored for terminal coloring

// Framework macros
pub use nexus_framework_macros::{
    controller, model, nexus_app, route, scheduled, service, service_impl,
};

// Framework types
pub use crate::container::DependencyContainer;

// Error handling
pub use crate::error::{AppError, ErrorResponse};

// Configuration
pub use crate::config::NexusConfig;

// Route guards
pub use crate::guard::GuardResult;

// Extractors
pub use crate::extractors::ValidatedJson;

// Web framework (Axum)
pub use axum;
pub use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json, Router,
};

// Middleware
pub use tower_http;
pub use tower_http::trace::TraceLayer;

// Async runtime
pub use tokio;

// Serialization
pub use serde;
pub use serde::{Deserialize, Serialize};
pub use serde_json;

// Tracing and logging
pub use tracing;
pub use tracing_subscriber;

// Utilities
pub use chrono;
pub use colored::Colorize;
pub use http_body_util;
pub use hyper;
pub use inventory;
pub use std::sync::Arc;
pub use sysinfo;
pub use tokio_cron_scheduler;
pub use uuid;
