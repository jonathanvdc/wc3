//! Scalar keyword enums and tagged records with deterministic name dispatch.
use super::attributes::identifier;
use super::sink_name;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse_quote, Data, DeriveInput, Error, Expr, Fields, Ident, Lit, LitStr, Path, Result, Type,
};

#[derive(Clone, Copy)]
enum Mode {
    Value,
    Tagged,
}
enum Kind {
    Value,
    Flag,
    Property,
    Block,
    Delegate,
}
struct Variant {
    member: Ident,
    name: Expr,
    kind: Kind,
    ty: Option<Type>,
}

pub(super) fn expand(input: &DeriveInput, reading: bool) -> Result<TokenStream> {
    let mut mode = None;
    let mut validate_read: Option<Path> = None;
    let mut validate_write: Option<Path> = None;
    for attr in &input.attrs {
        if !attr.path().is_ident("mdl") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("value") || meta.path.is_ident("tagged") {
                if mode.is_some() {
                    return Err(meta.error("choose exactly one of value or tagged"));
                }
                mode = Some(if meta.path.is_ident("value") {
                    Mode::Value
                } else {
                    Mode::Tagged
                });
                Ok(())
            } else if meta.path.is_ident("validate_read") || meta.path.is_ident("validate_write") {
                let target = if meta.path.is_ident("validate_read") {
                    &mut validate_read
                } else {
                    &mut validate_write
                };
                if target.is_some() {
                    return Err(meta.error("duplicate validation hook"));
                }
                let value: LitStr = meta.value()?.parse()?;
                *target = Some(value.parse()?);
                Ok(())
            } else {
                Err(meta.error("enums support value, tagged, validate_read, and validate_write"))
            }
        })?;
    }
    let mode =
        mode.ok_or_else(|| Error::new_spanned(&input.ident, "MDL enums require value or tagged"))?;
    let Data::Enum(data) = &input.data else {
        unreachable!()
    };
    if data.variants.is_empty() {
        return Err(Error::new_spanned(
            input,
            "MDL enums need at least one variant",
        ));
    }
    let mut variants = Vec::new();
    let mut literal_names = Vec::new();
    for variant in &data.variants {
        let mut name = None;
        let mut framing = None;
        let mut delegate = false;
        for attr in &variant.attrs {
            if !attr.path().is_ident("mdl") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("delegate") {
                    if delegate {
                        return Err(meta.error("duplicate delegate"));
                    }
                    delegate = true;
                    Ok(())
                } else if meta.path.is_ident("name")
                    || meta.path.is_ident("flag")
                    || meta.path.is_ident("property")
                    || meta.path.is_ident("block")
                {
                    if name.is_some() {
                        return Err(meta.error("choose exactly one variant name/framing attribute"));
                    }
                    let value: Expr = meta.value()?.parse()?;
                    if !match &value {
                        Expr::Path(_) => true,
                        Expr::Lit(lit) => matches!(lit.lit, Lit::Str(_)),
                        _ => false,
                    } {
                        return Err(
                            meta.error("variant names must be string literals or constant paths")
                        );
                    }
                    framing = Some(if meta.path.is_ident("flag") {
                        Kind::Flag
                    } else if meta.path.is_ident("property") {
                        Kind::Property
                    } else if meta.path.is_ident("block") {
                        Kind::Block
                    } else {
                        Kind::Value
                    });
                    name = Some(value);
                    Ok(())
                } else {
                    Err(meta.error("expected name, flag, property, block, or delegate"))
                }
            })?;
        }
        let ty = match &variant.fields {
            Fields::Unit => None,
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field = &fields.unnamed[0];
                if let Some(attr) = field.attrs.iter().find(|attr| attr.path().is_ident("mdl")) {
                    return Err(Error::new_spanned(
                        attr,
                        "enum payload fields do not support MDL attributes",
                    ));
                }
                Some(field.ty.clone())
            }
            _ => {
                return Err(Error::new_spanned(
                    variant,
                    "MDL enum variants must be unit or single-field tuple variants",
                ))
            }
        };
        let (name, kind) = match mode {
            Mode::Value => {
                if ty.is_some() || delegate || !matches!(framing, None | Some(Kind::Value)) {
                    return Err(Error::new_spanned(
                        variant,
                        "value enums support only unit variants with optional name attributes",
                    ));
                }
                let name = name.unwrap_or_else(|| {
                    let name = variant.ident.to_string();
                    let value = LitStr::new(
                        name.strip_prefix("r#").unwrap_or(&name),
                        variant.ident.span(),
                    );
                    parse_quote!(#value)
                });
                (name, Kind::Value)
            }
            Mode::Tagged => {
                let name = name.ok_or_else(|| {
                    Error::new_spanned(
                        variant,
                        "tagged variants require explicit name/framing attributes",
                    )
                })?;
                let kind = if delegate {
                    if !matches!(framing, Some(Kind::Value)) || ty.is_none() {
                        return Err(Error::new_spanned(
                            variant,
                            "delegate requires name and a single-field tuple variant",
                        ));
                    }
                    Kind::Delegate
                } else {
                    let kind = framing.expect("name and framing are paired");
                    match (&kind, &ty) {
                        (Kind::Flag, None) | (Kind::Property | Kind::Block, Some(_)) => {}
                        _ => return Err(Error::new_spanned(variant, "tagged variants require flag for unit variants, property/block for payloads, or name with delegate")),
                    }
                    kind
                };
                (name, kind)
            }
        };
        if let Expr::Lit(lit) = &name {
            if let Lit::Str(lit) = &lit.lit {
                identifier(lit)?;
                if literal_names.contains(&lit.value()) {
                    return Err(Error::new_spanned(lit, "duplicate MDL enum variant name"));
                }
                literal_names.push(lit.value());
            }
        }
        variants.push(Variant {
            member: variant.ident.clone(),
            name,
            kind,
            ty,
        });
    }
    let name = &input.ident;
    let names = variants
        .iter()
        .map(|variant| &variant.name)
        .collect::<Vec<_>>();
    let mut generics = input.generics.clone();
    for variant in &variants {
        if let Some(ty) = &variant.ty {
            let bound = match (&variant.kind, reading) {
                (Kind::Block, true) => parse_quote!(#ty: ::wc3::model::mdl::ReadFields),
                (Kind::Block, false) => parse_quote!(#ty: ::wc3::model::mdl::WriteFields),
                (_, true) => parse_quote!(#ty: ::wc3::model::mdl::Read),
                (_, false) => parse_quote!(#ty: ::wc3::model::mdl::Write),
            };
            generics.make_where_clause().predicates.push(bound);
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    if reading {
        let mut choices = Vec::new();
        for Variant {
            member, kind, ty, ..
        } in &variants
        {
            let read = match kind {
                Kind::Value => quote! { __wc3_mdl_parser.next_token()?; Self::#member },
                Kind::Flag => {
                    quote! { __wc3_mdl_parser.next_token()?; __wc3_mdl_parser.expect(::wc3::model::mdl::TokenKind::Comma)?; Self::#member }
                }
                Kind::Property => {
                    quote! { __wc3_mdl_parser.next_token()?; Self::#member(__wc3_mdl_parser.read_property::<#ty>()?) }
                }
                Kind::Block => {
                    quote! { __wc3_mdl_parser.next_token()?; Self::#member(::wc3::model::mdl::read_mdl_body::<#ty>(__wc3_mdl_parser, __wc3_mdl_token.span.start)?) }
                }
                Kind::Delegate => quote! { Self::#member(__wc3_mdl_parser.read::<#ty>()?) },
            };
            choices.push(quote! { { #read } });
        }
        let indexes = 0..choices.len();
        let validate = validate_read.map(|function| quote!(#function(&__wc3_mdl_value, ::wc3::model::mdl::Span::new(__wc3_mdl_token.span.start, __wc3_mdl_parser.position()))?;));
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdl::Read for #name #ty_generics #where_clause {
                fn read_mdl(__wc3_mdl_parser: &mut ::wc3::model::mdl::Parser<'_>) -> ::core::result::Result<Self, ::wc3::model::mdl::ReadError> {
                    let __wc3_mdl_names: &[&str] = &[#(#names,)*];
                    if !::wc3::model::mdl::enum_names_valid(__wc3_mdl_names) { return ::core::result::Result::Err(__wc3_mdl_parser.error(::wc3::model::mdl::ReadErrorKind::UnsupportedField)); }
                    let __wc3_mdl_token = __wc3_mdl_parser.peek()?.ok_or_else(|| __wc3_mdl_parser.error(::wc3::model::mdl::ReadErrorKind::Expected("an enum variant name")))?;
                    let ::wc3::model::mdl::TokenKind::Ident(__wc3_mdl_tag) = __wc3_mdl_token.kind else { return ::core::result::Result::Err(::wc3::model::mdl::ReadError::new(__wc3_mdl_token.span, ::wc3::model::mdl::ReadErrorKind::Expected("an enum variant name"))); };
                    let __wc3_mdl_value = match __wc3_mdl_names.iter().position(|__wc3_mdl_name| *__wc3_mdl_name == __wc3_mdl_tag) {
                        #(::core::option::Option::Some(#indexes) => #choices,)*
                        _ => return ::core::result::Result::Err(::wc3::model::mdl::ReadError::new(__wc3_mdl_token.span, ::wc3::model::mdl::ReadErrorKind::UnknownField)),
                    };
                    #validate
                    ::core::result::Result::Ok(__wc3_mdl_value)
                }
            }
        })
    } else {
        let sink = sink_name(input);
        let mut choices = Vec::new();
        for Variant {
            member, kind, ty, ..
        } in &variants
        {
            let pattern = if ty.is_some() {
                quote!(Self::#member(__wc3_mdl_value))
            } else {
                quote!(Self::#member)
            };
            let index = choices.len();
            let write = match kind {
                Kind::Value => quote!(__wc3_mdl_writer.identifier(__wc3_mdl_names[#index])),
                Kind::Flag => quote!(__wc3_mdl_writer.flag(__wc3_mdl_names[#index])),
                Kind::Property => {
                    quote!(__wc3_mdl_writer.property(__wc3_mdl_names[#index], __wc3_mdl_value))
                }
                Kind::Delegate => quote!(__wc3_mdl_writer.write(__wc3_mdl_value)),
                Kind::Block => quote! {
                    let __wc3_mdl_state = <#ty as ::wc3::model::mdl::WriteFields>::prepare_mdl_fields(__wc3_mdl_value, __wc3_mdl_writer.dialect())?;
                    __wc3_mdl_writer.indent()?;
                    __wc3_mdl_writer.identifier(__wc3_mdl_names[#index])?;
                    <#ty as ::wc3::model::mdl::WriteFields>::write_mdl_headers(__wc3_mdl_value, __wc3_mdl_writer)?;
                    __wc3_mdl_writer.open_body()?;
                    <#ty as ::wc3::model::mdl::WriteFields>::write_mdl_fields(__wc3_mdl_value, __wc3_mdl_state, __wc3_mdl_writer)?;
                    __wc3_mdl_writer.end_block()
                },
            };
            choices.push(quote!(#pattern => { #write }));
        }
        let validate = validate_write.map(|function| quote!(#function(self)?;));
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdl::Write for #name #ty_generics #where_clause {
                fn write_mdl<#sink: ::std::io::Write>(&self, __wc3_mdl_writer: &mut ::wc3::model::mdl::MdlWriter<#sink>) -> ::core::result::Result<(), ::wc3::model::mdl::WriteError> {
                    let __wc3_mdl_names: &[&str] = &[#(#names,)*];
                    if !::wc3::model::mdl::enum_names_valid(__wc3_mdl_names) { return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unsupported("invalid or duplicate MDL enum variant names")); }
                    #validate
                    match self { #(#choices,)* }
                }
            }
        })
    }
}
