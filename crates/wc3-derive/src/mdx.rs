//! Field-order binary codecs for MDX structs.
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote, Data, DeriveInput, Error, Expr, Fields, GenericArgument, Index, PathArguments,
    Result, Type,
};

fn sized_tag(input: &DeriveInput) -> Result<Option<Expr>> {
    let mut tag = None;
    for attr in &input.attrs {
        if !attr.path().is_ident("mdx") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("sized") {
                return Err(meta.error("expected sized(tag = ...)"));
            }
            if tag.is_some() {
                return Err(meta.error("duplicate sized record attribute"));
            }
            let mut sized_tag = None;
            meta.parse_nested_meta(|inner| {
                if !inner.path.is_ident("tag") {
                    return Err(inner.error("expected tag = ..."));
                }
                if sized_tag.is_some() {
                    return Err(inner.error("duplicate tag"));
                }
                sized_tag = Some(inner.value()?.parse()?);
                Ok(())
            })?;
            tag = Some(sized_tag.ok_or_else(|| meta.error("sized requires tag = ..."))?);
            Ok(())
        })?;
    }
    Ok(tag)
}

fn type_argument(ty: &Type, name: &str) -> Option<Type> {
    let Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    if segment.ident != name {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    if args.args.len() != 1 {
        return None;
    }
    let GenericArgument::Type(ty) = args.args.first()? else {
        return None;
    };
    Some(ty.clone())
}

pub(crate) fn expand(input: DeriveInput, reading: bool) -> TokenStream {
    match expand_checked(input, reading) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error(),
    }
}

