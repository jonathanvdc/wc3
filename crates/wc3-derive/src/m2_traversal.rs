//! Field-wise validation and animation-leaf traversal.
use crate::m2::fresh_ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Error, Fields, Index, Result};

pub(crate) enum Operation {
    Validate,
    VisitArrays,
}
pub(crate) fn expand(input: DeriveInput, operation: Operation) -> TokenStream {
    match expand_checked(input, operation) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error(),
    }
}
fn expand_checked(input: DeriveInput, operation: Operation) -> Result<TokenStream> {
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
                    Operation::Validate => quote!(&self.#member),
                    Operation::VisitArrays => quote!(&mut self.#member),
                }
            };
            let (bound, call) = match operation {
                Operation::Validate => (
                    parse_quote!(#ty: ::wc3::model::m2::Validate),
                    quote!(::wc3::model::m2::Validate::validate(#value)?;),
                ),
                Operation::VisitArrays => (
                    parse_quote!(#ty: ::wc3::model::m2::VisitArrays<#lifetime>),
                    quote!(::wc3::model::m2::VisitArrays::visit_arrays(#value, #visitor_argument)?;),
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
            ))
        }
    };
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    Ok(match operation {
        Operation::Validate => quote! {
            impl #impl_generics ::wc3::model::m2::Validate for #name #type_generics #where_clause {
                fn validate(&self) -> ::core::result::Result<(), ::wc3::model::m2::ReadError> { #body }
            }
        },
        Operation::VisitArrays => quote! {
            impl #impl_generics ::wc3::model::m2::VisitArrays<#lifetime> for #name #type_generics #where_clause {
                fn visit_arrays<#visitor: ::wc3::model::m2::ArrayVisitor<#lifetime>>(&mut self, #visitor_argument: &mut #visitor) -> ::core::result::Result<(), #visitor::Error> { #body }
            }
        },
    })
}
