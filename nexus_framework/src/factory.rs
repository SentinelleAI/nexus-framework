//! # Service, Controller, and Scheduled Job Factories
//!
//! Provides factory structs used by the procedural macros to register services,
//! controllers, and scheduled jobs with the framework's inventory system.

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::container::DependencyContainer;

/// Factory for creating and registering scheduled jobs with the framework.
///
/// This struct is used by the `#[scheduled]` macro to register jobs with the framework's
/// inventory system. When the application starts, all registered jobs are added to the
/// cron scheduler.
///
/// # Fields
///
/// * `name` - The name of the job (derived from the function name)
/// * `cron` - The cron expression for scheduling
/// * `job` - A function that returns a pinned boxed future to execute
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
/// Services can depend on other services through the `DependencyContainer`. The factory
/// receives the container so that services can resolve their dependencies during construction.
///
/// # Fields
///
/// * `name` - The name of the service (derived from the struct name)
/// * `factory` - A function that creates an instance of the service, receiving the container
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
pub struct ServiceFactory {
    /// The name of the service (derived from the struct name)
    pub name: &'static str,

    /// A function that creates an instance of the service.
    /// Receives the `DependencyContainer` to allow resolving inter-service dependencies.
    pub factory: fn(&DependencyContainer) -> Arc<dyn Any + Send + Sync>,
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
///         State(state): State<Arc<Self>>,
///         Path(id): Path<u64>,
///     ) -> Json<User> {
///         // Implementation...
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
