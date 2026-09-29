//! Generation of record writers and preflight validation.
use super::attributes::{Container, Field, Kind};
use super::{bounds, omission, schema::Schema, sink_name, state_type};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, Result};

pub(super) fn expand(
    input: &DeriveInput,
    options: &Container,
    schema: &Schema,
) -> Result<TokenStream> {
    let block = options.block.as_ref();
    let name = &input.ident;
    let ordered = schema.ordered.iter().map(|index| &schema.fields[*index]);
    let write_defaults = bounds::uses_writer_defaults(options, &schema.fields);
    let generics = bounds::build(input, options, schema, false)?;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut headers = Vec::new();
    let mut writes = Vec::new();
    let mut required_flags = Vec::new();
    let mut state_names = Vec::new();
    let mut state_types = Vec::new();
    for field in ordered {
        let Field {
            kind, write_with, ..
        } = field;
        let default_access = field.value(quote!(&__wc3_mdl_write_defaults));
        let access = field.value(quote!(self));
        let member = &field.access();
        let value = match write_with {
            Some(function) => quote!(#function(&#access, __wc3_mdl_writer)?;),
            None => quote!(__wc3_mdl_writer.write(&#access)?;),
        };
        match kind {
            Kind::Header => {}
            Kind::Skip => {}
            Kind::Flatten => {
                let ty = &field.ty;
                let local = &field.local;
                state_names.push(local.clone());
                state_types.push(quote!(<#ty as ::wc3::model::mdl::WriteFields>::State));
                required_flags.push(quote!(let #local = ::wc3::model::mdl::WriteFields::prepare_mdl_fields(&#access, __wc3_mdl_dialect)?;));
                writes.push(quote!(::wc3::model::mdl::WriteFields::write_mdl_fields(&#access, #local, __wc3_mdl_writer)?;));
            }
            Kind::Block(mdl_name) => {
                let ty = &field.ty;
                let local = &field.local;
                state_names.push(local.clone());
                state_types.push(quote!(<#ty as ::wc3::model::mdl::WriteFields>::State));
                required_flags.push(quote!(let #local = ::wc3::model::mdl::WriteFields::prepare_mdl_fields(&#access, __wc3_mdl_dialect)?;));
                writes.push(quote! {
                    __wc3_mdl_writer.indent()?;
                    __wc3_mdl_writer.identifier(#mdl_name)?;
                    ::wc3::model::mdl::WriteFields::write_mdl_headers(&#access, __wc3_mdl_writer)?;
                    __wc3_mdl_writer.open_body()?;
                    ::wc3::model::mdl::WriteFields::write_mdl_fields(&#access, #local, __wc3_mdl_writer)?;
                    __wc3_mdl_writer.end_block()?;
                });
            }
            Kind::Repeated(_) => {
                if let Some(key) = &field.unique_by {
                    required_flags.push(quote! {
                        for (index, item) in (#access).iter().enumerate() {
                            if (#access).iter().take(index).any(|previous| #key(previous) == #key(item)) {
                                return Err(::wc3::model::mdl::WriteError::InvalidStructure { field: "duplicate repeated record" }.into());
                            }
                        }
                    });
                }
                if field.virtual_field {
                    writes.push(quote!(for item in (#access).iter() {
                        use ::wc3::model::mdl::Write as _;
                        item.write_mdl(__wc3_mdl_writer)?;
                    }));
                } else {
                    writes.push(
                        quote!(for item in (#access).iter() { __wc3_mdl_writer.write(item)?; }),
                    );
                }
            }
            Kind::Counted(mdl_name) => {
                writes.push(quote!(__wc3_mdl_writer.counted(#mdl_name, #access.iter())?;))
            }
            Kind::DelegatedProperty(mdl_name) => {
                let hive_name = field.hive_name.as_ref().unwrap_or(mdl_name);
                required_flags.push(quote!(::wc3::model::mdl::WriteProperty::validate_mdl_property(&#access, if __wc3_mdl_dialect == ::wc3::model::mdl::Dialect::HiveWorkshop { #hive_name } else { #mdl_name }, __wc3_mdl_dialect)?;));
                writes.push(quote!(::wc3::model::mdl::WriteProperty::write_mdl_property(&#access, if __wc3_mdl_writer.dialect() == ::wc3::model::mdl::Dialect::HiveWorkshop { #hive_name } else { #mdl_name }, __wc3_mdl_writer)?;));
            }
            Kind::Animatable(mdl_name) => {
                let access = field.borrow(quote!(self));
                let condition = omission::predicate(field);
                let default = omission::default_value(field, options);
                let enabled = field
                    .enabled_if
                    .as_ref()
                    .map(|function| quote!(#function(self)))
                    .unwrap_or_else(|| quote!(true));
                let available = field
                    .slot
                    .as_ref()
                    .map(|_| {
                        let get = field.get.as_ref().unwrap();
                        quote!(#get(self).is_some())
                    })
                    .unwrap_or_else(|| quote!(true));
                let emit = format_ident!("{}_emit", field.local);
                state_names.push(emit.clone());
                state_types.push(quote!(bool));
                required_flags.push(quote! {
                    let #emit = (#enabled && #available && ::wc3::model::mdl::WriteAnimationProperty::has_animation(#access)) || #condition;
                    if !#emit && !::wc3::model::mdl::ValueEq::eq_mdl(#access, &#default) {
                        return Err(::wc3::model::mdl::WriteError::Unrepresentable { field: concat!("nondefault omitted property ", #mdl_name) }.into());
                    }
                });
                writes.push(quote!(if #emit { ::wc3::model::mdl::WriteAnimationProperty::write_mdl_animation_property(#access, #mdl_name, __wc3_mdl_writer)?; }));
            }
            Kind::Property(mdl_name) | Kind::StaticProperty(mdl_name) => {
                let prefix = matches!(kind, Kind::StaticProperty(_))
                    .then(|| quote!(__wc3_mdl_writer.raw("static ")?;));
                let hive_name = field.hive_name.as_ref().unwrap_or(mdl_name);
                let write = quote! {
                    __wc3_mdl_writer.indent()?; #prefix
                    __wc3_mdl_writer.identifier(if __wc3_mdl_writer.dialect() == ::wc3::model::mdl::Dialect::HiveWorkshop { #hive_name } else { #mdl_name })?;
                    __wc3_mdl_writer.raw(" ")?; #value __wc3_mdl_writer.raw(",\n")?;
                };
                let mut condition = omission::predicate(field);
                if omission::needs_check(field) {
                    let emit = format_ident!("{}_emit", field.local);
                    state_names.push(emit.clone());
                    state_types.push(quote!(bool));
                    let default = omission::default_value(field, options);
                    required_flags.push(quote! {
                        let #emit = #condition;
                        if !#emit && !::wc3::model::mdl::ValueEq::eq_mdl(&#access, &#default) {
                            return Err(::wc3::model::mdl::WriteError::Unrepresentable { field: concat!("nondefault omitted property ", #mdl_name) }.into());
                        }
                    });
                    condition = quote!(#emit);
                }
                writes.push(quote!(if #condition { #write }));
            }
            Kind::Flags(flags) => {
                let known = flags
                    .iter()
                    .fold(field.allow_bits, |bits, (_, mask)| bits | mask);
                required_flags.push(quote! {
                        if ::wc3::model::mdl::BitRange::<u32>::bit_range(&#access, 31, 0) & !#known != 0 { return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unrepresentable { field: concat!("unknown flag bits in ", stringify!(#member)) }.into()); }
                    });
                if options.default && field.default.is_none() && !field.virtual_field {
                    required_flags.push(quote! {
                            let defaults = ::wc3::model::mdl::BitRange::<u32>::bit_range(&#default_access, 31, 0);
                            let actual = ::wc3::model::mdl::BitRange::<u32>::bit_range(&#access, 31, 0);
                            if defaults & !actual != 0 {
                                return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unrepresentable { field: "cleared flag supplied by record default" }.into());
                            }
                        });
                }
                // Dialects share the full semantic flag set. Overrides change
                // spellings only; unmapped names keep their common spelling.
                // Engine-only bits are omitted when the schema marks them for Hive.
                let hive = flags
                    .iter()
                    .filter(|(_, mask)| field.hive_skip_bits & mask == 0)
                    .map(|(name, mask)| {
                        field
                            .hive_flags
                            .as_ref()
                            .and_then(|overrides| overrides.iter().find(|(_, bit)| bit == mask))
                            .cloned()
                            .unwrap_or_else(|| (name.clone(), *mask))
                    })
                    .collect::<Vec<_>>();
                let emit = |mappings: &Vec<(syn::LitStr, u32)>| {
                    mappings.iter().map(|(name, mask)| quote!(if ::wc3::model::mdl::BitRange::<u32>::bit_range(&#access, 31, 0) & #mask != 0 { __wc3_mdl_writer.flag(#name)?; })).collect::<Vec<_>>()
                };
                let engine_writes = emit(flags);
                let hive_writes = emit(&hive);
                writes.push(quote! {
                    if __wc3_mdl_writer.dialect() == ::wc3::model::mdl::Dialect::HiveWorkshop { #(#hive_writes)* } else { #(#engine_writes)* }
                });
            }
            Kind::Flag(mdl_name) => {
                if field.default.is_none() && !options.default {
                    required_flags.push(quote! {
                        if !#access { return ::core::result::Result::Err(::wc3::model::mdl::WriteError::InvalidStructure { field: concat!("absent required flag ", #mdl_name) }.into()); }
                    });
                }
                if options.default && field.default.is_none() {
                    required_flags.push(quote! {
                            if #default_access && !#access {
                                return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unrepresentable { field: "false flag supplied by record default" }.into());
                            }
                        });
                }
                writes.push(quote!(if #access { __wc3_mdl_writer.flag(#mdl_name)?; }));
            }
        }
    }
    // Body write_order must not reorder flattened or direct header values.
    for field in &schema.fields {
        let access = field.value(quote!(self));
        match &field.kind {
            Kind::Header => {
                let value = match &field.write_with {
                    Some(function) => quote!(#function(&#access, __wc3_mdl_writer)?;),
                    None => quote!(__wc3_mdl_writer.write(&#access)?;),
                };
                headers.push(quote!(__wc3_mdl_writer.raw(" ")?; #value));
            }
            Kind::Flatten => {
                headers.push(quote!(::wc3::model::mdl::WriteFields::write_mdl_headers(&#access, __wc3_mdl_writer)?;));
            }
            _ => {}
        }
    }
    let validate = options
        .validate_write
        .as_ref()
        .map(|function| quote!(#function(self)?;));
    let initialize_defaults = write_defaults
        .then(|| quote!(let __wc3_mdl_write_defaults: Self = ::core::default::Default::default();));
    let sink = sink_name(input);
    let visit_names = schema.visit_names(false);
    let (state_name, state_definition) =
        state_type(input, &generics, "Write", &state_names, &state_types);
    let check_names = schema.fields.iter().any(|field| matches!(field.kind, Kind::Flatten)).then(|| quote! {
        if !::wc3::model::mdl::field_names_unique(<Self as ::wc3::model::mdl::WriteFields>::visit_mdl_names) {
            return Err(::wc3::model::mdl::WriteError::InvalidStructure { field: "overlapping flattened MDL field names" }.into());
        }
    });
    let write_impl = block.map(|block| quote! {
        impl #impl_generics ::wc3::model::mdl::Write for #name #ty_generics #where_clause {
            fn write_mdl<#sink: ::std::io::Write>(&self, __wc3_mdl_writer: &mut ::wc3::model::mdl::Writer<#sink>) -> ::core::result::Result<(), ::wc3::model::IoError<::wc3::model::mdl::WriteError>> {
                let state = <Self as ::wc3::model::mdl::WriteFields>::prepare_mdl_fields(self, __wc3_mdl_writer.dialect())?;
                __wc3_mdl_writer.indent()?;
                __wc3_mdl_writer.identifier(#block)?;
                <Self as ::wc3::model::mdl::WriteFields>::write_mdl_headers(self, __wc3_mdl_writer)?;
                __wc3_mdl_writer.open_body()?;
                <Self as ::wc3::model::mdl::WriteFields>::write_mdl_fields(self, state, __wc3_mdl_writer)?;
                __wc3_mdl_writer.end_block()
            }
        }
    });
    Ok(quote! {
        #state_definition
        impl #impl_generics ::wc3::model::mdl::WriteFields for #name #ty_generics #where_clause {
            type State = #state_name #ty_generics;
            fn visit_mdl_names(visitor: &mut dyn FnMut(&'static str, bool)) { #visit_names }
            fn write_mdl_headers<#sink: ::std::io::Write>(&self, __wc3_mdl_writer: &mut ::wc3::model::mdl::Writer<#sink>) -> ::core::result::Result<(), ::wc3::model::IoError<::wc3::model::mdl::WriteError>> {
                #(#headers)*
                Ok(())
            }
            fn prepare_mdl_fields(&self, __wc3_mdl_dialect: ::wc3::model::mdl::Dialect) -> ::core::result::Result<Self::State, ::wc3::model::mdl::WriteError> {
                #check_names
                #validate
                #initialize_defaults
                #(#required_flags)*
                Ok(#state_name { #(#state_names,)* __wc3_mdl_marker: ::core::marker::PhantomData })
            }
            fn write_mdl_fields<#sink: ::std::io::Write>(&self, state: Self::State, __wc3_mdl_writer: &mut ::wc3::model::mdl::Writer<#sink>) -> ::core::result::Result<(), ::wc3::model::IoError<::wc3::model::mdl::WriteError>> {
                let #state_name { #(#state_names,)* .. } = state;
                #(#writes)*
                Ok(())
            }
        }
        #write_impl
    })
}
