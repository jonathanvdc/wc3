//! Generation of record writers and preflight validation.
use super::attributes::{Container, Field, Kind};
use super::{bounds, omission, schema::Schema, sink_name};
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
    let ordered = schema.ordered.iter().map(|index| &schema.fields[*index]);
    let tracks = schema.tracks();
    let animated = schema.animated().collect::<Vec<_>>();
    let write_defaults = bounds::uses_writer_defaults(options, &schema.fields);
    let generics = bounds::build(input, options, schema, false)?;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut headers = Vec::new();
    let mut writes = Vec::new();
    let mut required_flags = Vec::new();
    for field in ordered {
        let Field {
            member,
            kind,
            write_with,
            ..
        } = field;
        let value = match write_with {
            Some(function) => quote!(#function(&self.#member, __wc3_mdl_writer)?;),
            None => quote!(__wc3_mdl_writer.write(&self.#member)?;),
        };
        match kind {
            Kind::Header => headers.push(quote! { __wc3_mdl_writer.raw(" ")?; #value }),
            Kind::Skip => {}
            Kind::DelegatedProperty(mdl_name) => {
                let ty = &field.ty;
                required_flags.push(quote!(<#ty as ::wc3::model::mdl::WriteProperty>::validate_mdl_property(&self.#member, #mdl_name)?;));
                writes.push(quote!(<#ty as ::wc3::model::mdl::WriteProperty>::write_mdl_property(&self.#member, #mdl_name, __wc3_mdl_writer)?;));
            }
            Kind::Property(mdl_name)
            | Kind::StaticProperty(mdl_name)
            | Kind::Animatable(mdl_name) => {
                let prefix = (!matches!(kind, Kind::Property(_)))
                    .then(|| quote!(__wc3_mdl_writer.raw("static ")?;));
                let write = quote! {
                    __wc3_mdl_writer.indent()?;
                    #prefix
                    __wc3_mdl_writer.identifier(#mdl_name)?;
                    __wc3_mdl_writer.raw(" ")?;
                    #value
                    __wc3_mdl_writer.raw(",\n")?;
                };
                let mut condition = omission::predicate(field, tracks);
                if omission::needs_check(field) {
                    let emit = format_ident!("{}_emit", field.local);
                    let default = omission::default_value(field, options);
                    required_flags.push(quote! {
                        let #emit = #condition;
                        if !#emit && !::wc3::model::mdl::ValueEq::eq_mdl(&self.#member, &#default) {
                            return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unsupported(concat!("nondefault omitted property ", #mdl_name)));
                        }
                    });
                    condition = quote!(#emit);
                }
                writes.push(quote!(if #condition { #write }));
            }
            Kind::Tracks => {
                writes.push(quote!(for track in &self.#member { __wc3_mdl_writer.write(track)?; }));
                let mut checks = Vec::new();
                let mut choices = Vec::new();
                for animated in &animated {
                    let variant = animated.track.as_ref().expect("track was checked");
                    checks.push(quote! {
                            if self.#member.iter().filter(|track| matches!(track, #variant(_))).count() > 1 {
                                return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unsupported("duplicate animation track"));
                            }
                        });
                    if let Some(function) = &animated.enabled_if {
                        checks.push(quote! {
                                if !#function(self) && self.#member.iter().any(|track| matches!(track, #variant(_))) {
                                    return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unsupported("animation track for a disabled property"));
                                }
                            });
                    }
                    choices.push(quote!(#variant(_) => {}));
                }
                required_flags.push(quote! {
                        #(#checks)*
                        for track in &self.#member {
                            #[allow(unreachable_patterns)]
                            match track {
                                #(#choices,)*
                                _ => return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unsupported("unmapped animation track")),
                            }
                        }
                    });
            }
            Kind::Flags(flags) => {
                let known = flags
                    .iter()
                    .fold(field.allow_bits, |bits, (_, mask)| bits | mask);
                required_flags.push(quote! {
                        if ::wc3::model::mdl::BitRange::<u32>::bit_range(&self.#member, 31, 0) & !#known != 0 { return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unsupported(concat!("unknown flag bits in ", stringify!(#member)))); }
                    });
                if options.default && field.default.is_none() {
                    required_flags.push(quote! {
                            let defaults = ::wc3::model::mdl::BitRange::<u32>::bit_range(&__wc3_mdl_write_defaults.#member, 31, 0);
                            let actual = ::wc3::model::mdl::BitRange::<u32>::bit_range(&self.#member, 31, 0);
                            if defaults & !actual != 0 {
                                return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unsupported("cleared flag supplied by record default"));
                            }
                        });
                }
                for (mdl_name, mask) in flags {
                    writes.push(quote!(if ::wc3::model::mdl::BitRange::<u32>::bit_range(&self.#member, 31, 0) & #mask != 0 { __wc3_mdl_writer.flag(#mdl_name)?; }));
                }
            }
            Kind::Flag(mdl_name) => {
                if field.default.is_none() && !options.default {
                    required_flags.push(quote! {
                        if !self.#member { return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unsupported(concat!("absent required flag ", #mdl_name))); }
                    });
                }
                if options.default && field.default.is_none() {
                    required_flags.push(quote! {
                            if __wc3_mdl_write_defaults.#member && !self.#member {
                                return ::core::result::Result::Err(::wc3::model::mdl::WriteError::Unsupported("false flag supplied by record default"));
                            }
                        });
                }
                writes.push(quote!(if self.#member { __wc3_mdl_writer.flag(#mdl_name)?; }));
            }
        }
    }
    let validate = options
        .validate_write
        .as_ref()
        .map(|function| quote!(#function(self)?;));
    let initialize_defaults = write_defaults
        .then(|| quote!(let __wc3_mdl_write_defaults: Self = ::core::default::Default::default();));
    let sink = sink_name(input);
    Ok(quote! {
        impl #impl_generics ::wc3::model::mdl::Write for #name #ty_generics #where_clause {
            fn write_mdl<#sink: ::std::io::Write>(&self, __wc3_mdl_writer: &mut ::wc3::model::mdl::MdlWriter<#sink>) -> ::core::result::Result<(), ::wc3::model::mdl::WriteError> {
                #validate
                #initialize_defaults
                #(#required_flags)*
                __wc3_mdl_writer.indent()?;
                __wc3_mdl_writer.identifier(#block)?;
                #(#headers)*
                __wc3_mdl_writer.open_body()?;
                #(#writes)*
                __wc3_mdl_writer.end_block()
            }
        }
    })
}
