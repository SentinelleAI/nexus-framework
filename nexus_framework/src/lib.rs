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
//! - **Dependency Injection**: Automatic service discovery and injection
//! - **Declarative API**: Simple attribute macros for defining services, controllers, and routes
//! - **Tracing**: Built-in tracing with a customizable formatter
//! - **Auto-Discovery**: Services and controllers are automatically discovered and registered
//! 
//! ## Example
//! 
//! ```rust
//! use nexus_framework::prelude::*;
//! 
//! // Define a service
//! #[service]
//! pub struct UserService;
//! 
//! #[service_impl]
//! impl UserService {
//!     pub fn new() -> Self {
//!         UserService
//!     }
//!     
//!     pub fn find_user(&self, id: String) -> String {
//!         format!("User {}", id)
//!     }
//! }
//! 
//! // Define a controller
//! pub struct UserController {
//!     user_service: Arc<UserService>,
//! }
//! 
//! #[controller(path = "/users")]
//! impl UserController {
//!     pub fn new(container: &DependencyContainer) -> Self {
//!         Self {
//!             user_service: container.get(),
//!         }
//!     }
//!     
//!     #[route(GET, "/:id")]
//!     async fn get_user(
//!         Path(id): Path<String>,
//!         State(state): State<Self>,
//!     ) -> impl IntoResponse {
//!         Json(state.user_service.find_user(id))
//!     }
//! }
//! 
//! // Define the application
//! #[nexus_app(port = 8080)]
//! async fn main() {
//!     // The framework handles everything else!
//! }
//! ```
//! 
//! ## Module Organization
//! 
//! - **CustomFormatter**: A custom formatter for tracing output
//! - **Service and Controller Factories**: Auto-discovery mechanisms for services and controllers
//! - **Dependency Injection Container**: A simple DI container for service management
//! - **Utility Handlers**: Built-in handlers for health checks and ping
//! - **Prelude**: Common imports for using the framework

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
    future::Future,
    pin::Pin,
};
use tracing_subscriber::fmt::{format::Writer, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing::Level;
use colored::Colorize;
use sysinfo::{System, SystemExt, CpuExt, DiskExt};
use tokio;

pub use inventory;

pub mod middleware;

//--------------------------------------------------------------------------------------------------
// Tracing and Logging
//--------------------------------------------------------------------------------------------------

/// A custom formatter for tracing events that provides colorized, structured output.
///
/// This formatter enhances log readability by:
/// - Adding timestamps in ISO 8601 format with microsecond precision
/// - Colorizing output based on log level (when ANSI colors are supported)
/// - Including thread names for better concurrent execution tracking
/// - Displaying the full span context hierarchy
///
/// # Format
///
/// The output format is:
/// ```text
/// TIMESTAMP LEVEL [THREAD_NAME] [SPAN1] [SPAN2] ... : MESSAGE
/// ```
///
/// # Example Output
///
/// ```text
/// 2023-05-15 14:32:45.123456Z - INFO  [main] [http_server] [request]: Processing request from 192.168.1.1
/// ```
///
/// # Colors
///
/// When ANSI colors are supported:
/// - ERROR: Bold Red
/// - WARN: Bold Yellow
/// - INFO: Green
/// - DEBUG: Blue
/// - TRACE: Purple
/// - Thread names: Cyan
/// - Span names: Bold Blue
/// - Timestamps: Dimmed
pub struct CustomFormatter;

impl<S, N> FormatEvent<S, N> for CustomFormatter
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    /// Formats a tracing event according to the custom format.
    ///
    /// This method is called by the tracing subscriber for each event that is emitted.
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        // Check if the output supports ANSI colors
        let use_colors = writer.has_ansi_escapes();

        // Format the timestamp (ISO 8601 with microsecond precision)
        let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f %:z -").to_string();
        let time_colored = if use_colors { time.dimmed() } else { time.normal() };

        // Format the log level with appropriate colors
        let level = event.metadata().level();
        let level_colored = if use_colors {
            match *level {
                Level::ERROR => "ERROR".red().bold(),
                Level::WARN => "WARN ".yellow().bold(),
                Level::INFO => "INFO ".green(),
                Level::DEBUG => "DEBUG".blue(),
                Level::TRACE => "TRACE".purple(),
            }
        } else {
            level.to_string().normal()
        };

        // Write the basic log prefix: timestamp, level, and thread name
        write!(writer, "{} {}", time_colored, level_colored)?;

        // Add span context if available
        if let Some(span_ref) = ctx.lookup_current() {
            let id = span_ref.id();
            if let Some(scope) = ctx.span_scope(&id) {
                // Iterate through all spans in the current scope, from root to leaf
                for span in scope.from_root() {
                    let meta = span.metadata();
                    let name = if use_colors { meta.name().blue().bold() } else { meta.name().normal() };
                    write!(writer, " [{}]", name)?;
                }
            }
        }

        // Add separator before the actual message
        write!(writer, ": ")?;

        // Format the event fields (the actual log message)
        ctx.format_fields(writer.by_ref(), event)?;

        // End with a newline
        writeln!(writer)
    }
}

