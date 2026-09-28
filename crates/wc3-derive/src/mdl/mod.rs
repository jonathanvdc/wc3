//! MDL derive entry point. Parsing, validation and codec generation are separate.
mod attributes;
mod bounds;
mod omission;
mod read;
mod schema;
#[cfg(test)]
mod tests;
mod value;
mod write;

use attributes::container;
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::{DeriveInput, GenericParam, Ident, Result};

pub(crate) fn expand(input: DeriveInput, reading: bool) -> TokenStream {
    match expand_checked(input, reading) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error(),
    }
}

fn expand_checked(input: DeriveInput, reading: bool) -> Result<TokenStream> {
    let options = container(&input)?;
    if options.block.is_none() {
        return value::expand(input, options, reading);
    }
    let schema = schema::parse(&input, &options)?;
    if reading {
        read::expand(&input, &options, &schema)
    } else {
        write::expand(&input, &options, &schema)
    }
}

fn sink_name(input: &DeriveInput) -> Ident {
    let mut sink = format_ident!("__Wc3MdlSink");
    let mut suffix = 0;
    while input.generics.params.iter().any(|param| match param {
        GenericParam::Type(param) => param.ident == sink,
        GenericParam::Const(param) => param.ident == sink,
        _ => false,
    }) {
        suffix += 1;
        sink = format_ident!("__Wc3MdlSink{suffix}");
    }
    sink
}
