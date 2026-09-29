//! MDL derive entry point. Parsing, validation and codec generation are separate.
mod attributes;
mod bounds;
mod enumeration;
mod flags;
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
use syn::{
    punctuated::Punctuated, Data, DeriveInput, Fields, GenericParam, Generics, Ident, Meta, Result,
    Token,
};

pub(crate) fn expand(input: DeriveInput, reading: bool) -> TokenStream {
    match expand_checked(input, reading) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error(),
    }
}

fn expand_checked(input: DeriveInput, reading: bool) -> Result<TokenStream> {
    if matches!(input.data, Data::Enum(_)) {
        return enumeration::expand(&input, reading);
    }
    if matches!(&input.data, Data::Struct(data) if matches!(data.fields, Fields::Unnamed(_)))
        && input
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("mdl"))
            .any(|attr| {
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                    .is_ok_and(|items| items.iter().any(|meta| meta.path().is_ident("flags")))
            })
    {
        return flags::expand(&input, reading);
    }
    let options = container(&input)?;
    if options.block.is_none() && !options.fields {
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

// Keep implementation state opaque: a public record may contain private fields.
fn state_type(
    input: &DeriveInput,
    generics: &Generics,
    label: &str,
    names: &[Ident],
    types: &[TokenStream],
) -> (Ident, TokenStream) {
    use quote::quote;
    use syn::{parse_quote, Type, WherePredicate};
    let name = &input.ident;
    let state_name = format_ident!("__Wc3Mdl{}StateFor{}", label, name);
    let visibility = &input.vis;
    let (_, ty_generics, _) = generics.split_for_impl();
    let mut state_generics = generics.clone();
    if let Some(clause) = &mut state_generics.where_clause {
        for predicate in &mut clause.predicates {
            if let WherePredicate::Type(predicate) = predicate {
                if matches!(&predicate.bounded_ty, Type::Path(path) if path.path.is_ident("Self")) {
                    predicate.bounded_ty = parse_quote!(#name #ty_generics);
                }
            }
        }
    }
    let (params, _, clause) = state_generics.split_for_impl();
    let definition = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #visibility struct #state_name #params #clause {
            #(#names: #types,)*
            __wc3_mdl_marker: ::core::marker::PhantomData<fn() -> #name #ty_generics>,
        }
    };
    (state_name, definition)
}
