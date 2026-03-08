//! # Nexus Framework Macros
//!
//! This crate provides procedural macros for the Nexus Framework, a web application framework
//! built on top of Axum. These macros simplify the creation of web applications by providing
//! a declarative API for defining services, controllers, routes, and models.
//!
//! ## Available Macros
//!
//! - [`service`]: Marks a struct as a service that can be auto-discovered and injected
//! - [`service_impl`]: Adds tracing to service methods
//! - [`controller`]: Defines a controller with routes
//! - [`route`]: Defines an HTTP endpoint within a controller (with optional guard support)
//! - [`model`]: Marks a struct as a data model with serialization support
//! - [`nexus_app`]: Sets up the main application with common boilerplate
//! - [`scheduled`]: Defines a cron-based scheduled job

use proc_macro::TokenStream;

// ─── Internal Modules ────────────────────────────────────────────────────────

mod app;
mod controller;
mod model;
mod parsers;
mod service;

// ─── Proc Macro Exports ──────────────────────────────────────────────────────

/// Marks a struct as a service that can be auto-discovered and injected.
///
/// Use `#[service]` for services with no dependencies (calls `new()` with no arguments),
/// or `#[service(inject)]` to receive the `DependencyContainer` in `new()`.
#[proc_macro_attribute]
pub fn service(attr: TokenStream, item: TokenStream) -> TokenStream {
    service::service_macro(attr, item)
}

/// Adds tracing spans to service method implementations.
///
/// Wraps each method (except `new`) with a tracing span for observability.
#[proc_macro_attribute]
pub fn service_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    service::service_impl_macro(attr, item)
}

/// Defines a controller with HTTP routes.
///
/// Requires a `path` argument: `#[controller(path = "/users")]`.
#[proc_macro_attribute]
pub fn controller(args: TokenStream, item: TokenStream) -> TokenStream {
    controller::controller_macro(args, item)
}

/// Marks a method as an HTTP route handler within a controller.
///
/// Usage: `#[route(GET, "/path")]` or `#[route(POST, "/path", guard = "guard_fn")]`.
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    controller::route_macro(attr, item)
}

/// Marks a struct as a data model with serialization support.
///
/// Derives `Serialize`, `Deserialize`, `Debug`, and `Clone`.
#[proc_macro_attribute]
pub fn model(attr: TokenStream, item: TokenStream) -> TokenStream {
    model::model_macro(attr, item)
}

/// Defines a cron-based scheduled job.
///
/// Usage: `#[scheduled(cron = "0 */5 * * * *")]`.
#[proc_macro_attribute]
pub fn scheduled(args: TokenStream, item: TokenStream) -> TokenStream {
    controller::scheduled_macro(args, item)
}

/// Sets up the main Nexus application with all boilerplate.
///
/// Handles configuration loading, tracing setup, DI container, router
/// construction, and HTTP server startup with graceful shutdown.
#[proc_macro_attribute]
pub fn nexus_app(args: TokenStream, item: TokenStream) -> TokenStream {
    app::nexus_app_macro(args, item)
}