//--------------------------------------------------------------------------------------------------
// Service and Controller Factories for Auto-Discovery
//--------------------------------------------------------------------------------------------------

pub struct ScheduledJobFactory {
    pub name: &'static str,
    pub cron: &'static str,
    pub job: fn() -> Pin<Box<dyn Future<Output = ()> + Send>>,
}

/// Factory for creating and registering services with the framework.
///
/// This struct is used by the `#[service]` macro to register services with the framework's
/// inventory system. When the application starts, all registered services are instantiated
/// and added to the dependency container.
///
/// # Fields
///
/// * `name` - The name of the service (derived from the struct name)
/// * `factory` - A function that creates an instance of the service
///
/// # Example
///
/// This is typically used through the `#[service]` macro:
///
/// ```rust
/// #[service]
/// pub struct UserService;
///
/// impl UserService {
///     pub fn new() -> Self {
///         UserService
///     }
/// }
/// ```
///
/// The macro generates code equivalent to:
///
/// ```rust
/// inventory::submit! {
///     ServiceFactory {
///         name: "UserService",
///         factory: || Arc::new(UserService::new()) as Arc<dyn Any + Send + Sync>,
///     }
/// }
/// ```
pub struct ServiceFactory {
    /// The name of the service (derived from the struct name)
    pub name: &'static str,

    /// A function that creates an instance of the service
    pub factory: fn() -> Arc<dyn Any + Send + Sync>,
}

/// Factory for creating and registering controllers with the framework.
///
/// This struct is used by the `#[controller]` macro to register controllers with the framework's
/// inventory system. When the application starts, all registered controllers are instantiated
/// and their routes are added to the application router.
///
/// # Fields
///
/// * `name` - The name of the controller (derived from the struct name)
/// * `routes` - A list of routes defined in the controller (format: "METHOD /path")
/// * `factory` - A function that creates an instance of the controller and returns its router
///
/// # Example
///
/// This is typically used through the `#[controller]` macro:
///
/// ```rust
/// #[controller(path = "/users")]
/// impl UserController {
///     pub fn new(container: &DependencyContainer) -> Self {
///         Self {
///             user_service: container.get(),
///         }
///     }
///     
///     #[route(GET, "/:id")]
///     async fn get_user(
///         Path(id): Path<String>,
///         State(state): State<Self>,
///     ) -> impl IntoResponse {
///         Json(state.user_service.find_user(id))
///     }
/// }
/// ```
pub struct ControllerFactory {
    /// The name of the controller (derived from the struct name)
    pub name: &'static str,

    /// A list of routes defined in the controller (format: "METHOD /path")
    pub routes: &'static [&'static str],

    /// A function that creates an instance of the controller and returns its router
    pub factory: fn(container: &DependencyContainer) -> Box<axum::Router>,
}

/// Metadata about a controller that can be accessed at runtime.
///
/// This struct is attached to the router as an extension, allowing middleware
/// to access information about the controller that handled a request.
#[derive(Clone)]
pub struct ControllerInfo {
    /// The name of the controller (derived from the struct name)
    pub name: &'static str,
}

// Register the factories with the inventory system for auto-discovery
inventory::collect!(ServiceFactory);
inventory::collect!(ControllerFactory);
inventory::collect!(ScheduledJobFactory);

//--------------------------------------------------------------------------------------------------
// Dependency Injection Container
//--------------------------------------------------------------------------------------------------

