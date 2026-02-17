# Configuration

Nexus Framework uses a layered configuration system that loads settings from multiple sources.

## Configuration File

Create a `nexus.toml` file at the root of your project:

```toml
[server]
host = "0.0.0.0"
port = 8080

[app]
name = "my-app"
env = "development"
```

## Configuration Layers

Settings are loaded in order (each overrides the previous):

1. **Default values** — built-in defaults
2. **`nexus.toml`** — base configuration file
3. **`nexus.{env}.toml`** — environment-specific overrides (e.g., `nexus.production.toml`)
4. **Environment variables** — prefixed with `NFW_`, using `__` for nested keys

### Environment Variables

```bash
NFW_SERVER__PORT=9090          # Sets server.port
NFW_SERVER__HOST=127.0.0.1     # Sets server.host
NFW_APP__NAME=my-app           # Sets app.name
NFW_ENV=production             # Sets the active environment
```

## Built-in Configuration Keys

### `[server]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `host` | String | `"0.0.0.0"` | Host address to bind to |
| `port` | u16 | `3000` | Port to listen on |

### `[app]`

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `name` | String | `"nexus-app"` | Application name |
| `env` | String | `"development"` | Environment (`development`, `staging`, `production`) |

## Custom Configuration Keys

Add any custom sections to `nexus.toml`:

```toml
[database]
url = "postgres://localhost/mydb"
pool_size = 10

[redis]
url = "redis://localhost:6379"
```

Access them with `NexusConfig::get()`:

```rust
let config = NexusConfig::load();
let db_url: Option<String> = config.get("database.url");
let pool_size: Option<i64> = config.get("database.pool_size");
```

## Environment-Specific Files

Create environment-specific override files:

```
nexus.toml              # Base config
nexus.development.toml  # Development overrides
nexus.staging.toml      # Staging overrides
nexus.production.toml   # Production overrides
```

The active environment is determined by the `NFW_ENV` environment variable (defaults to `"development"`).

## Trace Levels by Environment

The framework automatically sets the trace level based on the environment:

| Environment | Trace Level |
|-------------|-------------|
| `production` | INFO |
| `staging` | DEBUG |
| Others | TRACE |
