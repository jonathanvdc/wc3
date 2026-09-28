//! Codecs for single-field property and entry wrappers.
use super::{attributes::Container, sink_name};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Error, Fields, Result};

pub(super) fn expand(input: DeriveInput, options: Container, reading: bool) -> Result<TokenStream> {
    if options.default {
        return Err(Error::new_spanned(
            &input.ident,
            "container default is only supported on blocks",
        ));
    }
    if options.write_order.is_some() {
        return Err(Error::new_spanned(
            &input.ident,
            "write_order is only supported on blocks",
        ));
    }
    let value = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => {
                return Err(Error::new_spanned(
                    &input.ident,
                    "MDL property and entry derives require a single-field tuple struct",
                ))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input.ident,
                "MDL derives support structs only",
            ))
        }
    };
    if let Some(attr) = value.attrs.iter().find(|attr| attr.path().is_ident("mdl")) {
        return Err(Error::new_spanned(
            attr,
            "value wrappers do not support MDL field attributes",
        ));
    }
    let ty = &value.ty;
    let name = &input.ident;
    let mut generics = input.generics.clone();
    if reading {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::wc3::model::mdl::Read));
    } else {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::wc3::model::mdl::Write));
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    if reading {
        let start = options.validate_read.as_ref().map(|_| {
            quote! {
                __wc3_mdl_parser.peek()?;
                let __wc3_mdl_start = __wc3_mdl_parser.position();
            }
        });
        let prefix = options
            .property
            .map(|property| quote!(__wc3_mdl_parser.expect_ident(#property)?;));
        let validate = options.validate_read.map(|function| quote!(#function(&__wc3_mdl_value, ::wc3::model::mdl::Span::new(__wc3_mdl_start, __wc3_mdl_parser.position()))?;));
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdl::Read for #name #ty_generics #where_clause {
                fn read_mdl(__wc3_mdl_parser: &mut ::wc3::model::mdl::Parser<'_>) -> ::core::result::Result<Self, ::wc3::model::mdl::ReadError> {
                    #start
                    #prefix
                    let __wc3_mdl_value = Self(__wc3_mdl_parser.read_property::<#ty>()?);
                    #validate
                    ::core::result::Result::Ok(__wc3_mdl_value)
                }
            }
        })
    } else {
        let sink = sink_name(&input);
        let validate = options
            .validate_write
            .map(|function| quote!(#function(self)?;));
        let write = match options.property {
            Some(property) => quote!(__wc3_mdl_writer.property(#property, &self.0)),
            None => quote!(__wc3_mdl_writer.entry(&self.0)),
        };
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdl::Write for #name #ty_generics #where_clause {
                fn write_mdl<#sink: ::std::io::Write>(&self, __wc3_mdl_writer: &mut ::wc3::model::mdl::MdlWriter<#sink>) -> ::core::result::Result<(), ::wc3::model::mdl::WriteError> {
                    #validate
                    #write
                }
            }
        })
    }
}
