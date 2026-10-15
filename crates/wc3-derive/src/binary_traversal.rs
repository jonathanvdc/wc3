//! Compositional field validation, animation-array traversal, and section references.
use crate::binary::{fresh_ident, Format};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Error, Fields, Index, Result};

pub(crate) enum Operation {
    Validate,
    VisitArrays,
    VisitReferences,
}
pub(crate) fn expand(input: DeriveInput, operation: Operation) -> TokenStream {
    expand_format(input, operation, Format::M2)
}
pub(crate) fn expand_format(
    input: DeriveInput,
    operation: Operation,
    format: Format,
) -> TokenStream {
    match expand_checked(input, operation, format) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error(),
    }
}
fn expand_checked(input: DeriveInput, operation: Operation, format: Format) -> Result<TokenStream> {
    let codec = match format {
        Format::M2 => quote!(::wc3::model::formats::m2),
        Format::Wmo => quote!(::wc3::model::formats::wmo),
        Format::M3 => quote!(::wc3::model::formats::m3),
    };
    let name = &input.ident;
    let (_, type_generics, _) = input.generics.split_for_impl();
    let mut generics = input.generics.clone();
    let lifetime = generics
        .lifetimes()
        .find(|parameter| parameter.lifetime.ident == "data")
        .or_else(|| generics.lifetimes().next())
        .map(|parameter| parameter.lifetime.clone())
        .unwrap_or_else(|| parse_quote!('data));
    if matches!(operation, Operation::VisitArrays) && generics.lifetimes().next().is_none() {
        generics.params.insert(0, parse_quote!(#lifetime));
    }
    let visitor = fresh_ident(&input, "__M2Visitor");
    let visitor_argument = fresh_ident(&input, "__m2_visitor");
    let mut field_calls = |fields: &Fields, enumeration: bool| {
        let mut calls = Vec::new();
        let mut bindings = Vec::new();
        let mut patterns = Vec::new();
        for (index, field) in fields.iter().enumerate() {
            let ty = &field.ty;
            let member = field
                .ident
                .as_ref()
                .map(|ident| quote!(#ident))
                .unwrap_or_else(|| {
                    let index = Index::from(index);
                    quote!(#index)
                });
            let binding = fresh_ident(&input, &format!("__m2_field_{index}"));
            let value = if enumeration {
                quote!(#binding)
            } else {
                match operation {
                    Operation::Validate | Operation::VisitReferences => quote!(&self.#member),
                    Operation::VisitArrays => quote!(&mut self.#member),
                }
            };
            let (bound, call) = match operation {
                Operation::Validate => (
                    parse_quote!(#ty: #codec::Validate),
                    quote!(#codec::Validate::validate(#value)?;),
                ),
                Operation::VisitReferences => (
                    parse_quote!(#ty:#codec::VisitReferences),
                    quote!(#codec::VisitReferences::visit_references(#value,#visitor_argument)?;),
                ),
                Operation::VisitArrays => (
                    parse_quote!(#ty: #codec::VisitArrays<#lifetime>),
                    quote!(#codec::VisitArrays::visit_arrays(#value, #visitor_argument)?;),
                ),
            };
            generics.make_where_clause().predicates.push(bound);
            calls.push(call);
            bindings.push(binding.clone());
            patterns.push(quote!(#member: #binding));
        }
        (calls, bindings, patterns)
    };
    let body = match &input.data {
        Data::Struct(data) => {
            let (calls, _, _) = field_calls(&data.fields, false);
            quote!(#(#calls)* ::core::result::Result::Ok(()))
        }
        Data::Enum(data) if data.variants.is_empty() => quote!(match *self {}),
        Data::Enum(data) => {
            let arms = data
                .variants
                .iter()
                .map(|variant| {
                    let variant_name = &variant.ident;
                    let (calls, bindings, patterns) = field_calls(&variant.fields, true);
                    let pattern = match &variant.fields {
                        Fields::Named(_) => quote!(Self::#variant_name { #(#patterns),* }),
                        Fields::Unnamed(_) => quote!(Self::#variant_name(#(#bindings),*)),
                        Fields::Unit => quote!(Self::#variant_name),
                    };
                    quote!(#pattern => { #(#calls)* ::core::result::Result::Ok(()) })
                })
                .collect::<Vec<_>>();
            quote!(match self { #(#arms),* })
        }
        Data::Union(_) => {
            return Err(Error::new_spanned(
                name,
                "M2 traversal derives do not support unions",
            ));
        }
    };
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    Ok(match operation {
        Operation::Validate => quote! {
            impl #impl_generics #codec::Validate for #name #type_generics #where_clause {
                fn validate(&self) -> ::core::result::Result<(), #codec::ReadError> { #body }
            }
        },
        Operation::VisitReferences => quote! {
            impl #impl_generics #codec::VisitReferences for #name #type_generics #where_clause{
                fn visit_references<#visitor:#codec::ReferenceVisitor>(&self,#visitor_argument:&mut #visitor)->::core::result::Result<(),#visitor::Error>{#body}
            }
        },
        Operation::VisitArrays => quote! {
            impl #impl_generics #codec::VisitArrays<#lifetime> for #name #type_generics #where_clause {
                fn visit_arrays<#visitor: #codec::ArrayVisitor<#lifetime>>(&mut self, #visitor_argument: &mut #visitor) -> ::core::result::Result<(), #visitor::Error> { #body }
            }
        },
    })
}
