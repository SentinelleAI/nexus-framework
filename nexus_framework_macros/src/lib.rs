//! # Nexus Framework Macros
//!
//! This crate provides procedural macros for the Nexus Framework, a web application framework
//! built on top of Axum. These macros simplify the creation of web applications by providing
//! a declarative API for defining services, controllers, routes, and models.
//!
//! ## Available Macros
//!
//! - [`service`](#service): Marks a struct as a service that can be auto-discovered and injected
//! - [`service_impl`](#service_impl): Adds tracing to service methods
//! - [`controller`](#controller): Defines a controller with routes
//! - [`route`](#route): Defines an HTTP endpoint within a controller
//! - [`model`](#model): Marks a struct as a data model with serialization support
//! - [`nexus_app`](#nexus_app): Sets up the main application with common boilerplate

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Ident, ImplItem, ItemFn, ItemImpl, ItemStruct, LitInt, LitStr, Result, Token};

//------------------------------------------------------------------------------
// Parser Structs
//------------------------------------------------------------------------------

/// Arguments for the `#[scheduled]` macro.
///
/// # Fields
/// - `cron`: The cron expression for scheduling
struct ScheduledArgs {
    cron: LitStr,
}

impl Parse for ScheduledArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut cron = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(Ident) {
                let ident: Ident = input.parse()?;
                if ident == "cron" {
                    input.parse::<Token![=]>()?;
                    cron = Some(input.parse()?);
                } else {
                    return Err(syn::Error::new(ident.span(), "unsupported property; only `cron` is supported"));
                }
            } else {
                return Err(lookahead.error());
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ScheduledArgs {
            cron: cron.expect(r#"#[scheduled] attribute requires a `cron` property, e.g., #[scheduled(cron = "0 0 * * * *")]"#),
        })
    }
}

/// Arguments for the `#[route]` macro.
///
/// # Fields
/// - `method`: The HTTP method (GET, POST, PUT, DELETE, etc.)
/// - `path`: The route path (e.g., "/:id", "/users")
struct RouteArgs {
    method: Ident,
    path: LitStr,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let method: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let path: LitStr = input.parse()?;
        Ok(RouteArgs { method, path })
    }
}

/// Arguments for the `#[nexus_app]` macro.
///
/// # Fields
/// - `name`: Optional application name (defaults to package name)
/// - `port`: Optional port number (defaults to 3000)
struct NexusAppArgs {
    name: Option<LitStr>,
    port: Option<LitInt>,
}

impl Parse for NexusAppArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name = None;
        let mut port = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(LitStr) {
                name = Some(input.parse()?);
            } else if lookahead.peek(Ident) {
                let ident: Ident = input.parse()?;
                if ident == "port" {
                    input.parse::<Token![=]>()?;
                    port = Some(input.parse()?);
                } else {
                    return Err(syn::Error::new(ident.span(), "unsupported property; only `port` is supported"));
                }
            } else {
                return Err(lookahead.error());
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(NexusAppArgs { name, port })
    }
}

//------------------------------------------------------------------------------
// Service Macros
//------------------------------------------------------------------------------

/// Marks a struct as a service that can be auto-discovered and injected.
///
/// This macro:
/// 1. Adds `#[derive(Clone)]` to the struct
/// 2. Registers the service with the framework's inventory system
///
/// # Example
///
/// ```
/// #[service]
/// pub struct UserService;
///
/// impl UserService {
///     pub fn new() -> Self { Self }
///     
///     pub fn find_user(&self, id: u64) -> User {
///         // Implementation...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn service(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as a struct
    let item_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = &item_struct.ident;

    // Add #[derive(Clone)] to the struct
    let derive_clone = quote! { #[derive(Clone)] };

    // Generate code to register the service with the inventory system
    let service_registration = quote! {
        inventory::submit! {
            nexus_framework::ServiceFactory {
                name: stringify!(#struct_name),
                factory: || std::sync::Arc::new(#struct_name::new()),
            }
        }
    };

    // Combine everything into the final output
    let expanded = quote! {
        #derive_clone
        #item_struct
        #service_registration
    };

    TokenStream::from(expanded)
}

/// Adds tracing to service methods.
///
/// This macro wraps each method (except `new`) with a tracing span
/// to provide better observability of service method calls.
///
/// # Example
///
/// ```
/// #[service_impl]
/// impl UserService {
///     pub fn new() -> Self { Self }
///     
///     // This method will be wrapped with a tracing span
///     pub fn find_user(&self, id: u64) -> User {
///         // Implementation...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn service_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as an impl block
    let mut item_impl = parse_macro_input!(item as ItemImpl);

    // Process each method in the impl block
    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            // Force all methods to be public so they can be used outside the module
            method.vis = syn::parse_quote!(pub);

            // Skip the `new` method - we don't want to add tracing to it
            let is_new_method = method.sig.ident == "new";

            if !is_new_method {
                let method_name = &method.sig.ident;
                let original_block = &method.block;
                let original_attrs = &method.attrs;
                let original_sig = &method.sig;

                // Create a new method with tracing added
                let method_vis = &method.vis;
                let method_defaultness = &method.defaultness;
                let traced_method: ImplItem = syn::parse_quote! {
                    #(#original_attrs)*
                    #method_vis #method_defaultness #original_sig {
                        let _span = nexus_framework::prelude::tracing::info_span!(stringify!(#method_name)).entered();
                        #original_block
                    }
                };

                // Replace the original method with the traced version
                *item = traced_method;
            }
        }
    }

    // Return the modified impl block
    TokenStream::from(quote! {
        #item_impl
    })
}

