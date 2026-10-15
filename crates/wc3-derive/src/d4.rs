//! Sequential Diablo IV record codecs.
use crate::binary::fresh_ident;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, Attribute, Data, DeriveInput, Error, Field, Fields, GenericArgument, GenericParam,
    Lifetime, LitInt, PathArguments, Result, Type, WherePredicate,
};

#[derive(Clone, Copy)]
pub(crate) enum Operation {
    Read,
    Write,
    Layout,
}

#[derive(Clone, Copy, Default)]
enum Kind {
    #[default]
    Inline,
    Value,
    Array,
    InlineArray,
    Bytes,
    Polymorphic(u32),
}

struct Options {
    kind: Kind,
    external: bool,
}

fn check_record_attributes(attributes: &[Attribute]) -> Result<()> {
    if let Some(attribute) = attributes.iter().find(|a| a.path().is_ident("d4")) {
        return Err(Error::new_spanned(
            attribute,
            "record-level D4 attributes are not supported",
        ));
    }
    Ok(())
}

fn field_options(field: &Field) -> Result<Options> {
    let mut kind = None;
    let mut external = false;
    for attribute in field.attrs.iter().filter(|a| a.path().is_ident("d4")) {
        attribute.parse_nested_meta(|meta| {
            if meta.path.is_ident("external") {
                if external {
                    return Err(meta.error("duplicate external address space"));
                }
                external = true;
                return Ok(());
            }
            let selected = if meta.path.is_ident("value") {
                Kind::Value
            } else if meta.path.is_ident("array") {
                Kind::Array
            } else if meta.path.is_ident("inline_array") {
                Kind::InlineArray
            } else if meta.path.is_ident("bytes") {
                Kind::Bytes
            } else if meta.path.is_ident("polymorphic") {
                Kind::Polymorphic(meta.value()?.parse::<LitInt>()?.base10_parse()?)
            } else {
                return Err(meta.error("unknown D4 field attribute"));
            };
            if kind.replace(selected).is_some() {
                return Err(meta.error("conflicting field encodings"));
            }
            Ok(())
        })?;
    }
    let kind = kind.unwrap_or_default();
    if external && !matches!(kind, Kind::Array | Kind::Polymorphic(_)) {
        return Err(Error::new_spanned(
            field,
            "external requires an array or polymorphic descriptor",
        ));
    }
    Ok(Options { kind, external })
}

fn array_element(ty: &Type) -> Result<(&Type, bool)> {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            if segment.ident == "Vec" || segment.ident == "ZeroVec" {
                if let PathArguments::AngleBracketed(arguments) = &segment.arguments {
                    if let Some(element) = arguments.args.iter().find_map(|argument| {
                        if let GenericArgument::Type(ty) = argument {
                            Some(ty)
                        } else {
                            None
                        }
                    }) {
                        return Ok((element, segment.ident == "ZeroVec"));
                    }
                }
            }
        }
    }
    Err(Error::new_spanned(
        ty,
        "array requires Vec<Record> or ZeroVec<'data, Value>",
    ))
}

fn needs_bound(ty: &Type, input: &DeriveInput) -> bool {
    if let Type::Path(path) = ty {
        if path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == input.ident || segment.ident == "Self")
        {
            return false;
        }
    }
    fn contains(tokens: TokenStream, names: &[&Ident]) -> bool {
        tokens.into_iter().any(|token| match token {
            TokenTree::Ident(ident) => names.iter().any(|name| **name == ident),
            TokenTree::Group(group) => contains(group.stream(), names),
            _ => false,
        })
    }
    let names = input
        .generics
        .type_params()
        .map(|p| &p.ident)
        .collect::<Vec<_>>();
    contains(ty.to_token_stream(), &names)
}

fn checked_layout(input: &DeriveInput, widths: &[TokenStream], actual: bool) -> TokenStream {
    let widths_name = fresh_ident(input, "__d4_widths");
    let total = fresh_ident(input, "__d4_total");
    let index = fresh_ident(input, "__d4_index");
    let actual_check = actual.then(|| {
        quote! {
            assert!(#total == <Self as ::wc3::model::formats::diablo::d4::RecordLayout>::SIZE,
                "D4 field widths differ from RecordLayout::SIZE");
        }
    });
    quote!({
        let #widths_name: &[usize] = &[#(#widths),*];
        let mut #total = 0usize;
        let mut #index = 0;
        while #index < #widths_name.len() {
            #total = #total.checked_add(#widths_name[#index]).expect("D4 record width overflow");
            #index += 1;
        }
        #actual_check
        #total
    })
}

