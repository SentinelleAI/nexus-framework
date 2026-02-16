# Getting Started with Nexus Framework

This guide walks you through creating your first web application with Nexus Framework.

## Prerequisites

- [Rust](https://rustup.rs/) 1.75 or later
- Cargo (included with Rust)

## Create a New Project

```bash
cargo new my-nexus-app
cd my-nexus-app
```

## Add the Dependency

In your `Cargo.toml`:

```toml
[dependencies]
nexus_framework = { git = "https://github.com/SentinelleAI/nexus-framework.git" }
```

## Write Your Application

Replace `src/main.rs` with:

```rust
use nexus_framework::prelude::*;

// Step 1: Define a data model
#[model]
pub struct Greeting {
    pub message: String,
}

// Step 2: Define a service
#[service]
pub struct GreetingService;

#[service_impl]
impl GreetingService {
    pub fn new(_container: &DependencyContainer) -> Self {
        Self
    }

    pub fn greet(&self, name: &str) -> Greeting {
        Greeting {
            message: format!("Hello, {}! Welcome to Nexus.", name),
        }
    }
}

// Step 3: Define a controller
pub struct GreetingController {
    greeting_service: Arc<GreetingService>,
}

#[controller(path = "/api")]
impl GreetingController {
    pub fn new(container: &DependencyContainer) -> Self {
        Self {
            greeting_service: container.get(),
        }
    }

    #[route(GET, "/greet/:name")]
    async fn greet(
        State(state): State<Arc<Self>>,
        Path(name): Path<String>,
    ) -> Json<Greeting> {
        Json(state.greeting_service.greet(&name))
    }
}

// Step 4: Start the application
#[nexus_app(port = 3000)]
async fn main() {}
```

## Run the Application

```bash
cargo run
```

## Test It

```bash
# Health check (built-in)
curl http://localhost:3000/health

# Ping (built-in)
curl http://localhost:3000/ping

# Your endpoint
curl http://localhost:3000/api/greet/World
```

Expected response:
```json
{"message": "Hello, World! Welcome to Nexus."}
```

## Add Configuration

Create `nexus.toml` in your project root:

```toml
[server]
host = "0.0.0.0"
port = 3000

[app]
name = "my-nexus-app"
env = "development"
```

## What's Next?

- [Dependency Injection](guides/dependency-injection.md) — Learn about the DI container
- [Controllers & Routes](guides/controllers-and-routes.md) — Build HTTP endpoints
- [Error Handling](guides/error-handling.md) — Return clean error responses
- [Configuration](guides/configuration.md) — Customize your app settings
- [Route Guards](guides/guards.md) — Protect your routes
- [Scheduled Jobs](guides/scheduled-jobs.md) — Run background tasks
- [Macro Reference](reference/macros.md) — Complete macro documentation
