# Nexus Framework

Nexus is a lightweight and modular web framework for Rust, designed to build high-performance and scalable web applications and APIs. It is built on top of the popular Axum library and provides a set of macros and conventions to streamline the development process.

## Getting Started

To start using the Nexus Framework, you need to have Rust and Cargo installed on your system. You can then add the following dependencies to your `Cargo.toml` file:

```toml
[dependencies]
nexus_framework = { path = "ssh://git@github.com/SentinelleAI/nexus-framework.git" }
```

## Core Concepts

The Nexus Framework is built around three core concepts:

*   **Models:** Models are simple structs that represent the data in your application. They are defined using the `#[model]` attribute.
*   **Services:** Services are responsible for handling the business logic of your application. They are defined using the `#[service]` and `#[service_impl]` attributes.
*   **Controllers:** Controllers are responsible for handling incoming HTTP requests and returning responses. They are defined using the `#[controller]` and `#[route]` attributes.

## Usage Examples

Here is an example of how to create a simple web application using the Nexus Framework:

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
    pub fn new() -> Self { Self }
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
        Self {
            user_service: container.get(),
        }
    }

    #[route(GET, "/:id")]
    async fn get_user(
        State(state): State<Arc<Self>>,
        Path(id): Path<u64>,
    ) -> Json<User> {
        Json(state.user_service.find_user(id))
    }
}

// Create the main application
#[nexus_app()]
async fn main() {
}
```

## Contributing

Contributions to the Nexus Framework are welcome! If you find a bug or have a feature request, please open an issue on the GitHub repository.

## License

The Nexus Framework is licensed under the MIT License.