pub(crate) fn expand(input: DeriveInput, operation: Operation) -> TokenStream {
    match expand_checked(input, operation) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error(),
    }
}

fn expand_checked(input: DeriveInput, operation: Operation) -> Result<TokenStream> {
    check_record_attributes(&input.attrs)?;
    let Data::Struct(data) = &input.data else {
        return Err(Error::new_spanned(
            &input.ident,
            "D4 record derives require a struct",
        ));
    };
    let Fields::Named(fields) = &data.fields else {
        return Err(Error::new_spanned(
            &input.ident,
            "D4 records require named fields",
        ));
    };
    let codec = quote!(::wc3::model::formats::diablo::d4);
    let name = &input.ident;
    let (_, type_generics, _) = input.generics.split_for_impl();
    let mut generics = input.generics.clone();
    let lifetime = generics
        .lifetimes()
        .next()
        .map(|parameter| parameter.lifetime.clone())
        .unwrap_or_else(|| Lifetime::new("'__d4_data", input.ident.span()));
    if matches!(operation, Operation::Read) && generics.lifetimes().next().is_none() {
        generics.params.insert(0, parse_quote!(#lifetime));
    }
    let decoder = fresh_ident(&input, "__d4_decoder");
    let encoder = fresh_ident(&input, "__d4_encoder");
    let base = fresh_ident(&input, "__d4_offset");
    let space = fresh_ident(&input, "__d4_space");
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    let mut widths = Vec::new();
    let cursor = fresh_ident(&input, "__d4_cursor");
    let value = fresh_ident(&input, "__d4_value");
    let cursor_start = (fields.named.len() > 1).then(|| quote!(let mut #cursor = #base;));

    for (index, field) in fields.named.iter().enumerate() {
        let options = field_options(field)?;
        let member = field.ident.as_ref().expect("named field");
        let ty = &field.ty;
        let external = options.external;
        let at = if fields.named.len() > 1 {
            quote!(#cursor)
        } else {
            quote!(#base)
        };
        let (read, write, width, bound): (
            TokenStream,
            TokenStream,
            TokenStream,
            Option<WherePredicate>,
        ) = match options.kind {
            Kind::Value => (
                quote!(#decoder.value(#at)?),
                quote!(#encoder.value(#at, &self.#member, #space)?;),
                quote!({
                    assert!(::core::mem::size_of::<<#ty as ::wc3::model::common::AsULE>::ULE>() > 0,
                        "D4 value fields require a nonzero encoded width; use inline_array for empty arrays");
                    ::core::mem::size_of::<<#ty as ::wc3::model::common::AsULE>::ULE>()
                }),
                Some(parse_quote!(#ty: ::wc3::model::common::AsULE)),
            ),
            Kind::Inline => {
                let bound = match operation {
                    Operation::Read => parse_quote!(#ty: #codec::Read<#lifetime>),
                    Operation::Write => parse_quote!(#ty: #codec::Write),
                    Operation::Layout => parse_quote!(#ty: #codec::RecordLayout),
                };
                (
                    quote!(<#ty as #codec::Read<#lifetime>>::read(#decoder, #at)?),
                    quote!(<#ty as #codec::Write>::write(&self.#member, #encoder, #at, #space)?;),
                    quote!(<#ty as #codec::RecordLayout>::SIZE),
                    Some(bound),
                )
            }
            Kind::Array => {
                let (element, leaf) = array_element(ty)?;
                let bound = if leaf {
                    parse_quote!(#element: ::wc3::model::common::AsULE)
                } else {
                    match operation {
                        Operation::Read => parse_quote!(#element: #codec::Read<#lifetime>),
                        Operation::Write => parse_quote!(#element: #codec::Write),
                        Operation::Layout => parse_quote!(#element: #codec::RecordLayout),
                    }
                };
                let method = Ident::new(if leaf { "array" } else { "records" }, member.span());
                (
                    quote!(#decoder.#method(#at, #external)?),
                    quote!(#encoder.#method(#at, &self.#member, #space, #external)?;),
                    quote!(16usize),
                    Some(bound),
                )
            }
            Kind::InlineArray => {
                let Type::Array(array) = ty else {
                    return Err(Error::new_spanned(ty, "inline_array requires [Record; N]"));
                };
                let element = &array.elem;
                let length = &array.len;
                let bound = match operation {
                    Operation::Read => parse_quote!(#element: #codec::Read<#lifetime>),
                    Operation::Write => parse_quote!(#element: #codec::Write),
                    Operation::Layout => parse_quote!(#element: #codec::RecordLayout),
                };
                (
                    quote!(#decoder.fixed(#at)?),
                    quote!(#encoder.fixed(#at, &self.#member, #space)?;),
                    quote!(<#element as #codec::RecordLayout>::SIZE.checked_mul(#length).expect("D4 inline array width overflow")),
                    Some(bound),
                )
            }
            Kind::Bytes => (
                quote!(::std::borrow::Cow::Borrowed(#decoder.bytes(#at)?)),
                quote!(#encoder.bytes(#at, self.#member.as_ref(), #space)?;),
                quote!(16usize),
                None,
            ),
            Kind::Polymorphic(hash) => (
                quote!(#decoder.polymorphic(#at, #external, #hash)?),
                quote!(#encoder.polymorphic(#at, &self.#member, #space, #external, #hash)?;),
                quote!(24usize),
                None,
            ),
        };
        if let Some(bound) = bound.filter(|_| {
            let selected = match options.kind {
                Kind::Array => array_element(ty).map(|(element, _)| element).unwrap(),
                Kind::InlineArray => match ty {
                    Type::Array(array) => &array.elem,
                    _ => unreachable!(),
                },
                _ => ty,
            };
            !(matches!(options.kind, Kind::Array) && matches!(operation, Operation::Layout))
                && needs_bound(selected, &input)
        }) {
            generics.make_where_clause().predicates.push(bound);
        }
        if index + 1 < fields.named.len() {
            reads.push(quote!(#member: {
                let #value = #read;
                #cursor = #cursor.checked_add(const { #width }).ok_or(#codec::ReadError::Overflow)?;
                #value
            }));
            writes.push(quote! {
                #write
                #cursor = #cursor.checked_add(const { #width }).ok_or(#codec::WriteError::Overflow)?;
            });
        } else {
            reads.push(quote!(#member: #read));
            writes.push(write);
        }
        widths.push(width);
    }
    // Read/Write may also be derived against a handwritten layout implementation.
    if !matches!(operation, Operation::Layout) {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(Self: #codec::RecordLayout));
    }
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    let layout = checked_layout(&input, &widths, !matches!(operation, Operation::Layout));
    Ok(match operation {
        Operation::Read => quote! {
            impl #impl_generics #codec::Read<#lifetime> for #name #type_generics #where_clause {
                fn read(#decoder: &#codec::codec::Decoder<#lifetime>, #base: usize) -> ::core::result::Result<Self, #codec::ReadError> {
                    #decoder.range(#base, const #layout)?;
                    #cursor_start
                    Ok(Self { #(#reads),* })
                }
            }
        },
        Operation::Write => quote! {
            impl #impl_generics #codec::Write for #name #type_generics #where_clause {
                fn write(&self, #encoder: &mut #codec::codec::Encoder, #base: usize, #space: #codec::codec::Space) -> ::core::result::Result<(), #codec::WriteError> {
                    #encoder.range(#base, const #layout, #space)?;
                    #cursor_start
                    #(#writes)*
                    Ok(())
                }
            }
        },
        Operation::Layout => {
            let force = if input
                .generics
                .params
                .iter()
                .all(|p| matches!(p, GenericParam::Lifetime(_)))
            {
                let arguments = input
                    .generics
                    .lifetimes()
                    .map(|_| quote!('static))
                    .collect::<Vec<_>>();
                let target = if arguments.is_empty() {
                    quote!(#name)
                } else {
                    quote!(#name<#(#arguments),*>)
                };
                quote!(const _: () = { let _ = <#target as #codec::RecordLayout>::SIZE; };)
            } else {
                quote!()
            };
            quote! {
                impl #impl_generics #codec::RecordLayout for #name #type_generics #where_clause {
                    const SIZE: usize = #layout;
                }

                #force
            }
        }
    })
}

#[cfg(test)]
#[path = "d4_tests.rs"]
mod tests;
