//! Field-order MDX codecs and streaming MDL codecs for structs.
mod mdl;
mod mdx;

use mdl::expand as expand_mdl;
use mdx::expand as expand_mdx;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Reads fields in declaration order. A sized record's final `Vec<T>` consumes its body.
#[proc_macro_derive(MdxRead, attributes(mdx))]
pub fn derive_mdx_read(input: TokenStream) -> TokenStream {
    expand_mdx(parse_macro_input!(input as DeriveInput), true).into()
}

/// Writes fields in declaration order. A sized record's final `Vec<T>` is written elementwise.
#[proc_macro_derive(MdxWrite, attributes(mdx))]
pub fn derive_mdx_write(input: TokenStream) -> TokenStream {
    expand_mdx(parse_macro_input!(input as DeriveInput), false).into()
}

/// Reads an MDL block, single-value property, or anonymous entry into a struct.
#[proc_macro_derive(MdlRead, attributes(mdl))]
pub fn derive_mdl_read(input: TokenStream) -> TokenStream {
    expand_mdl(parse_macro_input!(input as DeriveInput), true).into()
}

/// Writes an MDL block, single-value property, or anonymous entry to a sink.
#[proc_macro_derive(MdlWrite, attributes(mdl))]
pub fn derive_mdl_write(input: TokenStream) -> TokenStream {
    expand_mdl(parse_macro_input!(input as DeriveInput), false).into()
}