/// A simple dependency injection container for managing services.
///
/// The `DependencyContainer` is responsible for:
/// 1. Discovering and instantiating all services registered with the `#[service]` macro
/// 2. Storing service instances for later retrieval
/// 3. Providing type-safe access to services through the `get<T>()` method
///
/// # Example
///
/// ```rust
/// // Build the container (typically done by the framework)
/// let container = DependencyContainer::build();
///
/// // Get a service from the container
/// let user_service: Arc<UserService> = container.get();
///
/// // Use the service
/// let user = user_service.find_user("123");
/// ```
#[derive(Clone)]
pub struct DependencyContainer {
    /// Internal storage for service instances, indexed by their type ID
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl DependencyContainer {
    /// Builds a new dependency container by discovering and instantiating all registered services.
    ///
    /// This method:
    /// 1. Iterates through all `ServiceFactory` instances registered with the inventory system
    /// 2. Calls each factory function to create a service instance
    /// 3. Stores the service instance in the container, indexed by its type ID
    ///
    /// # Returns
    ///
    /// A new `DependencyContainer` with all discovered services instantiated and ready for use.
    ///
    /// # Example
    ///
    /// ```rust
    /// let container = DependencyContainer::build();
    /// ```
    pub fn build() -> Self {
        let mut map = HashMap::new();
        tracing::info!("🧩 Discovering and instantiating services...");
        for service_factory in inventory::iter::<ServiceFactory> {
            let service = service_factory.name;
            tracing::info!(service, "   ⚙️ Instantiated");
            let service_arc = (service_factory.factory)();
            map.insert((*service_arc).type_id(), service_arc);
        }
        tracing::info!("✅ Service container built");
        Self { map }
    }

    /// Gets a service from the container by its type.
    ///
    /// This method provides type-safe access to services. It uses Rust's type system
    /// to find the correct service instance based on the requested type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of service to retrieve. Must be `'static + Send + Sync`.
    ///
    /// # Returns
    ///
    /// An `Arc<T>` containing the requested service.
    ///
    /// # Panics
    ///
    /// Panics if no service of type `T` is found in the container. This typically means
    /// that the service was not properly registered with the `#[service]` macro.
    ///
    /// # Example
    ///
    /// ```rust
    /// let user_service: Arc<UserService> = container.get();
    /// ```
    pub fn get<T: 'static + Send + Sync>(&self) -> Arc<T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|service| service.clone().downcast::<T>().ok())
            .unwrap_or_else(|| panic!("Service of type {} not found in container", std::any::type_name::<T>()))
    }
}

//--------------------------------------------------------------------------------------------------
// Built-in Utility Handlers
//--------------------------------------------------------------------------------------------------

/// A health check endpoint handler that returns a JSON response with status "ok".
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
///
/// # Example
///
/// ```
/// // This is automatically added by the framework, but can be called manually:
/// let response = health_handler().await;
/// assert_eq!(response.0, axum::http::StatusCode::OK);
/// ```
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
///
/// # Example
///
/// ```
/// // This is automatically added by the framework, but can be called manually:
/// let response = ping_handler().await;
/// assert_eq!(response, "pong");
/// ```
pub async fn ping_handler() -> &'static str {
    "pong"
}

