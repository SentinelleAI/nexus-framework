//! # Dependency Injection Container
//!
//! Provides a type-safe dependency injection container for managing services
//! with inter-service dependency support.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::factory::ServiceFactory;

/// A dependency injection container for managing services with inter-service dependency support.
///
/// The `DependencyContainer` is responsible for:
/// 1. Discovering and instantiating all services registered with the `#[service]` macro
/// 2. Storing service instances for later retrieval
/// 3. Providing type-safe access to services through the `get<T>()` method
/// 4. Supporting inter-service dependencies (services can depend on other services)
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
    /// 2. Calls each factory function to create a service instance, passing the container
    ///    so services can resolve their dependencies
    /// 3. Stores the service instance in the container, indexed by its type ID
    ///
    /// Services are instantiated in registration order. If service A depends on service B,
    /// ensure B is registered before A (typically by defining B first in your code).
    ///
    /// # Returns
    ///
    /// A new `DependencyContainer` with all discovered services instantiated and ready for use.
    pub fn build() -> Self {
        let mut container = Self {
            map: HashMap::new(),
        };
        tracing::info!("🧩 Discovering and instantiating services...");
        for service_factory in inventory::iter::<ServiceFactory> {
            let service = service_factory.name;
            tracing::info!(service, "   ⚙️ Instantiated");
            let service_arc = (service_factory.factory)(&container);
            container.map.insert((*service_arc).type_id(), service_arc);
        }
        tracing::info!("✅ Service container built");
        container
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
            .unwrap_or_else(|| {
                panic!(
                    "Service of type {} not found in container",
                    std::any::type_name::<T>()
                )
            })
    }

    /// Tries to get a service from the container by its type, returning `None` if not found.
    ///
    /// This is a non-panicking alternative to `get()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// if let Some(cache) = container.try_get::<CacheService>() {
    ///     // Use cache
    /// }
    /// ```
    pub fn try_get<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|service| service.clone().downcast::<T>().ok())
    }

    /// Checks if a service of type `T` is registered in the container.
    ///
    /// # Example
    ///
    /// ```rust
    /// if container.has::<CacheService>() {
    ///     // Cache is available
    /// }
    /// ```
    pub fn has<T: 'static + Send + Sync>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    /// Returns the number of services registered in the container.
    pub fn service_count(&self) -> usize {
        self.map.len()
    }
}

impl std::fmt::Debug for DependencyContainer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DependencyContainer")
            .field("service_count", &self.map.len())
            .finish()
    }
}
