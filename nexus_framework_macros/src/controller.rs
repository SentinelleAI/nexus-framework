//! # Controller, Route, and Scheduled Job Macros
//!
//! Provides the `#[controller]`, `#[route]`, and `#[scheduled]` procedural macros
//! for defining HTTP controllers and scheduled jobs in the Nexus Framework.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Ident, ImplItem, ItemFn, ItemImpl, LitStr};

use crate::parsers::{RouteArgs, ScheduledArgs};

/// Marks a function as a scheduled job.
///
/// # Example
///
/// ```
/// #[scheduled(cron = "0 0 * * * *")]
/// async fn hourly_job() {
///     // Implementation...
/// }
/// ```
pub fn scheduled_macro(args: TokenStream, item: TokenStream) -> TokenStream {
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

/// Marks a method as an HTTP route handler (marker attribute processed by `controller`).
pub fn route_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

/// Defines a controller with routes.
///
/// This macro:
/// 1. Processes methods marked with `#[route]`
/// 2. Generates an `into_router` method that creates an Axum router
/// 3. Registers the controller with the framework's inventory system
/// 4. Applies guard middleware to routes that specify a `guard` parameter
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
///
///     #[route(GET, "/admin", guard = "auth_guard")]
///     async fn admin_panel(
///         State(state): State<Arc<Self>>,
///     ) -> impl IntoResponse {
///         // Protected by auth_guard
///     }
/// }
/// ```
pub fn controller_macro(args: TokenStream, item: TokenStream) -> TokenStream {
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

    let controller_base_path = controller_path_str
        .expect("#[controller] attribute requires a `path` property, e.g., #[controller(path = \"/users\")]");

    let mut item_impl = parse_macro_input!(item as ItemImpl);
    let controller_type = &item_impl.self_ty;

    let mut route_fragments = Vec::new();
    let mut route_info_strs = Vec::new();

    for item in &item_impl.items {
        if let ImplItem::Fn(method) = item {
            if let Some(route_attr) = method
                .attrs
                .iter()
                .find(|a| a.path().get_ident().map_or(false, |i| i == "route"))
            {
                let route_args: RouteArgs = match route_attr.parse_args() {
                    Ok(args) => args,
                    Err(e) => return e.to_compile_error().into(),
                };

                let http_method_str = route_args.method.to_string();
                let http_method_ident =
                    Ident::new(&http_method_str.to_lowercase(), route_args.method.span());
                let route_path = &route_args.path;
                let method_name = &method.sig.ident;

                let base_path_val = controller_base_path.value();
                let route_path_val = route_path.value();

                let full_path = if base_path_val.ends_with('/') {
                    format!("{}{}", base_path_val, &route_path_val[1..])
                } else {
                    format!("{}{}", base_path_val, route_path_val)
                };

                let guard_info = if route_args.guard.is_some() {
                    " 🛡️"
                } else {
                    ""
                };
                route_info_strs.push(format!("{} {}{}", http_method_str, full_path, guard_info));

                if let Some(guard_name) = &route_args.guard {
                    let guard_ident = Ident::new(&guard_name.value(), guard_name.span());
                    route_fragments.push(quote! {
                        .route(#route_path, axum::routing::#http_method_ident(#controller_type::#method_name)
                            .layer(axum::middleware::from_fn(|req: axum::http::Request<axum::body::Body>, next: axum::middleware::Next| async move {
                                nexus_framework::guard::guard_middleware(req, next, #guard_ident).await
                            }))
                        )
                    });
                } else {
                    route_fragments.push(quote! {
                        .route(#route_path, axum::routing::#http_method_ident(#controller_type::#method_name))
                    });
                }
            }
        }
    }

    let into_router_fn = quote! {
        pub fn into_router(self) -> axum::Router {
            use std::sync::Arc;
            axum::Router::new()
                #(#route_fragments)*
                .with_state(Arc::new(self))
        }
    };

    item_impl.items.push(syn::parse2(into_router_fn).unwrap());

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

    let expanded = quote! {
        #item_impl
        #controller_registration
    };

    TokenStream::from(expanded)
}
