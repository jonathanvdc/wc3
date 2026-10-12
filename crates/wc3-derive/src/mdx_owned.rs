//! MDX owned decoding, independent of ownership derivation.
use crate::ownership::view_type;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, DeriveInput, Type};

pub(crate) fn expand(input: &DeriveInput) -> TokenStream {
    if input.generics.lifetimes().next().is_none() {
        return expand_static_owned_read(input);
    }
    let name = &input.ident;
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let ty: Type = parse_quote!(#name #ty_generics);
    let static_ty = view_type(&ty, "'static");
    let mut reader_generics = input.generics.clone();
    reader_generics.params = reader_generics
        .params
        .into_iter()
        .filter(|p| !matches!(p,syn::GenericParam::Lifetime(l) if l.lifetime.ident=="data"))
        .collect();
    for p in input.generics.type_params() {
        let ident = &p.ident;
        reader_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ident:'static));
    }
    let read_ty = view_type(&ty, "'input");
    reader_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(for<'input> #read_ty: ::wc3::model::mdlx::mdx::Read<'input>));
    reader_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(
            for<'input> #read_ty: ::wc3::model::common::IntoOwned<Owned = #static_ty>
        ));
    let (reader_impl, _, reader_where) = reader_generics.split_for_impl();
    quote! {
            impl #reader_impl ::wc3::model::mdlx::mdx::ReadOwned for #static_ty #reader_where {
                fn decode_owned<'input>(bytes:&'input[u8])->Result<Self,::wc3::model::mdlx::mdx::ReadError> {
                    <#read_ty as ::wc3::model::mdlx::mdx::Read<'input>>::decode_mdx(bytes)
                        .map(::wc3::model::common::IntoOwned::into_owned)
                }
            }
    }
}
fn expand_static_owned_read(input: &DeriveInput) -> TokenStream {
    if input.generics.lifetimes().next().is_some() {
        return TokenStream::new();
    }
    let name = &input.ident;
    let (_, ty_generics, _) = input.generics.split_for_impl();
    let ty: Type = parse_quote!(#name #ty_generics);
    let mut generics = input.generics.clone();
    generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(#ty:'static+for<'input> ::wc3::model::mdlx::mdx::Read<'input>));
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics ::wc3::model::mdlx::mdx::ReadOwned for #ty #where_clause {
            fn decode_owned<'input>(bytes:&'input[u8])->Result<Self,::wc3::model::mdlx::mdx::ReadError> {
                <Self as ::wc3::model::mdlx::mdx::Read<'input>>::decode_mdx(bytes)
            }
        }
    }
}
