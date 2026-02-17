# Controllers & Routes

Controllers are the HTTP layer of your Nexus application. They define endpoints, handle requests, and return responses.

## Defining a Controller

A controller is a struct with an impl block annotated with `#[controller]`:

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

    #[route(GET, "/")]
    async fn list_users(
        State(state): State<Arc<Self>>,
    ) -> Json<Vec<User>> {
        Json(state.user_service.list_all())
    }

    #[route(GET, "/:id")]
    async fn get_user(
        State(state): State<Arc<Self>>,
        Path(id): Path<u64>,
    ) -> Result<Json<User>, AppError> {
        state.user_service.find_by_id(id)
            .map(Json)
            .ok_or_else(|| AppError::not_found("User not found"))
    }

    #[route(POST, "/")]
    async fn create_user(
        State(state): State<Arc<Self>>,
        Json(payload): Json<CreateUserPayload>,
    ) -> Result<(StatusCode, Json<User>), AppError> {
        let user = state.user_service.create(payload)?;
        Ok((StatusCode::CREATED, Json(user)))
    }

    #[route(DELETE, "/:id", guard = "auth_guard")]
    async fn delete_user(
        State(state): State<Arc<Self>>,
        Path(id): Path<u64>,
    ) -> Result<StatusCode, AppError> {
        state.user_service.delete(id)?;
        Ok(StatusCode::NO_CONTENT)
    }
}
```

## Route Parameters

The `#[route]` macro accepts:

| Parameter | Required | Description |
|-----------|----------|-------------|
| Method | Yes | HTTP method: `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, `OPTIONS` |
| Path | Yes | Route path, supports parameters with `/:param` |
| `guard` | No | Name of a guard function to protect the route |

## Path Parameters

Use Axum's `Path` extractor:

```rust
#[route(GET, "/:id")]
async fn get_user(Path(id): Path<u64>) -> String {
    format!("User {}", id)
}

// Multiple parameters
#[route(GET, "/:org_id/members/:user_id")]
async fn get_member(Path((org_id, user_id)): Path<(u64, u64)>) -> String {
    format!("Org {} Member {}", org_id, user_id)
}
```

## Query Parameters

Use Axum's `Query` extractor:

```rust
#[model]
pub struct Pagination {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[route(GET, "/")]
async fn list_users(Query(pagination): Query<Pagination>) -> Json<Vec<User>> {
    let page = pagination.page.unwrap_or(1);
    let limit = pagination.limit.unwrap_or(20);
    // ...
}
```

## JSON Request Bodies

Use `Json<T>` or `ValidatedJson<T>`:

```rust
// Standard — returns Axum's default error on parse failure
#[route(POST, "/")]
async fn create(Json(payload): Json<CreateUser>) -> Json<User> { ... }

// Validated — returns a clean AppError::unprocessable on failure
#[route(POST, "/")]
async fn create(ValidatedJson(payload): ValidatedJson<CreateUser>) -> Result<Json<User>, AppError> { ... }
```

## Response Types

Handlers can return anything that implements `IntoResponse`:

```rust
// Simple string
async fn handler() -> &'static str { "Hello" }

// JSON
async fn handler() -> Json<User> { Json(user) }

// Status code + body
async fn handler() -> (StatusCode, Json<User>) { (StatusCode::CREATED, Json(user)) }

// Result with AppError
async fn handler() -> Result<Json<User>, AppError> { Ok(Json(user)) }
```
