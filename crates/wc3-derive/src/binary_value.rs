//! Scalar enum encodings with named choices and an exact unknown-value payload.
use crate::binary::{fresh_ident, Format, Operation};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Data, DeriveInput, Error, Expr, Fields, Result, Type};

pub(crate) struct Value {
    wire: Type,
    read: Vec<TokenStream>,
    write: Vec<TokenStream>,
    unknown: syn::Ident,
}
fn codec(format: Format) -> TokenStream {
    match format {
        Format::M2 => quote!(::wc3::model::formats::m2),
        Format::M3 => quote!(::wc3::model::formats::m3),
        Format::Wmo => quote!(::wc3::model::formats::wmo),
    }
}
fn attribute(format: Format) -> &'static str {
    match format {
        Format::M2 => "m2",
        Format::M3 => "m3",
        Format::Wmo => "wmo",
    }
}
fn parse(input: &DeriveInput, format: Format) -> Result<Value> {
    let namespace = attribute(format);
    let mut wire = None;
    for attr in input.attrs.iter().filter(|a| a.path().is_ident(namespace)) {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("value") || wire.is_some() {
                return Err(meta.error("expected one value = integer type"));
            }
            let ty: Type = meta.value()?.parse()?;
            if !matches!(&ty,Type::Path(p) if ["u8","u16","u32","u64","i8","i16","i32","i64"].iter().any(|t|p.path.is_ident(t))) {
                return Err(meta.error("enum wire type must be an integer scalar"));
            }
            wire = Some(ty);
            Ok(())
        })?;
    }
    let wire = wire.ok_or_else(|| {
        Error::new_spanned(
            input,
            "scalar enums require a format-qualified value = integer type attribute",
        )
    })?;
    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(input, "Value requires a scalar enum"));
    };
    let mut read = Vec::new();
    let mut write = Vec::new();
    let mut unknown = None;
    let mut codes = Vec::new();
    for variant in &data.variants {
        let mut code = None;
        let mut fallback = false;
        for attr in variant
            .attrs
            .iter()
            .filter(|a| a.path().is_ident(namespace))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("value") && code.is_none() && !fallback {
                    code = Some(meta.value()?.parse::<Expr>()?);
                    Ok(())
                } else if meta.path.is_ident("unknown") && code.is_none() && !fallback {
                    fallback = true;
                    Ok(())
                } else {
                    Err(meta.error("expected one value = integer or unknown attribute"))
                }
            })?;
        }
        let name = &variant.ident;
        if fallback {
            if unknown.is_some() {
                return Err(Error::new_spanned(
                    variant,
                    "only one unknown variant is allowed",
                ));
            }
            if !matches!(&variant.fields,Fields::Unnamed(f) if f.unnamed.len()==1&&{let ty=&f.unnamed[0].ty;quote!(#ty).to_string()==quote!(#wire).to_string()})
            {
                return Err(Error::new_spanned(
                    variant,
                    "unknown variant requires exactly one field of the wire type",
                ));
            }
            unknown = Some(name.clone());
            write.push(quote!(Self::#name(value)=>*value));
        } else {
            if !matches!(variant.fields, Fields::Unit) {
                return Err(Error::new_spanned(
                    variant,
                    "known scalar choices must be unit variants",
                ));
            }
            let code = code.ok_or_else(|| {
                Error::new_spanned(
                    variant,
                    "known choices require a format-qualified value attribute",
                )
            })?;
            let literal = match &code {
                Expr::Lit(l) => match &l.lit {
                    syn::Lit::Int(i) => Some(i.base10_parse::<i128>()?),
                    _ => None,
                },
                Expr::Unary(u) if matches!(u.op, syn::UnOp::Neg(_)) => match &*u.expr {
                    Expr::Lit(l) => match &l.lit {
                        syn::Lit::Int(i) => Some(-i.base10_parse::<i128>()?),
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            };
            let literal = literal
                .ok_or_else(|| Error::new_spanned(&code, "enum values must be integer literals"))?;
            let wire_name = quote!(#wire).to_string();
            let width: u32 = wire_name[1..].parse().expect("validated integer width");
            let (minimum, maximum) = if wire_name.starts_with('i') {
                (-(1i128 << (width - 1)), (1i128 << (width - 1)) - 1)
            } else {
                (0, (1i128 << width) - 1)
            };
            if !(minimum..=maximum).contains(&literal) {
                return Err(Error::new_spanned(
                    &code,
                    "enum wire value exceeds the declared integer range",
                ));
            }
            if codes.contains(&literal) {
                return Err(Error::new_spanned(&code, "duplicate enum wire value"));
            }
            codes.push(literal);
            read.push(quote!(#code=>Self::#name));
            write.push(quote!(Self::#name=>#code));
        }
    }
    Ok(Value {
        wire,
        read,
        write,
        unknown: unknown.ok_or_else(|| {
            Error::new_spanned(input, "scalar enums require an unknown-value variant")
        })?,
    })
}
pub(crate) fn expand(input: DeriveInput, format: Format) -> TokenStream {
    match value(&input, format) {
        Ok(t) => t,
        Err(e) => e.to_compile_error(),
    }
}
fn value(input: &DeriveInput, format: Format) -> Result<TokenStream> {
    let Value {
        wire,
        read,
        write,
        unknown,
    } = parse(input, format)?;
    let name = &input.ident;
    let (generics, types, where_clause) = input.generics.split_for_impl();
    Ok(quote! {
        impl #generics #name #types #where_clause {
            /// Converts a wire code to its named choice, retaining unrecognized values.
            pub const fn from_raw(value: #wire) -> Self {
                match value { #(#read,)* value => Self::#unknown(value) }
            }
            /// Returns the exact wire code, including unrecognized values.
            pub const fn raw(self) -> #wire {
                match &self { #(#write,)* }
            }
        }
        impl #generics ::core::convert::From<#wire> for #name #types #where_clause {
            fn from(value: #wire) -> Self { Self::from_raw(value) }
        }
        impl #generics ::core::convert::From<#name #types> for #wire #where_clause {
            fn from(value: #name #types) -> Self { value.raw() }
        }
    })
}
pub(crate) fn binary(
    input: &DeriveInput,
    operation: Operation,
    format: Format,
) -> Result<TokenStream> {
    let Value {
        wire,
        read,
        write,
        unknown,
    } = parse(input, format)?;
    let codec = codec(format);
    let name = &input.ident;
    let (_, types, _) = input.generics.split_for_impl();
    let mut generics = input.generics.clone();
    let lifetime = fresh_ident(input, "__binary_data");
    let lifetime = syn::Lifetime::new(&format!("'{lifetime}"), lifetime.span());
    if matches!(operation, Operation::Read) {
        generics.params.insert(0, parse_quote!(#lifetime));
    }
    let envelope = fresh_ident(input, "__BinaryEnvelope");
    let m3 = matches!(format, Format::M3);
    if m3 {
        generics
            .params
            .push(parse_quote!(#envelope:#codec::Envelope));
    }
    let envelope_args = if m3 { quote!(<#envelope>) } else { quote!() };
    let read_args = if m3 {
        quote!(#lifetime,#envelope)
    } else {
        quote!(#lifetime)
    };
    let decoder = if m3 {
        quote!(#codec::codec::Decoder<#lifetime,#envelope>)
    } else {
        quote!(#codec::codec::Decoder<#lifetime>)
    };
    let encoder = if m3 {
        quote!(#codec::codec::Encoder<#envelope>)
    } else {
        quote!(#codec::codec::Encoder)
    };
    let (generics, _, where_clause) = generics.split_for_impl();
    let m2 = matches!(format, Format::M2);
    let parameters = if m2 {
        quote!(version:u32,flags:u32)
    } else {
        quote!()
    };
    let arguments = if m2 { quote!(version, flags) } else { quote!() };
    let size = quote!(size);
    Ok(match operation {
        Operation::Read => {
            quote!(impl #generics #codec::Read<#read_args> for #name #types #where_clause {
                fn read(decoder: &mut #decoder) -> ::core::result::Result<Self, #codec::ReadError> {
                    let value = <#wire as #codec::Read<#read_args>>::read(decoder)?;
                    Ok(match value { #(#read,)* value => Self::#unknown(value) })
                }
            })
        }
        Operation::Write => {
            quote!(impl #generics #codec::Write #envelope_args for #name #types #where_clause {
                fn write(&self, encoder: &mut #encoder) -> ::core::result::Result<(), #codec::WriteError> {
                    let value: #wire = match self { #(#write,)* };
                    <#wire as #codec::Write #envelope_args>::write(&value, encoder)
                }
            })
        }
        Operation::Layout => {
            quote! {
                impl #generics #codec::RecordLayout #envelope_args for #name #types #where_clause {
                    fn #size(#parameters) -> usize {
                        <#wire as #codec::RecordLayout #envelope_args>::#size(#arguments)
                    }
                }
            }
        }
    })
}

#[cfg(test)]
#[path = "binary_value_tests.rs"]
mod tests;