/// Scans and logs information about available system resources.
///
/// This function gathers and logs detailed information about:
/// - CPU (cores, threads, usage)
/// - Memory (total, available, used)
/// - Disks (total space, available space)
/// - Network interfaces
/// - Operating system details
///
/// It's called during application startup to provide visibility into the
/// system resources available to the application.
pub fn log_system_resources() {
    tracing::info!("ℹ️ System resource scan initiated in background; details will appear shortly.");
    tokio::task::spawn_blocking(|| {
        let span = tracing::info_span!("nfw-init");
        let _enter = span.enter();
        let span = tracing::span!(Level::INFO, "sysdiag");
        let _enter = span.enter();
        // Initialize system information
        let sys = System::new_all();

        // Log OS information
        tracing::info!(
            "🖥️ Operating System: {} {}",
            sys.name().unwrap_or_else(|| "Unknown".to_string()),
            sys.os_version().unwrap_or_else(|| "Unknown".to_string())
        );
        tracing::info!(
            "🖥️ Kernel Version: {}",
            sys.kernel_version().unwrap_or_else(|| "Unknown".to_string())
        );
        tracing::info!(
            "🖥️ Host Name: {}",
            sys.host_name().unwrap_or_else(|| "Unknown".to_string())
        );

        // Log CPU information
        let cpu_count = sys.cpus().len();
        let physical_core_count = sys.physical_core_count().unwrap_or(0);
        tracing::info!(
            "🧠 Physical cores: {}, Logical cores: {}",
            physical_core_count,
            cpu_count
        );

        // Log CPU details
        if let Some(cpu) = sys.cpus().first() {
            tracing::info!(
                "🧠 CPU Frequency: {} MHz",
                cpu.frequency()
            );
        } else {
            tracing::warn!("🧠 No CPU information available");
        }

        // Log memory information
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let available_memory = sys.available_memory();

        tracing::info!(
            "💾 Total: {:.2} GB, Used: {:.2} GB, Available: {:.2} GB",
            total_memory as f64 / 1_073_741_824.0, // Convert to GB
            used_memory as f64 / 1_073_741_824.0,
            available_memory as f64 / 1_073_741_824.0
        );

        // Log disk information
        tracing::info!("💽 Scanning available disks:");
        for disk in sys.disks() {
            let total_space = disk.total_space();
            let available_space = disk.available_space();
            let used_space = total_space - available_space;
            let usage_percent = if total_space > 0 {
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            };

            tracing::info!(
                "💽 {}: {:.2} GB total, {:.2} GB used ({:.1}%), FS: {}",
                disk.mount_point().to_string_lossy(),
                total_space as f64 / 1_073_741_824.0,
                used_space as f64 / 1_073_741_824.0,
                usage_percent,
                String::from_utf8_lossy(disk.file_system())
            );
        }
    });
}

//--------------------------------------------------------------------------------------------------
// Prelude - Common Imports
//--------------------------------------------------------------------------------------------------

/// A collection of commonly used imports for working with the Nexus Framework.
///
/// The prelude module re-exports the most frequently used types, traits, and functions
/// from the framework and its dependencies. By importing this module with a wildcard import
/// (`use nexus_framework::prelude::*`), you can reduce the number of import statements
/// needed in your application code.
///
/// # Included Items
///
/// ## Framework Macros
/// - `controller`: For defining controllers with routes
/// - `nexus_app`: For setting up the main application
/// - `route`: For defining HTTP endpoints
/// - `service`: For defining injectable services
/// - `service_impl`: For adding tracing to service methods
/// - `model`: For defining data models
/// - `scheduled`: For defining scheduled jobs
///
/// ## Web Framework (Axum)
/// - Core Axum types and traits
/// - Extractors (Path, Query, State)
/// - Response types and traits
/// - HTTP status codes
///
/// ## Async Runtime (Tokio)
/// - Tokio runtime and utilities
///
/// ## Serialization (Serde)
/// - Serialize and Deserialize traits
///
/// ## Tracing and Logging
/// - Tracing macros and types
/// - Tracing subscriber configuration
///
/// ## Utilities
/// - Arc for shared ownership
/// - Inventory for type registration
/// - Chrono for date/time handling
/// - Colored for terminal coloring
///
/// # Example
///
/// ```rust
/// use nexus_framework::prelude::*;
///
/// // Now you can use all the imported items without additional imports
/// #[service]
/// pub struct MyService;
///
/// #[controller(path = "/api")]
/// impl MyController {
///     // ...
/// }
/// ```
pub mod prelude {
    // Framework macros
    pub use nexus_framework_macros::{controller, nexus_app, route, service, service_impl, model, scheduled};

    // Framework types
    pub use super::DependencyContainer;

    // Web framework (Axum)
    pub use axum;
    pub use axum::{
        extract::{Path, Query, State},
        http::StatusCode,
        response::{IntoResponse, Response},
        Json, Router,
        Extension,
    };

    // Middleware
    pub use tower_http;
    pub use tower_http::trace::TraceLayer;

    // Async runtime
    pub use tokio;

    // Serialization
    pub use serde;
    pub use serde::{Deserialize, Serialize};

    // Tracing and logging
    pub use tracing;
    pub use tracing_subscriber;

    // Utilities
    pub use std::sync::Arc;
    pub use inventory;
    pub use chrono;
    pub use colored::Colorize;
    pub use uuid;
    pub use sysinfo;
    pub use http_body_util;
    pub use hyper;
    pub use tokio_cron_scheduler;
}
