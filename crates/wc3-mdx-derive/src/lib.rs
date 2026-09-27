//! Field-order codecs for MDX structs.
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Expr, Fields, GenericArgument, Index,
    PathArguments, Type,
};

/// Reads fields in declaration order. A sized record's final `Vec<T>` consumes its body.
#[proc_macro_derive(Readable, attributes(mdx))]
pub fn derive_readable(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput), true).into()
}

/// Writes fields in declaration order. A sized record's final `Vec<T>` is written elementwise.
#[proc_macro_derive(Writable, attributes(mdx))]
pub fn derive_writable(input: TokenStream) -> TokenStream {
    expand(parse_macro_input!(input as DeriveInput), false).into()
}

fn sized_tag(input: &DeriveInput) -> syn::Result<Option<Expr>> {
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

fn expand(input: DeriveInput, reading: bool) -> proc_macro2::TokenStream {
    match expand_checked(input, reading) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error(),
    }
}

fn expand_checked(input: DeriveInput, reading: bool) -> syn::Result<proc_macro2::TokenStream> {
    let tag = sized_tag(&input)?;
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => {
            return Err(syn::Error::new_spanned(
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
            return Err(syn::Error::new_spanned(
                ty,
                "Vec<T> must be the final field of an #[mdx(sized(tag = ...))] struct",
            ));
        }
        if let Some(item) = vec_item {
            let predicate = if reading {
                parse_quote!(#item: ::wc3_mdx::Readable)
            } else {
                parse_quote!(#item: ::wc3_mdx::Writable)
            };
            generics.make_where_clause().predicates.push(predicate);
            reads.push(quote!({
                let mut values = ::std::vec::Vec::new();
                while !cursor.remaining().is_empty() {
                    let offset = cursor.absolute_position();
                    let value = cursor.read::<#item>()?;
                    if cursor.absolute_position() == offset {
                        return Err(::wc3_mdx::DecodeError::MalformedRecord { tag: #tag, offset });
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
                parse_quote!(#ty: ::wc3_mdx::Readable)
            } else {
                parse_quote!(#ty: ::wc3_mdx::Writable)
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
            impl #impl_generics ::wc3_mdx::Readable for #name #ty_generics #where_clause {
                fn read_from(source: &mut ::wc3_mdx::Cursor<'_>) -> Result<Self, ::wc3_mdx::DecodeError> {
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
            impl #impl_generics ::wc3_mdx::Writable for #name #ty_generics #where_clause {
                fn write_to(&self, encoder: &mut ::wc3_mdx::Encoder<'_>) -> Result<(), ::wc3_mdx::EncodeError> {
                    #body
                    Ok(())
                }
            }
        })
    }
}
