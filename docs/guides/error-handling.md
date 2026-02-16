# Error Handling

Nexus Framework provides a centralized `AppError` type that automatically converts to consistent JSON error responses.

## Using AppError

Return `AppError` from route handlers:

```rust
#[route(GET, "/:id")]
async fn get_user(Path(id): Path<u64>) -> Result<Json<User>, AppError> {
    if id == 0 {
        return Err(AppError::bad_request("ID must be greater than 0"));
    }
    // ...
}
```

## Available Constructors

| Constructor | HTTP Status |
|-------------|-------------|
| `AppError::bad_request(msg)` | 400 Bad Request |
| `AppError::unauthorized(msg)` | 401 Unauthorized |
| `AppError::forbidden(msg)` | 403 Forbidden |
| `AppError::not_found(msg)` | 404 Not Found |
| `AppError::conflict(msg)` | 409 Conflict |
| `AppError::unprocessable(msg)` | 422 Unprocessable Entity |
| `AppError::too_many_requests(msg)` | 429 Too Many Requests |
| `AppError::internal(msg)` | 500 Internal Server Error |
| `AppError::service_unavailable(msg)` | 503 Service Unavailable |
| `AppError::gateway_timeout(msg)` | 504 Gateway Timeout |
| `AppError::new(status, msg)` | Custom status code |

## Adding Details

Enrich errors with structured details using `with_details()`:

```rust
Err(AppError::bad_request("Validation failed")
    .with_details(serde_json::json!({
        "errors": [
            { "field": "email", "reason": "invalid format" },
            { "field": "age", "reason": "must be positive" }
        ]
    })))
```

Response:

```json
{
  "status": 400,
  "error": "Bad Request",
  "message": "Validation failed",
  "details": {
    "errors": [
      { "field": "email", "reason": "invalid format" },
      { "field": "age", "reason": "must be positive" }
    ]
  }
}
```

> The `details` field is only included when set. Otherwise, the JSON response omits it.

## Automatic Conversions

`AppError` provides automatic `From` implementations:

| Source Type | Maps To |
|-------------|---------|
| `std::io::Error` | 500 Internal Server Error |
| `serde_json::Error` | 400 Bad Request |
| `String` | 500 Internal Server Error |
| `&str` | 500 Internal Server Error |

```rust
// This automatically converts io::Error to AppError::internal
async fn handler() -> Result<String, AppError> {
    let content = std::fs::read_to_string("file.txt")?;
    Ok(content)
}
```

## ValidatedJson Extractor

For better deserialization error messages, use `ValidatedJson<T>` instead of `Json<T>`:

```rust
use nexus_framework::prelude::*;

#[route(POST, "/")]
async fn create_user(
    ValidatedJson(payload): ValidatedJson<CreateUser>,
) -> Result<Json<User>, AppError> {
    // If JSON parsing fails, a 422 error with a descriptive
    // message is returned automatically
    Ok(Json(create(payload)))
}
```
