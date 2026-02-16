# Route Guards

Guards are async functions that run before a route handler and can accept or reject the request.

## Defining a Guard

A guard function takes a `Request<Body>` and returns a `GuardResult`:

```rust
use nexus_framework::prelude::*;

async fn auth_guard(req: axum::http::Request<axum::body::Body>) -> GuardResult {
    if req.headers().get("Authorization").is_some() {
        Ok(req)  // Allow the request
    } else {
        Err(AppError::unauthorized("Missing authorization header"))  // Reject
    }
}
```

## Using a Guard on a Route

Specify the guard with the `guard` parameter in `#[route]`:

```rust
#[controller(path = "/admin")]
impl AdminController {
    // ...

    #[route(GET, "/dashboard", guard = "auth_guard")]
    async fn dashboard(State(state): State<Arc<Self>>) -> impl IntoResponse {
        "Welcome to the admin dashboard"
    }

    #[route(DELETE, "/users/:id", guard = "admin_guard")]
    async fn delete_user(
        State(state): State<Arc<Self>>,
        Path(id): Path<u64>,
    ) -> Result<StatusCode, AppError> {
        // Only accessible if admin_guard passes
        Ok(StatusCode::NO_CONTENT)
    }
}
```

## Common Guard Patterns

### Role-Based Access

```rust
async fn admin_guard(req: axum::http::Request<axum::body::Body>) -> GuardResult {
    let role = req.headers()
        .get("X-User-Role")
        .and_then(|v| v.to_str().ok());

    match role {
        Some("admin") => Ok(req),
        _ => Err(AppError::forbidden("Admin access required")),
    }
}
```

### API Key Authentication

```rust
async fn api_key_guard(req: axum::http::Request<axum::body::Body>) -> GuardResult {
    let api_key = req.headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if key == std::env::var("API_KEY").unwrap_or_default() => Ok(req),
        _ => Err(AppError::unauthorized("Invalid or missing API key")),
    }
}
```

## How Guards Work

Guards are implemented as Axum middleware layers on the specific route they protect. The framework wraps your guard function using `guard_middleware`, which:

1. Calls your guard function with the request
2. If `Ok(req)` — passes the request to the route handler
3. If `Err(AppError)` — returns the error as a JSON response