fn expand_checked(input: DeriveInput, reading: bool) -> Result<TokenStream> {
    if matches!(input.data, Data::Enum(_)) {
        return expand_enum(&input, reading, false);
    }
    let tag = sized_tag(&input)?;
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return Err(Error::new_spanned(
                name,
                "codec derives support structs only",
            ))
        }
    };
    let field_count = fields.len();
    let mut generics = input.generics;
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    let mut names = Vec::new();
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
        let phantom = type_argument(ty, "PhantomData").is_some();
        let vec_item = type_argument(ty, "Vec");
        if vec_item.is_some() && (tag.is_none() || index + 1 != field_count) {
            return Err(Error::new_spanned(
                ty,
                "Vec<T> must be the final field of an #[mdx(sized(tag = ...))] struct",
            ));
        }
        if let Some(item) = vec_item {
            let predicate = if reading {
                parse_quote!(#item: ::wc3::model::mdx::Read)
            } else {
                parse_quote!(#item: ::wc3::model::mdx::Write)
            };
            generics.make_where_clause().predicates.push(predicate);
            reads.push(quote!({
                let mut values = ::std::vec::Vec::new();
                while !cursor.remaining().is_empty() {
                    let offset = cursor.absolute_position();
                    let value = cursor.read::<#item>()?;
                    if cursor.absolute_position() == offset {
                        return Err(::wc3::model::mdx::ReadError::MalformedRecord { tag: #tag, offset });
                    }
                    values.push(value);
                }
                values
            }));
            writes.push(quote!(for value in &self.#member { encoder.write(value)?; }));
        } else if phantom {
            reads.push(quote!(::std::marker::PhantomData));
        } else {
            let predicate = if reading {
                parse_quote!(#ty: ::wc3::model::mdx::Read)
            } else {
                parse_quote!(#ty: ::wc3::model::mdx::Write)
            };
            generics.make_where_clause().predicates.push(predicate);
            reads.push(quote!(cursor.read()?));
            writes.push(quote!(encoder.write(&self.#member)?;));
        }
        if let Some(ident) = &field.ident {
            names.push(ident.clone());
        }
    }
    let constructor = match fields {
        Fields::Named(_) => quote!(Self { #(#names: #reads,)* }),
        Fields::Unnamed(_) => quote!(Self( #(#reads,)* )),
        Fields::Unit => quote!(Self),
    };
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    if reading {
        let body = if tag.is_some() {
            quote! {
                let mut cursor = source.slice_u32_sized()?;
                let value = #constructor;
                cursor.finish()?;
                Ok(value)
            }
        } else {
            quote! {
                let cursor = source;
                Ok(#constructor)
            }
        };
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdx::Read for #name #ty_generics #where_clause {
                fn read_mdx(source: &mut ::wc3::model::mdx::Cursor<'_>) -> Result<Self, ::wc3::model::mdx::ReadError> {
                    #body
                }
            }
        })
    } else {
        let body = if let Some(tag) = tag {
            quote! {
                let marker = encoder.begin_sized();
                #(#writes)*
                encoder.finish_sized(marker, #tag)?;
            }
        } else {
            quote!(#(#writes)*)
        };
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdx::Write for #name #ty_generics #where_clause {
                fn write_mdx(&self, encoder: &mut ::wc3::model::mdx::Encoder<'_>) -> Result<(), ::wc3::model::mdx::WriteError> {
                    #body
                    Ok(())
                }
            }
        })
    }
}

pub(crate) fn expand_value(input: DeriveInput) -> TokenStream {
    if !matches!(input.data, Data::Enum(_)) {
        return Error::new_spanned(input, "Value requires a numeric enum").to_compile_error();
    }
    match expand_enum(&input, false, true) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error(),
    }
}

/// Numeric choices have an explicit wire representation independent of Rust layout.
fn expand_enum(input: &DeriveInput, reading: bool, conversions: bool) -> Result<TokenStream> {
    let mut wire = None;
    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("mdx"))
    {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("value") || wire.is_some() {
                return Err(meta.error("numeric enums require exactly one value = u32 attribute"));
            }
            let ty: Type = meta.value()?.parse()?;
            if !matches!(&ty, Type::Path(path) if path.path.is_ident("u32")) {
                return Err(meta.error("numeric enum wire type must be u32"));
            }
            wire = Some(ty);
            Ok(())
        })?;
    }
    let wire =
        wire.ok_or_else(|| Error::new_spanned(input, "numeric enums require #[mdx(value = u32)]"))?;
    let Data::Enum(data) = &input.data else {
        unreachable!()
    };
    let mut values = Vec::new();
    let mut read_arms = Vec::new();
    let mut write_arms = Vec::new();
    let mut unknown = None;
    for variant in &data.variants {
        let member = &variant.ident;
        let mut value = None;
        let mut fallback = false;
        for attr in variant
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("mdx"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("value") && value.is_none() && !fallback {
                    let literal: syn::LitInt = meta.value()?.parse()?;
                    let number = literal.base10_parse::<u32>()?;
                    if !literal.suffix().is_empty() && literal.suffix() != "u32" {
                        return Err(meta.error("wire values must be unsuffixed or u32 literals"));
                    }
                    value = Some(number);
                    Ok(())
                } else if meta.path.is_ident("unknown") && !fallback && value.is_none() {
                    fallback = true;
                    Ok(())
                } else {
                    Err(meta.error("expected one value = integer or unknown attribute"))
                }
            })?;
        }
        if variant.discriminant.is_some() {
            return Err(Error::new_spanned(
                variant,
                "use #[mdx(value = ...)] instead of Rust discriminants",
            ));
        }
        if fallback {
            if unknown.is_some() {
                return Err(Error::new_spanned(
                    variant,
                    "only one unknown variant is allowed",
                ));
            }
            if !matches!(&variant.fields, Fields::Unnamed(fields) if fields.unnamed.len() == 1 && matches!(&fields.unnamed[0].ty, Type::Path(path) if path.path.is_ident("u32")))
            {
                return Err(Error::new_spanned(
                    variant,
                    "unknown variant must contain exactly one u32 field",
                ));
            }
            unknown = Some(member);
            write_arms.push(quote!(Self::#member(value) => *value));
        } else {
            if !matches!(variant.fields, Fields::Unit) {
                return Err(Error::new_spanned(
                    variant,
                    "known numeric variants must be unit variants",
                ));
            }
            let value = value.ok_or_else(|| {
                Error::new_spanned(variant, "known variants require #[mdx(value = ...)]")
            })?;
            if values.contains(&value) {
                return Err(Error::new_spanned(
                    variant,
                    "duplicate numeric enum wire value",
                ));
            }
            values.push(value);
            read_arms.push(quote!(#value => Self::#member));
            write_arms.push(quote!(Self::#member => #value));
        }
    }
    if data.variants.is_empty() {
        return Err(Error::new_spanned(
            input,
            "numeric enums need at least one variant",
        ));
    }
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    if conversions {
        let (return_type, body) = if let Some(member) = unknown {
            (
                quote!(Self),
                quote!(match value { #(#read_arms,)* value => Self::#member(value) }),
            )
        } else {
            (
                quote!(::core::option::Option<Self>),
                quote!(::core::option::Option::Some(match value {
                    #(#read_arms,)* _ => return ::core::option::Option::None,
                })),
            )
        };
        return Ok(quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                /// Converts a binary value, preferring named variants.
                pub const fn from_raw(value: #wire) -> #return_type { #body }
                /// Returns the binary value, including an unknown payload verbatim.
                pub const fn raw(self) -> #wire { match &self { #(#write_arms,)* } }
            }
        });
    }
    if reading {
        let fallback = if let Some(member) = unknown {
            quote!(value => Self::#member(value))
        } else {
            quote!(value => return Err(::wc3::model::mdx::ReadError::UnknownEnumValue { enum_name: stringify!(#name), value, offset }))
        };
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdx::Read for #name #ty_generics #where_clause {
                fn read_mdx(cursor: &mut ::wc3::model::mdx::Cursor<'_>) -> Result<Self, ::wc3::model::mdx::ReadError> {
                    let offset = cursor.absolute_position();
                    let value = cursor.read::<#wire>()?;
                    let _ = offset;
                    Ok(match value { #(#read_arms,)* #fallback })
                }
            }
        })
    } else {
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdx::Write for #name #ty_generics #where_clause {
                fn write_mdx(&self, encoder: &mut ::wc3::model::mdx::Encoder<'_>) -> Result<(), ::wc3::model::mdx::WriteError> {
                    let value: #wire = match self { #(#write_arms,)* };
                    encoder.write(&value)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::expand_checked;
    use syn::parse_quote;

    #[test]
    fn rejects_invalid_numeric_enum_declarations() {
        let cases: [(syn::DeriveInput, &str); 9] = [
            (
                parse_quote!(
                    enum Bad {
                        A,
                    }
                ),
                "require #[mdx(value = u32)]",
            ),
            (
                parse_quote!(
                    #[mdx(value = u16)]
                    enum Bad {
                        #[mdx(value = 0)]
                        A,
                    }
                ),
                "wire type must be u32",
            ),
            (
                parse_quote!(
                    #[mdx(value = u32)]
                    enum Bad {
                        A,
                    }
                ),
                "known variants require",
            ),
            (
                parse_quote!(
                    #[mdx(value = u32)]
                    enum Bad {
                        #[mdx(value = 1)]
                        A,
                        #[mdx(value = 1)]
                        B,
                    }
                ),
                "duplicate numeric",
            ),
            (
                parse_quote!(
                    #[mdx(value = u32)]
                    enum Bad {
                        #[mdx(value = 4294967296)]
                        A,
                    }
                ),
                "number too large",
            ),
            (
                parse_quote!(
                    #[mdx(value = u32)]
                    enum Bad {
                        #[mdx(value = 0)]
                        A(u32),
                    }
                ),
                "must be unit",
            ),
            (
                parse_quote!(
                    #[mdx(value = u32)]
                    enum Bad {
                        #[mdx(unknown)]
                        A(u16),
                    }
                ),
                "exactly one u32",
            ),
            (
                parse_quote!(
                    #[mdx(value = u32)]
                    enum Bad {
                        #[mdx(unknown)]
                        A(u32),
                        #[mdx(unknown)]
                        B(u32),
                    }
                ),
                "only one unknown",
            ),
            (
                parse_quote!(
                    #[mdx(value = u32, sized(tag = TAG))]
                    enum Bad {
                        #[mdx(value = 0)]
                        A,
                    }
                ),
                "exactly one",
            ),
        ];
        for (input, message) in cases {
            for reading in [true, false] {
                let error = expand_checked(input.clone(), reading)
                    .unwrap_err()
                    .to_string();
                assert!(error.contains(message), "{error}");
            }
        }
    }
}
