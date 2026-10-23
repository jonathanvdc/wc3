//! Detachment without imposing a borrowed-view API on semantic payloads.
use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_quote, Data, DeriveInput, Fields, Index, Lifetime, Type};

struct Rebind<'a> {
    parameters: &'a HashSet<String>,
    lifetime: Lifetime,
    found: bool,
}
impl VisitMut for Rebind<'_> {
    fn visit_lifetime_mut(&mut self, lifetime: &mut Lifetime) {
        if self.parameters.contains(&lifetime.ident.to_string()) {
            self.found = true;
            *lifetime = self.lifetime.clone();
        } else {
            visit_mut::visit_lifetime_mut(self, lifetime);
        }
    }
}
fn rebound(ty: &Type, parameters: &HashSet<String>, lifetime: &Lifetime) -> (Type, bool) {
    let mut ty = ty.clone();
    let mut visitor = Rebind {
        parameters,
        lifetime: lifetime.clone(),
        found: false,
    };
    visitor.visit_type_mut(&mut ty);
    (ty, visitor.found)
}

pub(crate) fn expand(input: DeriveInput) -> TokenStream {
    if matches!(input.data, Data::Union(_)) {
        return syn::Error::new_spanned(&input.ident, "detachment cannot be derived for a union")
            .to_compile_error();
    }
    let name = &input.ident;
    let parameters: HashSet<_> = input
        .generics
        .lifetimes()
        .map(|parameter| parameter.lifetime.ident.to_string())
        .collect();
    let mut lifetime_name = "__wc3_owned".to_owned();
    while parameters.contains(&lifetime_name) {
        lifetime_name.push('_');
    }
    let owned_lifetime = Lifetime::new(&format!("'{lifetime_name}"), name.span());
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let ty: Type = parse_quote!(#name #ty_generics);
    let owned_ty = rebound(&ty, &parameters, &owned_lifetime).0;
    let static_ty = rebound(&ty, &parameters, &parse_quote!('static)).0;
    let mut generics = input.generics.clone();
    generics.params.insert(0, parse_quote!(#owned_lifetime));
    for parameter in input.generics.type_params() {
        let ident = &parameter.ident;
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ident: 'static));
    }
    let convert = |field_ty: &Type, value: TokenStream| {
        let (_, changed) = rebound(field_ty, &parameters, &owned_lifetime);
        if changed {
            quote!(::wc3::model::common::Detached::into_detached(#value))
        } else {
            value
        }
    };
    let body = match &input.data {
        Data::Struct(data) => {
            let values: Vec<_> = data
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let member = field
                        .ident
                        .as_ref()
                        .map(|ident| quote!(#ident))
                        .unwrap_or_else(|| {
                            let index = Index::from(index);
                            quote!(#index)
                        });
                    let value = convert(&field.ty, quote!(self.#member));
                    (member, value)
                })
                .collect();
            match &data.fields {
                Fields::Named(_) => {
                    let fields = values.iter().map(|(member, value)| quote!(#member: #value));
                    quote!(#name { #(#fields),* })
                }
                Fields::Unnamed(_) => {
                    let values = values.iter().map(|(_, value)| value);
                    quote!(#name(#(#values),*))
                }
                Fields::Unit => quote!(#name),
            }
        }
        Data::Enum(data) => {
            let arms: Vec<_> = data
                .variants
                .iter()
                .map(|variant| {
                    let variant_name = &variant.ident;
                    let names: Vec<_> = variant
                        .fields
                        .iter()
                        .enumerate()
                        .map(|(index, field)| {
                            field.ident.clone().unwrap_or_else(|| {
                                syn::Ident::new(&format!("field_{index}"), name.span())
                            })
                        })
                        .collect();
                    let values: Vec<_> = variant
                        .fields
                        .iter()
                        .zip(&names)
                        .map(|(field, name)| convert(&field.ty, quote!(#name)))
                        .collect();
                    match &variant.fields {
                        Fields::Named(_) => quote!(
                            Self::#variant_name { #(#names),* } => #name::#variant_name {
                                #(#names: #values),*
                            }
                        ),
                        Fields::Unnamed(_) => quote!(
                            Self::#variant_name(#(#names),*) => #name::#variant_name(#(#values),*)
                        ),
                        Fields::Unit => quote!(Self::#variant_name => #name::#variant_name),
                    }
                })
                .collect();
            quote!(match self { #(#arms),* })
        }
        Data::Union(_) => unreachable!(),
    };
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let (method_generics, _, method_where) = input.generics.split_for_impl();
    quote! {
        impl #impl_generics ::wc3::model::common::Detached<#owned_lifetime> for #ty #where_clause {
            type Detached = #owned_ty;
            fn into_detached(self) -> Self::Detached { #body }
        }
        impl #method_generics #ty #method_where {
            /// Copies borrowed buffers and moves owned buffers into independent storage.
            pub fn into_owned(self) -> #static_ty
            where Self: ::wc3::model::common::Detached<'static, Detached = #static_ty> {
                ::wc3::model::common::Detached::into_detached(self)
            }
        }
    }
}
