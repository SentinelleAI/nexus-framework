# Dependency Injection

Nexus Framework provides a built-in dependency injection container that automatically discovers, instantiates, and manages services.

## Defining a Service

Use the `#[service]` attribute to mark a struct as a service:

```rust
#[service]
pub struct UserService;

#[service_impl]
impl UserService {
    pub fn new(_container: &DependencyContainer) -> Self {
        Self
    }

    pub fn find_user(&self, id: u64) -> String {
        format!("User {}", id)
    }
}
```

The `#[service]` macro:
- Adds `#[derive(Clone)]` to the struct
- Registers it with the framework's inventory system for auto-discovery

The `#[service_impl]` macro:
- Wraps each method (except `new`) with a tracing span for observability
- Makes all methods public

## Inter-Service Dependencies

Services can depend on other services by resolving them from the `DependencyContainer` in `new()`:

```rust
#[service]
pub struct NotificationService {
    user_service: Arc<UserService>,
}

#[service_impl]
impl NotificationService {
    pub fn new(container: &DependencyContainer) -> Self {
        Self {
            user_service: container.get(),
        }
    }

    pub fn notify_user(&self, id: u64, message: &str) {
        let user = self.user_service.find_user(id);
        tracing::info!("Notifying {}: {}", user, message);
    }
}
```

> **Note:** Services are instantiated in registration order. If service A depends on service B, define B before A in your code.

## DependencyContainer API

| Method | Description |
|--------|-------------|
| `get::<T>()` | Returns `Arc<T>`. Panics if not found. |
| `try_get::<T>()` | Returns `Option<Arc<T>>`. |
| `has::<T>()` | Returns `bool` — checks if a service is registered. |
| `service_count()` | Returns the number of registered services. |

### Example

```rust
// Panics if not found
let user_service: Arc<UserService> = container.get();

// Safe alternative
if let Some(cache) = container.try_get::<CacheService>() {
    // Use cache
}

// Check existence
if container.has::<CacheService>() {
    // ...
}
```

## Using Services in Controllers

Controllers resolve services from the container in their `new()` method:

```rust
pub struct UserController {
    user_service: Arc<UserService>,
}

#[controller(path = "/users")]
impl UserController {
    pub fn new(container: &DependencyContainer) -> Self {
        Self {
            user_service: container.get(),
        }
    }

    #[route(GET, "/:id")]
    async fn get_user(
        State(state): State<Arc<Self>>,
        Path(id): Path<u64>,
    ) -> Json<String> {
        Json(state.user_service.find_user(id))
    }
}
```
