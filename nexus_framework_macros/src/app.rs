//! # Application Bootstrap Macro
//!
//! Provides the `#[nexus_app]` procedural macro that sets up the main application
//! with all necessary boilerplate.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

use crate::parsers::NexusAppArgs;

/// Sets up the main application with common boilerplate.
///
/// This macro:
/// 1. Loads configuration from `nexus.toml` and environment variables
/// 2. Configures tracing
/// 3. Sets up the dependency container
/// 4. Discovers and registers controllers
/// 5. Adds default routes (/ping, /health)
/// 6. Starts the HTTP server
///
/// # Arguments
///
/// * `name` - Optional application name (defaults to package name)
/// * `port` - Optional port number (overrides config, defaults to 3000)
///
/// # Example
///
/// ```
/// #[nexus_app(port = 8080)]
/// async fn main() {
///     // Custom initialization code here
/// }
/// ```
pub fn nexus_app_macro(args: TokenStream, item: TokenStream) -> TokenStream {
    let app_args = parse_macro_input!(args as NexusAppArgs);

    let main_fn = parse_macro_input!(item as ItemFn);
    let user_code = main_fn.block;
    let main_attrs = main_fn.attrs;

    let app_name = app_args
        .name
        .map_or_else(|| quote! { env!("CARGO_PKG_NAME") }, |n| quote! { #n });

    let port_override = app_args
        .port
        .map(|p| quote! { Some(#p as u16) })
        .unwrap_or(quote! { None::<u16> });

    let setup_logic = quote! {
        // Load configuration
        let nexus_config = nexus_framework::config::NexusConfig::load();

        // Determine the port: macro attribute > config > default (3000)
        let port: u16 = {
            let macro_port: Option<u16> = #port_override;
            macro_port.unwrap_or(nexus_config.server.port)
        };

        // Start timing the application startup
        let start_time = nexus_framework::prelude::tokio::time::Instant::now();
        let env = nexus_config.app.env.clone();
        let trace_level = match env.as_str() {
            "production" => nexus_framework::prelude::tracing::Level::INFO,
            "staging" => nexus_framework::prelude::tracing::Level::DEBUG,
            _ => nexus_framework::prelude::tracing::Level::TRACE,
        };

        // Configure tracing
        nexus_framework::prelude::tracing_subscriber::fmt()
            .event_format(nexus_framework::CustomFormatter)
            .with_max_level(trace_level)
            .init();
        let span = nexus_framework::prelude::tracing::info_span!("nfw-init");
        let _enter = span.enter();
        tracing::info!("🔧 Configuring tracing system...");

        // Create a tracing span for the application
        let _span = nexus_framework::prelude::tracing::info_span!(#app_name).entered();

        // Log environment information
        tracing::info!("✅ Tracing system initialized");
        tracing::info!("🚀 Starting {} application...", #app_name);
        tracing::info!("🌐 Environment: {}", env);
        tracing::info!("📦 Version: {}", env!("CARGO_PKG_VERSION"));
        tracing::info!("⚙️ Configuration loaded (server.port={}, server.host={})", nexus_config.server.port, nexus_config.server.host);

        // Scan and log system resources
        tracing::info!("🔍 Scanning system resources...");
        nexus_framework::log_system_resources();

        // Build the dependency container
        tracing::debug!("🧰 Building dependency container...");
        let container = nexus_framework::DependencyContainer::build();
        tracing::info!("✅ {} services discovered and instantiated",
            inventory::iter::<nexus_framework::ServiceFactory>.into_iter().count());

        // Build the application router
        tracing::info!("🔄 Building application router...");
        let mut app = axum::Router::new();

        // Discover and register controllers
        for factory in inventory::iter::<nexus_framework::ControllerFactory> {
            tracing::info!(controller = factory.name, "🎮 Discovered routes:");

            for route_str in factory.routes {
                if let Some((method, path)) = route_str.split_once(' ') {
                    let padded_method = format!("{:<7}", method);
                    use nexus_framework::prelude::Colorize;
                    let colored_method = match method.trim_end() {
                        "GET" => padded_method.green().bold().to_string(),
                        "POST" => padded_method.yellow().bold().to_string(),
                        "PUT" => padded_method.blue().bold().to_string(),
                        "PATCH" => padded_method.magenta().bold().to_string(),
                        "DELETE" => padded_method.red().bold().to_string(),
                        "HEAD" => padded_method.green().bold().to_string(),
                        "OPTIONS" => padded_method.bright_black().bold().to_string(),
                        _ => padded_method,
                    };
                    tracing::info!("{} -> {}", colored_method, path);
                } else {
                    tracing::info!("        -> {}", route_str);
                }
            }

            let controller_router = (factory.factory)(&container);
            app = app.merge(*controller_router);
        }

        // Add default routes
        tracing::info!("🏓 Adding default /ping and /health routes...");
        app = app
            .route(
                "/ping",
                nexus_framework::prelude::axum::routing::get(nexus_framework::ping_handler)
            )
            .route(
                "/health",
                nexus_framework::prelude::axum::routing::get(nexus_framework::health_handler)
            );

        // Add request logging middleware
        app = app.layer(axum::middleware::from_fn(nexus_framework::middleware::request_logging));
        tracing::info!("✅ Router built and HTTP tracing layer enabled.");

        // Start the HTTP server
        tracing::info!("🔌 Preparing HTTP server...");
        let host = nexus_config.server.host.clone();
        let addr: std::net::SocketAddr = format!("{}:{}", host, port).parse().unwrap_or_else(|_| {
            tracing::warn!("⚠️ Invalid address {}:{}, falling back to 0.0.0.0:{}", host, port, port);
            std::net::SocketAddr::from(([0, 0, 0, 0], port))
        });
        tracing::info!("🔒 Binding to address: {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        tracing::info!(listen_addr = %listener.local_addr().unwrap(), "📡 Server listening");

        // Discover and start scheduled jobs
        let scheduler = nexus_framework::prelude::tokio_cron_scheduler::JobScheduler::new().await.unwrap();
        for factory in inventory::iter::<nexus_framework::ScheduledJobFactory> {
            let job = nexus_framework::prelude::tokio_cron_scheduler::Job::new_async(factory.cron, move |_, _| {
                (factory.job)()
            }).unwrap();
            scheduler.add(job).await.unwrap();
            tracing::info!(job = factory.name, cron = factory.cron, "📅 Scheduled job added");
        }
        scheduler.start().await.unwrap();
        tracing::info!("✅ Scheduler started");

        tracing::info!("✨ Application initialization complete");
        tracing::info!("⚡ Application startup took {} ms", start_time.elapsed().as_millis());

        // Run user code
        #user_code

        // Start the server with graceful shutdown
        tracing::info!("🚀 Starting HTTP server, ready to handle requests...");
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let ctrl_c = async {
                    tokio::signal::ctrl_c()
                        .await
                        .expect("Failed to install Ctrl+C handler");
                };

                #[cfg(unix)]
                let terminate = async {
                    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                        .expect("Failed to install SIGTERM handler")
                        .recv()
                        .await;
                };

                #[cfg(not(unix))]
                let terminate = std::future::pending::<()>(); // Placeholder for non-Unix systems

                tokio::select! {
                    _ = ctrl_c => {},
                    _ = terminate => {},
                }
                let shutdown_span = nexus_framework::prelude::tracing::info_span!("nfw-exit").entered();
                tracing::info!("🛑 Shutdown signal received, cleaning up resources...");
                tracing::info!("👋 Goodbye from {}!", #app_name);
            })
            .await
            .expect("Server failed to start or shut down gracefully");
    };

    quote! {
        #(#main_attrs)*
        #[tokio::main]
        async fn main() {
            #setup_logic
        }
    }
    .into()
}
