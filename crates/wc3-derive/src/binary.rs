//! Field-order binary record codecs with format-specific framing and envelopes.
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
            GenericParam::Lifetime(parameter) => parameter.lifetime.ident == name,
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
    M3,
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
    if matches!(input.data, Data::Enum(_)) {
        return crate::binary_value::binary(&input, operation, format);
    }
    let m2 = matches!(format, Format::M2);
    let m3 = matches!(format, Format::M3);
    let codec = match format {
        Format::M2 => quote!(::wc3::model::formats::m2),
        Format::Wmo => quote!(::wc3::model::formats::wmo),
        Format::M3 => quote!(::wc3::model::formats::m3),
    };
    let envelope = fresh_ident(&input, "__M3Envelope");
    let envelope_args = if m3 { quote!(<#envelope>) } else { quote!() };
    let name = &input.ident;
    let decoder = fresh_ident(&input, "__m2_decoder");
    let encoder = fresh_ident(&input, "__m2_encoder");
    let version = fresh_ident(&input, "__m2_version");
    let flags = fresh_ident(&input, "__m2_flags");
    let arguments = if m2 {
        quote!(#version,#flags)
    } else {
        quote!()
    };
    let parameters = if m2 {
        quote!(#version:u32,#flags:u32)
    } else {
        quote!()
    };
    let reserve = if m2 {
        quote!(#encoder.reserve_metadata(<Self as #codec::RecordLayout>::size(#encoder.version,#encoder.flags.bits()))?;)
    } else {
        quote!()
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
    if m3 {
        generics
            .params
            .push(parse_quote!(#envelope:#codec::Envelope));
    }
    let read_args = if m3 {
        quote!(#lifetime,#envelope)
    } else {
        quote!(#lifetime)
    };
    let decoder_type = if m3 {
        quote!(#codec::codec::Decoder<#lifetime,#envelope>)
    } else {
        quote!(#codec::codec::Decoder<#lifetime>)
    };
    let encoder_type = if m3 {
        quote!(#codec::codec::Encoder<#envelope>)
    } else {
        quote!(#codec::codec::Encoder)
    };
    let mut members = Vec::new();
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    let mut sizes = Vec::new();
    let size_method = quote!(size);
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
            Operation::Read => parse_quote!(#ty: #codec::Read<#read_args>),
            Operation::Write => parse_quote!(#ty: #codec::Write #envelope_args),
            Operation::Layout => parse_quote!(#ty: #codec::RecordLayout #envelope_args),
        };
        generics.make_where_clause().predicates.push(predicate);
        members.push(member.clone());
        reads.push(quote!(<#ty as #codec::Read<#read_args>>::read(#decoder)?));
        writes.push(quote!(<#ty as #codec::Write #envelope_args>::write(&self.#member,#encoder)?;));
        sizes.push(quote!(<#ty as #codec::RecordLayout #envelope_args>::#size_method(#arguments)));
    }
    if matches!(operation, Operation::Write) && m2 {
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
    let width = quote!(0 #(+ #sizes)*);
    Ok(match operation {
        Operation::Read => quote! {
            impl #impl_generics #codec::Read<#read_args> for #name #type_generics #where_clause {
                fn read(#decoder: &mut #decoder_type) -> ::core::result::Result<Self, #codec::ReadError> {
                    ::core::result::Result::Ok(#construct)
                }
            }
        },
        Operation::Write => quote! {
            impl #impl_generics #codec::Write #envelope_args for #name #type_generics #where_clause {
                fn write(&self, #encoder: &mut #encoder_type) -> ::core::result::Result<(), #codec::WriteError> {
                    #reserve
                    #(#writes)*
                    ::core::result::Result::Ok(())
                }
            }
        },
        Operation::Layout => quote! {
            impl #impl_generics #codec::RecordLayout #envelope_args for #name #type_generics #where_clause {
                fn #size_method(#parameters) -> usize { #width }
            }
        },
    })
}
