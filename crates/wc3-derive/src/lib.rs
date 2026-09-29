//! Codec derives used to implement Warcraft III model records in `wc3`.
//!
//! These macros are for maintaining or extending the record implementations.
//! Applications using the built-in model types only need the codec traits in
//! `wc3::model::mdx` and `wc3::model::mdl`.
//!
//! Import macros through those format modules, for example
//! `#[derive(mdl::Read, mdl::Write)]`. The MDL attribute reference follows.
#![doc = include_str!("mdl.md")]
mod mdl;
mod mdx;

use mdl::expand as expand_mdl;
use mdx::{expand as expand_mdx, expand_value};
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Reads fields in declaration order. A sized record's final `Vec<T>` consumes its body.
///
/// Numeric enums use `#[mdx(value = u32)]` with explicit `#[mdx(value = N)]`
/// unit variants and an optional `#[mdx(unknown)]` single-u32 fallback variant.
#[proc_macro_derive(MdxRead, attributes(mdx))]
pub fn derive_mdx_read(input: TokenStream) -> TokenStream {
    expand_mdx(parse_macro_input!(input as DeriveInput), true).into()
}

/// Writes fields in declaration order. A sized record's final `Vec<T>` is written elementwise.
/// Numeric enums write their explicit wire value or the unknown payload verbatim.
#[proc_macro_derive(MdxWrite, attributes(mdx))]
pub fn derive_mdx_write(input: TokenStream) -> TokenStream {
    expand_mdx(parse_macro_input!(input as DeriveInput), false).into()
}

/// Reads an MDL block, single-value property, anonymous entry, or keyword enum.
/// Value and choice enums may mark one payload variant `#[mdl(unknown)]`; it has no text spelling.
#[proc_macro_derive(MdlRead, attributes(mdl))]
pub fn derive_mdl_read(input: TokenStream) -> TokenStream {
    expand_mdl(parse_macro_input!(input as DeriveInput), true).into()
}

/// Writes an MDL block, single-value property, anonymous entry, or keyword enum.
/// Writing a value or choice enum's `#[mdl(unknown)]` variant returns an unsupported-value error.
#[proc_macro_derive(MdlWrite, attributes(mdl))]
pub fn derive_mdl_write(input: TokenStream) -> TokenStream {
    expand_mdl(parse_macro_input!(input as DeriveInput), false).into()
}

/// Generates const `raw()` and `from_raw()` methods from numeric MDX mappings.
/// With an unknown variant, `from_raw()` returns `Self`; without it, `Option<Self>`.
#[proc_macro_derive(MdxValue, attributes(mdx))]
pub fn derive_mdx_value(input: TokenStream) -> TokenStream {
    expand_value(parse_macro_input!(input as DeriveInput)).into()
}
