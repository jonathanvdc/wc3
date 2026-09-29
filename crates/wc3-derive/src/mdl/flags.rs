//! Bitfield definitions expose their named bits as an ordinary flattened field group.
use super::attributes::{field, Kind};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, Data, DeriveInput, Error, Fields, Meta, Result, Token,
    Type,
};

fn is_u32(ty: &Type) -> bool {
    match ty {
        Type::Group(group) => is_u32(&group.elem),
        Type::Paren(paren) => is_u32(&paren.elem),
        Type::Path(path) => path.path.is_ident("u32"),
        _ => false,
    }
}

pub(super) fn expand(input: &DeriveInput, reading: bool) -> Result<TokenStream> {
    let Data::Struct(data) = &input.data else {
        unreachable!()
    };
    if !matches!(&data.fields, Fields::Unnamed(fields) if fields.unnamed.len() == 1 && is_u32(&fields.unnamed[0].ty))
    {
        return Err(Error::new_spanned(
            input,
            "MDL flags require a single u32 tuple field",
        ));
    }
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("mdl"))
    {
        let items = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in items {
            if !["flags", "hive_flags", "allow_bits", "hive_skip_bits"]
                .iter()
                .any(|name| meta.path().is_ident(name))
            {
                return Err(Error::new_spanned(
                    meta,
                    "bitfield codecs support flags, hive_flags, allow_bits, and hive_skip_bits",
                ));
            }
        }
    }
    let attrs = &input.attrs;
    let storage = parse_quote! { #(#attrs)* bits: u32 };
    let mapping = field(&storage, 0)?;
    let Kind::Flags(flags) = &mapping.kind else {
        return Err(Error::new_spanned(input, "expected flags mappings"));
    };
    let name = &input.ident;
    let (impl_generics, ty_generics, clause) = input.generics.split_for_impl();
    let aliases = &mapping.hive_flags;
    let names: Vec<_> = flags
        .iter()
        .map(|(name, _)| name)
        .chain(
            aliases
                .iter()
                .flatten()
                .filter(|(name, _)| {
                    !flags
                        .iter()
                        .any(|(original, _)| original.value() == name.value())
                })
                .map(|(name, _)| name),
        )
        .collect();
    let visit = quote! { fn visit_mdl_names(visitor: &mut dyn ::core::ops::FnMut(&'static str, bool)) { #(visitor(#names, false);)* } };
    if reading {
        let arms = flags.iter().map(|(name, mask)| {
            let alias = aliases
                .iter()
                .flatten()
                .filter(|(alias, bit)| bit == mask && alias.value() != name.value())
                .map(|(name, _)| name);
            quote! {
                #name #(| #alias)* => {
                    if state & #mask != 0 {
                        return Err(::wc3::model::mdl::ReadError::new(
                            field.span, ::wc3::model::mdl::ReadErrorKind::DuplicateField,
                        ));
                    }
                    parser.expect(::wc3::model::mdl::TokenKind::Comma)?;
                    state |= #mask;
                }
            }
        });
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdl::ReadFields for #name #ty_generics #clause {
                type State = u32;
                #visit
                fn accepts_mdl_field(name: &str, static_form: bool) -> bool { !static_form && matches!(name, #(#names)|*) }
                fn begin_mdl_fields(_: &mut ::wc3::model::mdl::Parser<'_>) -> ::core::result::Result<Self::State, ::wc3::model::mdl::ReadError> { Ok(0) }
                fn read_mdl_field<'a>(mut state: Self::State, parser: &mut ::wc3::model::mdl::Parser<'a>, field: ::wc3::model::mdl::Field<'a>, _: ::wc3::model::mdl::Parser<'a>) -> ::core::result::Result<Self::State, ::wc3::model::mdl::ReadError> {
                    match field.name { #(#arms)* _ => return Err(parser.error(::wc3::model::mdl::ReadErrorKind::UnknownField)) }
                    Ok(state)
                }
                fn finish_mdl_fields(state: Self::State, _: ::wc3::model::mdl::Span, _: ::wc3::model::mdl::Span) -> ::core::result::Result<Self, ::wc3::model::mdl::ReadError> { Ok(Self(state)) }
            }
        })
    } else {
        let known = flags
            .iter()
            .fold(mapping.allow_bits, |bits, (_, mask)| bits | mask);
        let writes = flags.iter().map(|(name, mask)| {
            let hive = aliases
                .iter()
                .flatten()
                .find(|(_, bit)| bit == mask)
                .map(|(name, _)| name)
                .unwrap_or(name);
            let skip = mapping.hive_skip_bits & mask != 0;
            quote! {
                if self.0 & #mask != 0 {
                    if writer.dialect() == ::wc3::model::mdl::Dialect::HiveWorkshop {
                        if !#skip { writer.flag(#hive)?; }
                    } else {
                        writer.flag(#name)?;
                    }
                }
            }
        });
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdl::WriteFields for #name #ty_generics #clause {
                type State = ();
                #visit
                fn prepare_mdl_fields(&self, _: ::wc3::model::mdl::Dialect) -> ::core::result::Result<(), ::wc3::model::mdl::WriteError> {
                    if self.0 & !#known != 0 { return Err(::wc3::model::mdl::WriteError::Unrepresentable { field: concat!("unknown flag bits in ", stringify!(#name)) }.into()); } Ok(())
                }
                fn write_mdl_headers<W: ::std::io::Write>(&self, _: &mut ::wc3::model::mdl::Writer<W>) -> ::core::result::Result<(), ::wc3::model::IoError<::wc3::model::mdl::WriteError>> { Ok(()) }
                fn write_mdl_fields<W: ::std::io::Write>(&self, _: (), writer: &mut ::wc3::model::mdl::Writer<W>) -> ::core::result::Result<(), ::wc3::model::IoError<::wc3::model::mdl::WriteError>> { #(#writes)* Ok(()) }
            }
        })
    }
}
