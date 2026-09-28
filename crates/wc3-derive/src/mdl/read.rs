//! Generation of direct streaming record readers.
use super::attributes::{vec_element, Container, DefaultValue, Field, Kind};
use super::{bounds, schema::Schema, state_type};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Lifetime, Result};

pub(super) fn expand(
    input: &DeriveInput,
    options: &Container,
    schema: &Schema,
) -> Result<TokenStream> {
    let block = options.block.as_ref();
    let name = &input.ident;
    let mut lifetime_name = String::from("__wc3_mdl_source");
    while input
        .generics
        .lifetimes()
        .any(|param| param.lifetime.ident == lifetime_name)
    {
        lifetime_name.push('_');
    }
    let source_lifetime = Lifetime::new(&format!("'{lifetime_name}"), input.ident.span());
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
    let mut flattened = Vec::new();
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
        if matches!(kind, Kind::Flatten) {
            headers.push(quote!(let mut #local: <#ty as ::wc3::model::mdl::ReadFields>::State = <#ty as ::wc3::model::mdl::ReadFields>::begin_mdl_fields(__wc3_mdl_parser)?;));
            members.push(quote!(#member: <#ty as ::wc3::model::mdl::ReadFields>::finish_mdl_fields(#local, __wc3_mdl_span, __wc3_mdl_record_span)?));
            flattened.push(field);
            continue;
        }
        if let Kind::Repeated(mdl_name) = kind {
            let element = vec_element(ty)?;
            locals.push(quote!(let mut #local: #ty = ::std::vec::Vec::new();));
            arms.push(quote!(#mdl_name => {
                *__wc3_mdl_body = __wc3_mdl_checkpoint;
                let start = __wc3_mdl_body.position();
                let value = __wc3_mdl_body.read::<#element>()?;
                if __wc3_mdl_body.position() == start { return Err(__wc3_mdl_body.error(::wc3::model::mdl::ReadErrorKind::NoProgress)); }
                #local.push(value);
            }));
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
        if let Kind::DelegatedProperty(mdl_name) = kind {
            locals.push(
                quote!(let mut #local: ::core::option::Option<#ty> = ::core::option::Option::None;),
            );
            arms.push(quote!(#mdl_name => {
                __wc3_mdl_fields.mark(#bit, __wc3_mdl_field)?;
                #local = ::core::option::Option::Some(<#ty as ::wc3::model::mdl::ReadProperty>::read_mdl_property(__wc3_mdl_body, __wc3_mdl_field)?);
            }));
            members.push(quote!(#member: match #local {
                ::core::option::Option::Some(value) => value,
                ::core::option::Option::None => <#ty as ::wc3::model::mdl::ReadProperty>::missing_mdl_property(#mdl_name, ::wc3::model::mdl::ReadError::new(__wc3_mdl_span, ::wc3::model::mdl::ReadErrorKind::MissingField(#mdl_name)).span)?,
            }));
            bit += 1;
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
            Kind::Block(name)
            | Kind::Counted(name)
            | Kind::Property(name)
            | Kind::StaticProperty(name)
            | Kind::Animatable(name)
            | Kind::Flag(name) => name,
            _ => unreachable!(),
        };
        let value = if let Kind::Counted(_) = kind {
            let element = vec_element(ty)?;
            quote!(__wc3_mdl_body.counted::<#element>()?.collect::<::core::result::Result<::std::vec::Vec<_>, _>>()?)
        } else if matches!(kind, Kind::Block(_)) {
            quote!(::wc3::model::mdl::read_mdl_body::<#ty>(__wc3_mdl_body, __wc3_mdl_field.span.start)?)
        } else if matches!(kind, Kind::Flag(_)) {
            quote!({
                __wc3_mdl_body.expect(::wc3::model::mdl::TokenKind::Comma)?;
                true
            })
        } else {
            let read = match read_with {
                Some(function) => quote!(#function(__wc3_mdl_body)?),
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
            members.push(quote!(#member: #local.ok_or_else(|| ::wc3::model::mdl::ReadError::new(__wc3_mdl_span, ::wc3::model::mdl::ReadErrorKind::MissingField(#mdl_name)))?));
        }
        bit += 1;
    }
    let capture_start = quote! {
        __wc3_mdl_parser.peek()?;
        let __wc3_mdl_start = __wc3_mdl_parser.position();
    };
    let validate = options
        .validate_read
        .as_ref()
        .map(|function| quote!(#function(&__wc3_mdl_value, __wc3_mdl_record_span)?;));
    let mut state_names = Vec::new();
    let mut state_types = Vec::new();
    for field in fields {
        let local = &field.local;
        let ty = &field.ty;
        let has_default = field.default.is_some() || (options.default && !field.required);
        let state_type = match &field.kind {
            Kind::Flatten => quote!(<#ty as ::wc3::model::mdl::ReadFields>::State),
            Kind::DelegatedProperty(_) => quote!(::core::option::Option<#ty>),
            Kind::Header | Kind::Flags(_) | Kind::Tracks | Kind::Repeated(_) | Kind::Skip => {
                quote!(#ty)
            }
            _ if has_default => quote!(#ty),
            _ => quote!(::core::option::Option<#ty>),
        };
        state_names.push(local.clone());
        state_types.push(state_type);
        if field.enable_with.is_some() {
            state_names.push(format_ident!("{}_enabled", local));
            state_types.push(quote!(bool));
        }
    }
    state_names.push(format_ident!("__wc3_mdl_fields"));
    state_types.push(quote!(::wc3::model::mdl::Fields));
    let (state_name, state_definition) =
        state_type(input, &generics, "Read", &state_names, &state_types);
    let state_values = quote!(#(#state_names,)*);
    let state_bindings = quote!(#(#state_names: mut #state_names,)*);
    let visit_names = schema.visit_names(true);
    let accepts = schema.accepts();
    let fallback = flattened.iter().map(|field| {
        let local = &field.local;
        let ty = &field.ty;
        quote! {
            if <#ty as ::wc3::model::mdl::ReadFields>::accepts_mdl_field(__wc3_mdl_dispatch_name, __wc3_mdl_static_form) {
                *__wc3_mdl_body = __wc3_mdl_checkpoint;
                __wc3_mdl_body.next_token()?;
                #local = <#ty as ::wc3::model::mdl::ReadFields>::read_mdl_field(#local, __wc3_mdl_body, __wc3_mdl_original_field, __wc3_mdl_checkpoint)?;
                return ::core::result::Result::Ok(#state_name { #state_values __wc3_mdl_marker: ::core::marker::PhantomData });
            }
        }
    });
    let initialize_defaults = options
        .default
        .then(|| quote!(let __wc3_mdl_defaults: Self = ::core::default::Default::default();));
    let mutable_value = (!enable_calls.is_empty()).then(|| quote!(mut));
    let fallback_tokens = quote! {
        #(#fallback)*
        return ::core::result::Result::Err(::wc3::model::mdl::ReadError::new(__wc3_mdl_field.span, ::wc3::model::mdl::ReadErrorKind::UnknownField));
    };
    let static_dispatch = (has_static || !flattened.is_empty()).then(|| quote! {
        "static" if !<Self as ::wc3::model::mdl::ReadFields>::accepts_mdl_field("static", false) => {
            let token = __wc3_mdl_body.next_token()?;
            let __wc3_mdl_field = match token.kind {
                ::wc3::model::mdl::TokenKind::Ident(name) => ::wc3::model::mdl::Field { name, span: token.span },
                _ => return ::core::result::Result::Err(::wc3::model::mdl::ReadError::new(token.span, ::wc3::model::mdl::ReadErrorKind::Expected("a static property name"))),
            };
            match __wc3_mdl_field.name {
                #(#static_arms,)*
                _ => { #fallback_tokens }
            }
        },
    });
    let check_names = (!flattened.is_empty()).then(|| quote! {
        if !::wc3::model::mdl::field_names_unique(<Self as ::wc3::model::mdl::ReadFields>::visit_mdl_names) {
            return Err(__wc3_mdl_parser.error(::wc3::model::mdl::ReadErrorKind::DuplicateField));
        }
    });
    let read_impl = block.map(|block| quote! {
        impl #impl_generics ::wc3::model::mdl::Read for #name #ty_generics #where_clause {
            fn read_mdl(__wc3_mdl_parser: &mut ::wc3::model::mdl::Parser<'_>) -> ::core::result::Result<Self, ::wc3::model::mdl::ReadError> {
                #capture_start
                __wc3_mdl_parser.expect_ident(#block)?;
                let __wc3_mdl_value = ::wc3::model::mdl::read_mdl_body::<Self>(__wc3_mdl_parser, __wc3_mdl_start)?;
                ::core::result::Result::Ok(__wc3_mdl_value)
            }
        }
    });

    Ok(quote! {
        #state_definition
        impl #impl_generics ::wc3::model::mdl::ReadFields for #name #ty_generics #where_clause {
            type State = #state_name #ty_generics;
            fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) { #visit_names }
            fn accepts_mdl_field(name: &str, static_form: bool) -> bool { #accepts }
            fn begin_mdl_fields(__wc3_mdl_parser: &mut ::wc3::model::mdl::Parser<'_>) -> ::core::result::Result<Self::State, ::wc3::model::mdl::ReadError> {
                #check_names
                #initialize_defaults
                #(#headers)*
                #(#locals)*
                let __wc3_mdl_fields = ::wc3::model::mdl::Fields::default();
                Ok(#state_name { #state_values __wc3_mdl_marker: ::core::marker::PhantomData })
            }
            #[allow(unreachable_code)]
            fn read_mdl_field<#source_lifetime>(state: Self::State, __wc3_mdl_body: &mut ::wc3::model::mdl::Parser<#source_lifetime>, __wc3_mdl_field: ::wc3::model::mdl::Field<#source_lifetime>, __wc3_mdl_checkpoint: ::wc3::model::mdl::Parser<#source_lifetime>) -> ::core::result::Result<Self::State, ::wc3::model::mdl::ReadError> {
                #[allow(unused_mut)]
                let #state_name { #state_bindings .. } = state;
                let __wc3_mdl_original_field = __wc3_mdl_field;
                let (__wc3_mdl_dispatch_name, __wc3_mdl_static_form) = if <Self as ::wc3::model::mdl::ReadFields>::accepts_mdl_field("static", false) {
                    (__wc3_mdl_field.name, false)
                } else { ::wc3::model::mdl::dispatch_name(__wc3_mdl_field, __wc3_mdl_checkpoint)? };
                match __wc3_mdl_field.name {
                    #static_dispatch
                    #(#arms,)*
                    _ => { #fallback_tokens }
                }
                Ok(#state_name { #state_values __wc3_mdl_marker: ::core::marker::PhantomData })
            }
            fn finish_mdl_fields(state: Self::State, __wc3_mdl_span: ::wc3::model::mdl::Span, __wc3_mdl_record_span: ::wc3::model::mdl::Span) -> ::core::result::Result<Self, ::wc3::model::mdl::ReadError> {
                let #state_name { #state_values .. } = state;
                let #mutable_value __wc3_mdl_value = Self { #(#members,)* };
                #(#enable_calls)*
                #validate
                Ok(__wc3_mdl_value)
            }
        }
        #read_impl
    })
}
