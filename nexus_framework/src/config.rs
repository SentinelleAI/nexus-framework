//! # Application Configuration
//!
//! Provides a layered configuration system that loads settings from:
//! 1. A default `nexus.toml` file (if present)
//! 2. An environment-specific file like `nexus.production.toml` (if present)
//! 3. Environment variables prefixed with `NFW_`
//!
//! ## Example `nexus.toml`
//!
//! ```toml
//! [server]
//! port = 8080
//! host = "0.0.0.0"
//!
//! [app]
//! name = "my-app"
//! env = "development"
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use nexus_framework::config::NexusConfig;
//!
//! let config = NexusConfig::load();
//! println!("Port: {}", config.server.port);
//! ```

use serde::Deserialize;
use std::path::Path;

/// Top-level application configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct NexusConfig {
    /// Server-related settings (host, port).
    pub server: ServerConfig,
    /// Application metadata (name, environment).
    pub app: AppConfig,

    /// Raw config source for accessing custom user-defined keys.
    #[serde(skip)]
    raw: Option<config::Config>,
}

/// Server configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// The host address to bind to (default: `"0.0.0.0"`).
    pub host: String,
    /// The port to listen on (default: `3000`).
    pub port: u16,
}

/// Application metadata configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// The application name (default: `CARGO_PKG_NAME` or `"nexus-app"`).
    pub name: String,
    /// The environment (default: value of `NFW_ENV` or `"development"`).
    pub env: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "nexus-app".to_string(),
            env: std::env::var("NFW_ENV").unwrap_or_else(|_| "development".to_string()),
        }
    }
}

impl NexusConfig {
    /// Loads the configuration using a layered approach:
    ///
    /// 1. Start with default values
    /// 2. Merge `nexus.toml` if it exists
    /// 3. Merge `nexus.{env}.toml` if it exists (e.g., `nexus.production.toml`)
    /// 4. Override with environment variables prefixed with `NFW_`
    ///    (e.g., `NFW_SERVER__PORT=8080` sets `server.port`)
    pub fn load() -> Self {
        let env = std::env::var("NFW_ENV").unwrap_or_else(|_| "development".to_string());

        let builder = config::Config::builder()
            // Default values
            .set_default("server.host", "0.0.0.0")
            .unwrap()
            .set_default("server.port", 3000)
            .unwrap()
            .set_default("app.name", "nexus-app")
            .unwrap()
            .set_default("app.env", env.clone())
            .unwrap();

        // Merge nexus.toml if it exists
        let builder = if Path::new("nexus.toml").exists() {
            builder.add_source(config::File::with_name("nexus").format(config::FileFormat::Toml))
        } else {
            builder
        };

        // Merge environment-specific file if it exists
        let env_file = format!("nexus.{}", env);
        let builder = if Path::new(&format!("{}.toml", env_file)).exists() {
            builder.add_source(config::File::with_name(&env_file).format(config::FileFormat::Toml))
        } else {
            builder
        };

        // Override with environment variables (NFW_SERVER__PORT=8080 -> server.port)
        let builder = builder.add_source(config::Environment::with_prefix("NFW").separator("__"));

        match builder.build() {
            Ok(raw_config) => {
                let mut nexus_config: NexusConfig =
                    raw_config.clone().try_deserialize().unwrap_or_else(|e| {
                        tracing::warn!("⚠️ Failed to deserialize config: {}, using defaults", e);
                        NexusConfig::default()
                    });
                nexus_config.raw = Some(raw_config);
                nexus_config
            }
            Err(e) => {
                tracing::warn!("⚠️ Failed to load config: {}, using defaults", e);
                NexusConfig::default()
            }
        }
    }

    /// Gets a custom configuration value by key.
    ///
    /// This allows access to user-defined configuration keys beyond the built-in
    /// `server` and `app` sections.
    ///
    /// # Example
    ///
    /// Given a `nexus.toml`:
    /// ```toml
    /// [database]
    /// url = "postgres://localhost/mydb"
    /// pool_size = 10
    /// ```
    ///
    /// Access values with:
    /// ```rust
    /// let db_url: Option<String> = config.get("database.url");
    /// let pool_size: Option<i64> = config.get("database.pool_size");
    /// ```
    pub fn get<'de, T: Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.raw.as_ref().and_then(|c| c.get::<T>(key).ok())
    }
}