//------------------------------------------------------------------------------
// Controller Macros
//------------------------------------------------------------------------------

/// Marks a method as a scheduled job.
///
/// This macro is used to define a function that runs on a schedule.
///
/// # Example
///
/// ```
/// #[scheduled(cron = "0 0 * * * *")]
/// async fn hourly_job() {
///     // Implementation...
/// }
/// ```
#[proc_macro_attribute]
pub fn scheduled(args: TokenStream, item: TokenStream) -> TokenStream {
    // This macro is just a marker - the actual processing happens in the nexus_app macro
    let scheduled_args = parse_macro_input!(args as ScheduledArgs);
    let cron_str = scheduled_args.cron;

    let item_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &item_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let job_run_log = format!("{} has been run", fn_name_str);

    let expanded = quote! {
        inventory::submit! {
            nexus_framework::ScheduledJobFactory {
                name: stringify!(#fn_name),
                cron: #cron_str,
                job: || {
                    Box::pin(async move {
                        let span = nexus_framework::prelude::tracing::info_span!("nfw-cron");
                        let _enter = span.enter();
                        let span = nexus_framework::prelude::tracing::info_span!(#fn_name_str);
                        let _enter = span.enter();
                        #fn_name().await;
                        nexus_framework::prelude::tracing::info!(#job_run_log);
                    })
                },
            }
        }
        #item_fn
    };

    TokenStream::from(expanded)
}

/// Marks a method as an HTTP route handler.
///
/// This macro is used within a controller to define HTTP endpoints.
/// It's a marker attribute that is processed by the `controller` macro.
///
/// # Example
///
/// ```
/// #[controller(path = "/users")]
/// impl UserController {
///     #[route(GET, "/:id")]
///     async fn get_user(
///         State(state): State<Arc<Self>>,
///         Path(id): Path<u64>,
///     ) -> Json<User> {
///         // Implementation...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn route(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // This macro is just a marker - the actual processing happens in the controller macro
    item
}

/// Defines a controller with routes.
///
/// This macro:
/// 1. Processes methods marked with `#[route]`
/// 2. Generates an `into_router` method that creates an Axum router
/// 3. Registers the controller with the framework's inventory system
///
/// # Arguments
///
/// * `path` - The base path for all routes in this controller
///
/// # Example
///
/// ```
/// pub struct UserController {
///     user_service: Arc<UserService>,
/// }
///
/// #[controller(path = "/users")]
/// impl UserController {
///     pub fn new(container: &DependencyContainer) -> Self {
///         Self {
///             user_service: container.get(),
///         }
///     }
///
///     #[route(GET, "/:id")]
///     async fn get_user(
///         State(state): State<Arc<Self>>,
///         Path(id): Path<u64>,
///     ) -> Json<User> {
///         // Implementation...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn controller(args: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the path argument from the attribute
    let mut controller_path_str: Option<LitStr> = None;
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("path") {
            controller_path_str = Some(meta.value()?.parse()?);
            Ok(())
        } else {
            Err(meta.error("unsupported property; only `path` is supported"))
        }
    });
    parse_macro_input!(args with parser);

    // Ensure the path argument is provided
    let controller_base_path = controller_path_str
        .expect("#[controller] attribute requires a `path` property, e.g., #[controller(path = \"/users\")]");

    // Parse the impl block
    let mut item_impl = parse_macro_input!(item as ItemImpl);
    let controller_type = &item_impl.self_ty;

    // Collect route information
    let mut route_fragments = Vec::new();
    let mut route_info_strs = Vec::new();

    // Process each method in the impl block
    for item in &item_impl.items {
        if let ImplItem::Fn(method) = item {
            // Look for methods marked with #[route]
            if let Some(route_attr) = method.attrs.iter().find(|a| a.path().get_ident().map_or(false, |i| i == "route")) {
                // Parse the route arguments
                let route_args: RouteArgs = match route_attr.parse_args() {
                    Ok(args) => args,
                    Err(e) => return e.to_compile_error().into(),
                };

                // Extract route information
                let http_method_str = route_args.method.to_string();
                let http_method_ident = Ident::new(&http_method_str.to_lowercase(), route_args.method.span());
                let route_path = &route_args.path;
                let method_name = &method.sig.ident;

                // Construct the full path by combining the controller base path and the route path
                let base_path_val = controller_base_path.value();
                let route_path_val = route_path.value();

                let full_path = if base_path_val.ends_with('/') {
                    format!("{}{}", base_path_val, &route_path_val[1..])
                } else {
                    format!("{}{}", base_path_val, route_path_val)
                };

                // Store route information for logging and registration
                route_info_strs.push(format!("{} {}", http_method_str, full_path));

                // Generate the route fragment for the router
                route_fragments.push(quote! {
                    .route(#route_path, axum::routing::#http_method_ident(#controller_type::#method_name))
                });
            }
        }
    }

    // Generate the into_router method
    let into_router_fn = quote! {
        pub fn into_router(self) -> axum::Router {
            use std::sync::Arc;
            axum::Router::new()
                #(#route_fragments)*
                .with_state(Arc::new(self))
        }
    };

    // Add the into_router method to the impl block
    item_impl.items.push(syn::parse2(into_router_fn).unwrap());

    // Generate code to register the controller with the inventory system
    let controller_registration = quote! {
        inventory::submit! {
            nexus_framework::ControllerFactory {
                name: stringify!(#controller_type),
                routes: &[ #(#route_info_strs),* ],
                factory: |container| {
                    let controller = #controller_type::new(container);
                    let router = controller.into_router();
                    let tagged_router = router.layer(
                        nexus_framework::prelude::Extension(
                            nexus_framework::ControllerInfo { name: stringify!(#controller_type) }
                        )
                    );
                    Box::new(axum::Router::new().nest(#controller_base_path, tagged_router))
                }
            }
        }
    };

    // Combine everything into the final output
    let expanded = quote! {
        #item_impl
        #controller_registration
    };

    TokenStream::from(expanded)
}

//------------------------------------------------------------------------------
// Model Macros
//------------------------------------------------------------------------------

/// Marks a struct as a data model with serialization support.
///
/// This macro:
/// 1. Adds common derive macros (Serialize, Deserialize, Clone, Debug, PartialEq)
/// 2. Configures serde to use the framework's prelude
///
/// # Example
///
/// ```
/// #[model]
/// pub struct User {
///     id: u64,
///     username: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn model(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input as a struct
    let mut item_struct = parse_macro_input!(item as ItemStruct);

    // Configure serde to use the framework's prelude
    let serde_crate_path: syn::Attribute = syn::parse_quote! {
        #[serde(crate = "nexus_framework::prelude::serde")]
    };

    // Add common derive macros
    let derives: syn::Attribute = syn::parse_quote! {
        #[derive(
            nexus_framework::prelude::Serialize,
            nexus_framework::prelude::Deserialize,
            Clone,
            Debug,
            PartialEq
        )]
    };

    // Add the attributes to the struct
    item_struct.attrs.push(derives);
    item_struct.attrs.push(serde_crate_path);

    // Return the modified struct
    TokenStream::from(quote! { #item_struct })
}

//------------------------------------------------------------------------------
// Application Macros
//------------------------------------------------------------------------------

/// Sets up the main application with common boilerplate.
///
/// This macro:
/// 1. Configures tracing
/// 2. Sets up the dependency container
/// 3. Discovers and registers controllers
/// 4. Adds default routes (/ping, /health)
/// 5. Starts the HTTP server
///
/// # Arguments
///
/// * `name` - Optional application name (defaults to package name)
/// * `port` - Optional port number (defaults to 3000)
///
/// # Example
///
/// ```
/// #[nexus_app(port = 8080)]
/// async fn main() {
///     // Custom initialization code here
/// }
/// ```
#[proc_macro_attribute]
pub fn nexus_app(args: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the arguments
    let app_args = parse_macro_input!(args as NexusAppArgs);

    // Parse the main function
    let main_fn = parse_macro_input!(item as ItemFn);
    let user_code = main_fn.block;
    let main_attrs = main_fn.attrs;

    // Get the application name (or use the package name if not provided)
    let app_name = app_args.name.map_or_else(
        || quote! { env!("CARGO_PKG_NAME") },
        |n| quote! { #n },
    );

    // Get the port (or use 3000 if not provided)
    let port = app_args.port.map_or(quote! { 3000 }, |p| quote! { #p });

    // Generate the application setup code
    let setup_logic = quote! {
        // Start timing the application startup
        let start_time = nexus_framework::prelude::tokio::time::Instant::now();
        let env = std::env::var("NFW_ENV").unwrap_or_else(|_| "development".to_string());
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

            // Log each route
            for route_str in factory.routes {
                if let Some((method, path)) = route_str.split_once(' ') {
                    let padded_method = format!("{:<7}", method);
                    let colored_method = match method {
                        "GET" => padded_method.green().bold(),
                        "POST" => padded_method.yellow().bold(),
                        "PUT" => padded_method.blue().bold(),
                        "PATCH" => padded_method.magenta().bold(),
                        "DELETE" => padded_method.red().bold(),
                        "HEAD" => padded_method.green().bold(),
                        "OPTIONS" => padded_method.bright_black().bold(),
                        _ => padded_method.normal(),
                    };
                    tracing::info!("{} -> {}", colored_method, path);
                } else {
                    tracing::info!("        -> {}", route_str);
                }
            }

            // Create and merge the controller router
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
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], #port));
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

    // Generate the final main function
    quote! {
        #(#main_attrs)*
        #[tokio::main]
        async fn main() {
            #setup_logic
        }
    }
    .into()
}
