# Nexus Framework

[![CI](https://github.com/SentinelleAI/nexus-framework/actions/workflows/ci.yml/badge.svg)](https://github.com/SentinelleAI/nexus-framework/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE.md)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.2.0-green.svg)](Cargo.toml)

A lightweight, modular web framework for Rust built on [Axum](https://github.com/tokio-rs/axum) — designed for high-performance, scalable web applications and APIs.

---

## Features

| Feature | Description |
|---------|-------------|
| **Dependency Injection** | Auto-discovery and type-safe injection with inter-service dependencies |
| **Declarative API** | Simple attribute macros (`#[service]`, `#[controller]`, `#[route]`) |
| **Route Guards** | Protect routes with custom async guard functions |
| **Error Handling** | Centralized `AppError` with automatic JSON responses and structured details |
| **Configuration** | Layered config system: TOML files + environment variables |
| **Scheduled Jobs** | Cron-based job scheduling with `#[scheduled]` |
| **Request Logging** | Built-in colorized request/response middleware |
| **System Diagnostics** | Automatic system resource scanning on startup |
| **Custom Extractors** | `ValidatedJson<T>` for user-friendly deserialization errors |

---

## Quick Start

### 1. Add the dependency

```toml
[dependencies]
nexus_framework = { git = "https://github.com/SentinelleAI/nexus-framework.git" }
```

### 2. Create your application

```rust
use nexus_framework::prelude::*;

// Define a model
#[model]
pub struct User { id: u64, username: String }

// Define a service
#[service]
pub struct UserService;

#[service_impl]
impl UserService {
    pub fn new(_container: &DependencyContainer) -> Self { Self }
    pub fn find_user(&self, id: u64) -> User {
        User { id, username: format!("user_{}", id) }
    }
}

// Define a controller
pub struct UserController {
    user_service: Arc<UserService>,
}

#[controller(path = "/users")]
impl UserController {
    pub fn new(container: &DependencyContainer) -> Self {
        Self { user_service: container.get() }
    }

    #[route(GET, "/:id")]
    async fn get_user(
        State(state): State<Arc<Self>>,
        Path(id): Path<u64>,
    ) -> Result<Json<User>, AppError> {
        if id == 0 {
            return Err(AppError::bad_request("ID must be greater than 0"));
        }
        Ok(Json(state.user_service.find_user(id)))
    }

    #[route(DELETE, "/:id", guard = "auth_guard")]
    async fn delete_user(
        State(_state): State<Arc<Self>>,
        Path(id): Path<u64>,
    ) -> Result<Json<serde_json::Value>, AppError> {
        Ok(Json(serde_json::json!({ "deleted": id })))
    }
}

// Define a guard
async fn auth_guard(req: axum::http::Request<axum::body::Body>) -> GuardResult {
    if req.headers().get("Authorization").is_some() {
        Ok(req)
    } else {
        Err(AppError::unauthorized("Missing authorization header"))
    }
}

// Run the application
#[nexus_app(port = 8080)]
async fn main() {
}
```

### 3. Run

```bash
cargo run
```

---

## Documentation

| Guide | Description |
|-------|-------------|
| [Getting Started](docs/getting-started.md) | Installation and first project |
| [Dependency Injection](docs/guides/dependency-injection.md) | DI container and inter-service dependencies |
| [Controllers & Routes](docs/guides/controllers-and-routes.md) | Defining HTTP endpoints |
| [Error Handling](docs/guides/error-handling.md) | Centralized errors with `AppError` |
| [Configuration](docs/guides/configuration.md) | Layered TOML and environment config |
| [Route Guards](docs/guides/guards.md) | Protecting routes with custom logic |
| [Scheduled Jobs](docs/guides/scheduled-jobs.md) | Cron-based background job scheduling |
| [Macro Reference](docs/reference/macros.md) | Complete reference for all macros |

---

## Architecture

```
nexus_framework/src/
├── lib.rs              Module orchestrator and re-exports
├── container.rs        DependencyContainer (DI)
├── factory.rs          Service/Controller/Job factories
├── config.rs           Layered configuration (NexusConfig)
├── error.rs            AppError + ErrorResponse
├── guard.rs            Route guard system
├── extractors.rs       ValidatedJson<T> extractor
├── handlers.rs         Built-in /health and /ping
├── middleware.rs       Request logging middleware
├── diagnostics.rs      System resource scanner
├── tracing.rs          Custom log formatter
└── prelude.rs          Convenience re-exports

nexus_framework_macros/src/
├── lib.rs              Proc macro entrypoints
├── parsers.rs          Macro argument parsers
├── service.rs          #[service] + #[service_impl]
├── controller.rs       #[controller] + #[route] + #[scheduled]
├── model.rs            #[model]
└── app.rs              #[nexus_app]
```

---

## Configuration

Create a `nexus.toml` at the root of your project:

```toml
[server]
host = "0.0.0.0"
port = 8080

[app]
name = "my-app"
env = "development"

# Custom sections accessible via config.get()
[database]
url = "postgres://localhost/mydb"
```

Configuration is loaded in layers (each overrides the previous):
1. Default values
2. `nexus.toml`
3. `nexus.{env}.toml` (e.g., `nexus.production.toml`)
4. Environment variables with `NFW_` prefix (e.g., `NFW_SERVER__PORT=9090`)

---

## Error Handling

```rust
// Convenient constructors
AppError::bad_request("Invalid input")          // 400
AppError::unauthorized("Not authenticated")     // 401
AppError::forbidden("Access denied")            // 403
AppError::not_found("Resource not found")       // 404
AppError::conflict("Already exists")            // 409
AppError::unprocessable("Validation failed")    // 422
AppError::too_many_requests("Slow down")        // 429
AppError::internal("Something went wrong")      // 500
AppError::service_unavailable("Try later")      // 503
AppError::gateway_timeout("Upstream timeout")   // 504

// With structured details
AppError::bad_request("Validation failed")
    .with_details(serde_json::json!({
        "field": "email",
        "reason": "invalid format"
    }))
```

All errors serialize to JSON:

```json
{
  "status": 400,
  "error": "Bad Request",
  "message": "Validation failed",
  "details": { "field": "email", "reason": "invalid format" }
}
```

---

## Route Guards

```rust
async fn admin_guard(req: axum::http::Request<axum::body::Body>) -> GuardResult {
    if is_admin(&req) {
        Ok(req)
    } else {
        Err(AppError::forbidden("Admin access required"))
    }
}

#[route(POST, "/admin/action", guard = "admin_guard")]
async fn admin_action() -> impl IntoResponse {
    "Admin action executed"
}
```

---

## Scheduled Jobs

```rust
#[scheduled(cron = "0 */5 * * * *")]
async fn cleanup_job() {
    tracing::info!("Running cleanup every 5 minutes");
}
```

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the [MIT License](LICENSE.md).
