//! # Model Macro
//!
//! Provides the `#[model]` procedural macro for defining data models
//! with automatic serialization support.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

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
pub fn model_macro(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(item as ItemStruct);

    let serde_crate_path: syn::Attribute = syn::parse_quote! {
        #[serde(crate = "nexus_framework::prelude::serde")]
    };

    let derives: syn::Attribute = syn::parse_quote! {
        #[derive(
            nexus_framework::prelude::Serialize,
            nexus_framework::prelude::Deserialize,
            Clone,
            Debug,
            PartialEq
        )]
    };

    item_struct.attrs.push(derives);
    item_struct.attrs.push(serde_crate_path);

    TokenStream::from(quote! { #item_struct })
}
