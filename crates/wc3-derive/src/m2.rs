//! Field-order codecs for records in an offset address space.
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Error, Fields, GenericParam, Index, Result};

pub(crate) fn fresh_ident(input: &DeriveInput, base: &str) -> Ident {
    let mut name = base.to_owned();
    while input
        .generics
        .params
        .iter()
        .any(|parameter| match parameter {
            GenericParam::Type(parameter) => parameter.ident == name,
            GenericParam::Const(parameter) => parameter.ident == name,
            GenericParam::Lifetime(_) => false,
        })
    {
        name.push('_');
    }
    Ident::new(&name, input.ident.span())
}

#[derive(Clone, Copy)]
pub(crate) enum Format {
    M2,
    Wmo,
}

pub(crate) enum Operation {
    Read,
    Write,
    Layout,
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
    let wmo = matches!(format, Format::Wmo);
    let codec = if wmo {
        quote!(::wc3::model::wmo)
    } else {
        quote!(::wc3::model::m2)
    };
    let name = &input.ident;
    let decoder = fresh_ident(&input, "__m2_decoder");
    let encoder = fresh_ident(&input, "__m2_encoder");
    let version = fresh_ident(&input, "__m2_version");
    let flags = fresh_ident(&input, "__m2_flags");
    let arguments = if wmo {
        quote!()
    } else {
        quote!(#version,#flags)
    };
    let parameters = if wmo {
        quote!()
    } else {
        quote!(#version:u32,#flags:u32)
    };
    let reserve = if wmo {
        quote!()
    } else {
        quote!(#encoder.reserve_metadata(<Self as #codec::RecordLayout>::size(#encoder.version,#encoder.flags))?;)
    };
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            name,
            "Binary codec derives support structs; implement discriminated payload codecs explicitly",
        ));
    };
    let (_, type_generics, _) = input.generics.split_for_impl();
    let mut generics = input.generics.clone();
    let lifetime = generics
        .lifetimes()
        .next()
        .map(|p| p.lifetime.clone())
        .unwrap_or_else(|| parse_quote!('data));
    if matches!(operation, Operation::Read) && generics.lifetimes().next().is_none() {
        generics.params.insert(0, parse_quote!(#lifetime));
    }
    let mut members = Vec::new();
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    let mut sizes = Vec::new();
    for (index, field) in data.fields.iter().enumerate() {
        let ty = &field.ty;
        let member = field
            .ident
            .as_ref()
            .map(|ident| quote!(#ident))
            .unwrap_or_else(|| {
                let index = Index::from(index);
                quote!(#index)
            });
        let predicate = match operation {
            Operation::Read => parse_quote!(#ty: #codec::Read<#lifetime>),
            Operation::Write => parse_quote!(#ty: #codec::Write),
            Operation::Layout => parse_quote!(#ty: #codec::RecordLayout),
        };
        generics.make_where_clause().predicates.push(predicate);
        members.push(member.clone());
        reads.push(quote!(<#ty as #codec::Read<#lifetime>>::read(#decoder)?));
        writes.push(quote!(#codec::Write::write(&self.#member, #encoder)?;));
        sizes.push(quote!(<#ty as #codec::RecordLayout>::size(#arguments)));
    }
    if matches!(operation, Operation::Write) && !wmo {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(Self: #codec::RecordLayout));
    }
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let construct = match &data.fields {
        Fields::Named(_) => quote!(Self { #(#members: #reads),* }),
        Fields::Unnamed(_) => quote!(Self(#(#reads),*)),
        Fields::Unit => quote!(Self),
    };
    Ok(match operation {
        Operation::Read => quote! {
            impl #impl_generics #codec::Read<#lifetime> for #name #type_generics #where_clause {
                fn read(#decoder: &mut #codec::codec::Decoder<#lifetime>) -> ::core::result::Result<Self, #codec::ReadError> {
                    ::core::result::Result::Ok(#construct)
                }
            }
        },
        Operation::Write => quote! {
            impl #impl_generics #codec::Write for #name #type_generics #where_clause {
                fn write(&self, #encoder: &mut #codec::codec::Encoder) -> ::core::result::Result<(), #codec::WriteError> {
                    #reserve
                    #(#writes)*
                    ::core::result::Result::Ok(())
                }
            }
        },
        Operation::Layout => quote! {
            impl #impl_generics #codec::RecordLayout for #name #type_generics #where_clause {
                fn size(#parameters) -> usize { 0 #(+ #sizes)* }
            }
        },
    })
}
