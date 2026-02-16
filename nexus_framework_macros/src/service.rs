//! # Service Macros
//!
//! Provides the `#[service]` and `#[service_impl]` procedural macros for defining
//! and instrumenting services in the Nexus Framework.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ItemImpl, ItemStruct};

/// Marks a struct as a service that can be auto-discovered and injected.
///
/// This macro:
/// 1. Adds `#[derive(Clone)]` to the struct
/// 2. Registers the service with the framework's inventory system
///
/// The service's `new()` method can optionally accept a `&DependencyContainer` parameter
/// to resolve dependencies on other services.
///
/// # Example
///
/// ```
/// // Simple service without dependencies
/// #[service]
/// pub struct UserService;
///
/// impl UserService {
///     pub fn new() -> Self { Self }
/// }
///
/// // Service with dependencies
/// #[service]
/// pub struct OrderService;
///
/// impl OrderService {
///     pub fn new(container: &DependencyContainer) -> Self {
///         let user_service: Arc<UserService> = container.get();
///         Self { user_service }
///     }
/// }
/// ```
pub fn service_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    let struct_name = &item_struct.ident;

    let derive_clone = quote! { #[derive(Clone)] };

    let service_registration = quote! {
        inventory::submit! {
            nexus_framework::ServiceFactory {
                name: stringify!(#struct_name),
                factory: |container| std::sync::Arc::new(#struct_name::new(container)),
            }
        }
    };

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
pub fn service_impl_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_impl = parse_macro_input!(item as ItemImpl);

    for item in &mut item_impl.items {
        if let ImplItem::Fn(method) = item {
            method.vis = syn::parse_quote!(pub);

            let is_new_method = method.sig.ident == "new";

            if !is_new_method {
                let method_name = &method.sig.ident;
                let original_block = &method.block;
                let original_attrs = &method.attrs;
                let original_sig = &method.sig;

                let method_vis = &method.vis;
                let method_defaultness = &method.defaultness;
                let traced_method: ImplItem = syn::parse_quote! {
                    #(#original_attrs)*
                    #method_vis #method_defaultness #original_sig {
                        let _span = nexus_framework::prelude::tracing::info_span!(stringify!(#method_name)).entered();
                        #original_block
                    }
                };

                *item = traced_method;
            }
        }
    }

    TokenStream::from(quote! {
        #item_impl
    })
}
