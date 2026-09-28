//! Generation of direct streaming record readers.
use super::attributes::{Container, DefaultValue, Field, Kind};
use super::{bounds, schema::Schema};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Result};

pub(super) fn expand(
    input: &DeriveInput,
    options: &Container,
    schema: &Schema,
) -> Result<TokenStream> {
    let block = options.block.as_ref().expect("block was checked");
    let name = &input.ident;
    let fields = &schema.fields;
    let tracks = schema.tracks();
    let has_static = schema.has_static();
    let generics = bounds::build(input, options, schema, true)?;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // Runtime paths are qualified in generated code to avoid shadowing the
    // caller's field types, generic parameters, and imports.
    let mut headers = Vec::new();
    let mut locals = Vec::new();
    let mut arms = Vec::new();
    let mut static_arms = Vec::new();
    let mut enable_calls = Vec::new();
    let mut members = Vec::new();
    let mut bit = 0u32;
    for field in fields {
        let Field {
            member,
            local,
            ty,
            kind,
            default,
            read_with,
            ..
        } = field;
        let read = match read_with {
            Some(function) => quote!(#function(__wc3_mdl_parser)?),
            None => quote!(__wc3_mdl_parser.read::<#ty>()?),
        };
        if matches!(kind, Kind::Header) {
            headers.push(quote!(let #local: #ty = #read;));
            members.push(quote!(#member: #local));
            continue;
        }
        if matches!(kind, Kind::Tracks) {
            locals.push(quote!(let mut #local: #ty = ::std::vec::Vec::new();));
            members.push(quote!(#member: #local));
            continue;
        }
        if let Kind::Flags(flags) = kind {
            if options.default && default.is_none() {
                locals.push(quote!(let mut #local: #ty = __wc3_mdl_defaults.#member;));
            } else {
                locals.push(quote! {
                    let mut #local: #ty = ::core::default::Default::default();
                    ::wc3::model::mdl::BitRangeMut::<u32>::set_bit_range(&mut #local, 31, 0, 0);
                });
            }
            for (mdl_name, mask) in flags {
                arms.push(quote!(#mdl_name => {
                        __wc3_mdl_fields.mark(#bit, __wc3_mdl_field)?;
                        __wc3_mdl_body.expect(::wc3::model::mdl::TokenKind::Comma)?;
                        let __wc3_mdl_bits = ::wc3::model::mdl::BitRange::<u32>::bit_range(&#local, 31, 0);
                        ::wc3::model::mdl::BitRangeMut::<u32>::set_bit_range(&mut #local, 31, 0, __wc3_mdl_bits | #mask);
                    }));
                bit += 1;
            }
            members.push(quote!(#member: #local));
            continue;
        }
        let initial = match default {
            Some(DefaultValue::Trait) => Some(quote!(::core::default::Default::default())),
            Some(DefaultValue::Function(function)) => Some(quote!(#function())),
            None if options.default && !field.required => Some(quote!(__wc3_mdl_defaults.#member)),
            None => None,
        };
        let has_default = initial.is_some();
        if matches!(kind, Kind::Skip) {
            locals.push(quote!(let #local: #ty = #initial;));
            members.push(quote!(#member: #local));
            continue;
        }
        if let Some(initial) = initial {
            locals.push(quote!(let mut #local: #ty = #initial;));
        } else {
            locals.push(
                quote!(let mut #local: ::core::option::Option<#ty> = ::core::option::Option::None;),
            );
        }
        let mdl_name = match kind {
            Kind::Property(name)
            | Kind::StaticProperty(name)
            | Kind::Animatable(name)
            | Kind::Flag(name) => name,
            _ => unreachable!(),
        };
        let value = if matches!(kind, Kind::Flag(_)) {
            quote!({
                __wc3_mdl_body.expect(::wc3::model::mdl::TokenKind::Comma)?;
                true
            })
        } else {
            let read = match read_with {
                Some(function) => quote!(#function(&mut __wc3_mdl_body)?),
                None => quote!(__wc3_mdl_body.read::<#ty>()?),
            };
            quote!({ let value = #read; __wc3_mdl_body.expect(::wc3::model::mdl::TokenKind::Comma)?; value })
        };
        let assignment = if has_default {
            quote!(#local = #value;)
        } else {
            quote!(#local = ::core::option::Option::Some(#value);)
        };
        let mut mark_enabled = TokenStream::new();
        if let Some(function) = &field.enable_with {
            let enabled = format_ident!("{}_enabled", local);
            locals.push(quote!(let mut #enabled = false;));
            mark_enabled = quote!(#enabled = true;);
            enable_calls.push(quote!(if #enabled { #function(&mut __wc3_mdl_value); }));
        }
        let static_arm = quote!(#mdl_name => {
            __wc3_mdl_fields.mark(#bit, __wc3_mdl_field)?;
            #assignment
            #mark_enabled
        });
        match kind {
            Kind::StaticProperty(_) => static_arms.push(static_arm),
            Kind::Animatable(_) => {
                static_arms.push(static_arm);
                let variant = field.track.as_ref().expect("track was checked");
                let collection = &tracks.expect("tracks was checked").local;
                arms.push(quote!(#mdl_name => {
                    __wc3_mdl_fields.mark(#bit, __wc3_mdl_field)?;
                    *__wc3_mdl_body = __wc3_mdl_checkpoint;
                    #collection.push(#variant(__wc3_mdl_body.read()?));
                    #mark_enabled
                }));
            }
            _ => arms.push(static_arm),
        }
        if has_default {
            members.push(quote!(#member: #local));
        } else {
            members.push(quote!(#member: #local.ok_or_else(|| __wc3_mdl_body.error(::wc3::model::mdl::ReadErrorKind::MissingField(#mdl_name)))?));
        }
        bit += 1;
    }
    let capture_start = options.validate_read.as_ref().map(|_| {
        quote! {
            __wc3_mdl_parser.peek()?;
            let __wc3_mdl_start = __wc3_mdl_parser.position();
        }
    });
    let validate = options.validate_read.as_ref().map(|function| quote! {
            #function(&__wc3_mdl_value, ::wc3::model::mdl::Span::new(__wc3_mdl_start, __wc3_mdl_parser.position()))?;
        });
    let field_tracking = (!arms.is_empty() || !static_arms.is_empty())
        .then(|| quote!(let mut __wc3_mdl_fields = ::wc3::model::mdl::Fields::default();));
    let static_dispatch = has_static.then(|| quote! {
            "static" => {
                let token = __wc3_mdl_body.next_token()?;
                let __wc3_mdl_field = match token.kind {
                    ::wc3::model::mdl::TokenKind::Ident(name) => ::wc3::model::mdl::Field { name, span: token.span },
                    _ => return ::core::result::Result::Err(::wc3::model::mdl::ReadError::new(token.span, ::wc3::model::mdl::ReadErrorKind::Expected("a static property name"))),
                };
                match __wc3_mdl_field.name {
                    #(#static_arms,)*
                    _ => return ::core::result::Result::Err(::wc3::model::mdl::ReadError::new(__wc3_mdl_field.span, ::wc3::model::mdl::ReadErrorKind::UnknownField)),
                }
            },
        });
    let initialize_defaults = options
        .default
        .then(|| quote!(let __wc3_mdl_defaults: Self = ::core::default::Default::default();));
    let mutable_value = (!enable_calls.is_empty()).then(|| quote!(mut));
    Ok(quote! {
        impl #impl_generics ::wc3::model::mdl::Read for #name #ty_generics #where_clause {
            fn read_mdl(__wc3_mdl_parser: &mut ::wc3::model::mdl::Parser<'_>) -> ::core::result::Result<Self, ::wc3::model::mdl::ReadError> {
                #capture_start
                __wc3_mdl_parser.expect_ident(#block)?;
                #(#headers)*
                #initialize_defaults
                #(#locals)*
                #field_tracking
                let mut __wc3_mdl_body = __wc3_mdl_parser.begin_block()?;
                loop {
                    let __wc3_mdl_checkpoint = *__wc3_mdl_body;
                    let ::core::option::Option::Some(__wc3_mdl_field) = __wc3_mdl_body.next_field()? else { break; };
                    match __wc3_mdl_field.name {
                        #static_dispatch
                        #(#arms,)*
                        _ => return ::core::result::Result::Err(::wc3::model::mdl::ReadError::new(__wc3_mdl_field.span, ::wc3::model::mdl::ReadErrorKind::UnknownField)),
                    }
                }
                let #mutable_value __wc3_mdl_value = Self { #(#members,)* };
                #(#enable_calls)*
                __wc3_mdl_body.finish()?;
                #validate
                ::core::result::Result::Ok(__wc3_mdl_value)
            }
        }
    })
}
