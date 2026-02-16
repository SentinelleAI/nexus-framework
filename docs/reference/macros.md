# Macro Reference

Complete reference for all procedural macros provided by the Nexus Framework.

---

## `#[service]`

**Applies to:** Struct definition

Marks a struct as an auto-discoverable service.

**What it does:**
- Adds `#[derive(Clone)]`
- Registers the struct with the framework's inventory system
- The struct must have a `new(container: &DependencyContainer) -> Self` method

**Usage:**
```rust
#[service]
pub struct MyService;
```

---

## `#[service_impl]`

**Applies to:** `impl` block

Adds automatic tracing spans to service methods.

**What it does:**
- Wraps each method (except `new`) with a `tracing::info_span!`
- Makes all methods `pub`

**Usage:**
```rust
#[service_impl]
impl MyService {
    pub fn new(_container: &DependencyContainer) -> Self { Self }

    // This gets a tracing span automatically
    pub fn do_something(&self) -> String {
        "result".to_string()
    }
}
```

---

## `#[controller(path = "...")]`

**Applies to:** `impl` block

Defines an HTTP controller with a base path.

**Parameters:**

| Name | Required | Description |
|------|----------|-------------|
| `path` | Yes | Base path for all routes (e.g., `"/users"`) |

**What it does:**
- Processes all methods marked with `#[route]`
- Generates an `into_router()` method
- Registers the controller with the inventory system

**Usage:**
```rust
pub struct UserController { /* fields */ }

#[controller(path = "/users")]
impl UserController {
    pub fn new(container: &DependencyContainer) -> Self { /* ... */ }

    #[route(GET, "/:id")]
    async fn get_user(/* ... */) -> impl IntoResponse { /* ... */ }
}
```

---

## `#[route(METHOD, "path", guard = "...")]`

**Applies to:** Method inside a `#[controller]` impl block

Defines an HTTP route handler.

**Parameters:**

| Position | Name | Required | Description |
|----------|------|----------|-------------|
| 1 | Method | Yes | HTTP method: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS` |
| 2 | Path | Yes | Route path relative to controller base path |
| named | `guard` | No | Guard function name (string literal) |

**Usage:**
```rust
#[route(GET, "/")]
async fn list(State(state): State<Arc<Self>>) -> Json<Vec<Item>> { /* ... */ }

#[route(POST, "/", guard = "auth_guard")]
async fn create(State(state): State<Arc<Self>>, Json(body): Json<NewItem>) -> StatusCode { /* ... */ }
```

---

## `#[model]`

**Applies to:** Struct definition

Marks a struct as a data model with serialization.

**What it does:**
- Adds `#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]`
- Configures serde to use the framework's serde re-export

**Usage:**
```rust
#[model]
pub struct User {
    pub id: u64,
    pub username: String,
    pub email: String,
}
```

---

## `#[scheduled(cron = "...")]`

**Applies to:** Async function

Defines a cron-based scheduled job.

**Parameters:**

| Name | Required | Description |
|------|----------|-------------|
| `cron` | Yes | Cron expression (6 fields: sec min hour day month weekday) |

**Usage:**
```rust
#[scheduled(cron = "0 0 * * * *")]
async fn hourly_job() {
    tracing::info!("Running every hour");
}
```

---

## `#[nexus_app(port = ...)]`

**Applies to:** `async fn main()`

Sets up the full application bootstrap.

**Parameters:**

| Name | Required | Description |
|------|----------|-------------|
| `port` | No | Port override (takes priority over config) |

**What it does:**
1. Loads configuration from `nexus.toml` and environment
2. Configures tracing with `CustomFormatter`
3. Logs system diagnostics
4. Builds the `DependencyContainer` with all discovered services
5. Discovers and registers all controllers and routes
6. Adds built-in `/ping` and `/health` routes
7. Adds request logging middleware
8. Starts all scheduled jobs
9. Binds the HTTP server with graceful shutdown

**Usage:**
```rust
#[nexus_app(port = 8080)]
async fn main() {
    // Optional: custom initialization code runs before the server starts
}
```
