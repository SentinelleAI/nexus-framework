//! # Macro Argument Parsers
//!
//! Contains the parsing logic for procedural macro attributes used throughout the framework.

use syn::{
    parse::{Parse, ParseStream},
    Ident, LitInt, LitStr, Result, Token,
};

/// Arguments for the `#[scheduled]` macro.
///
/// # Fields
/// - `cron`: The cron expression for scheduling
pub struct ScheduledArgs {
    pub cron: LitStr,
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
                    return Err(syn::Error::new(
                        ident.span(),
                        "unsupported property; only `cron` is supported",
                    ));
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
/// - `guard`: Optional guard function name (e.g., "auth_guard")
pub struct RouteArgs {
    pub method: Ident,
    pub path: LitStr,
    pub guard: Option<LitStr>,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let method: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let path: LitStr = input.parse()?;

        let mut guard = None;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.peek(Ident) {
                let ident: Ident = input.parse()?;
                if ident == "guard" {
                    input.parse::<Token![=]>()?;
                    guard = Some(input.parse::<LitStr>()?);
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        "unsupported property; only `guard` is supported",
                    ));
                }
            }
        }

        Ok(RouteArgs {
            method,
            path,
            guard,
        })
    }
}

/// Arguments for the `#[nexus_app]` macro.
///
/// # Fields
/// - `name`: Optional application name (defaults to package name)
/// - `port`: Optional port number (defaults to 3000)
pub struct NexusAppArgs {
    pub name: Option<LitStr>,
    pub port: Option<LitInt>,
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
                    return Err(syn::Error::new(
                        ident.span(),
                        "unsupported property; only `port` is supported",
                    ));
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
