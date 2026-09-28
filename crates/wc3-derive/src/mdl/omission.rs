//! The writer's omission decision and the default restored by the reader.
use super::attributes::{Container, DefaultValue, Field, Kind};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn needs_check(field: &Field) -> bool {
    matches!(field.kind, Kind::Animatable(_))
        || (field.skip_if.is_some()
            && matches!(field.kind, Kind::Property(_) | Kind::StaticProperty(_)))
}

pub(super) fn predicate(field: &Field, tracks: Option<&Field>) -> TokenStream {
    let member = &field.member;
    let mut condition = quote!(true);
    if let Some(function) = &field.skip_if {
        condition = quote!(#condition && !#function(&self.#member));
    }
    if matches!(field.kind, Kind::Animatable(_)) {
        let variant = field.track.as_ref().expect("track was checked");
        let collection = &tracks.expect("tracks was checked").member;
        condition = quote!(#condition && !self.#collection.iter().any(|track| matches!(track, #variant(_))));
        if let Some(function) = &field.enabled_if {
            condition = quote!(#condition && #function(self));
        }
    }
    condition
}

pub(super) fn default_value(field: &Field, options: &Container) -> TokenStream {
    match &field.default {
        Some(DefaultValue::Trait) => {
            let ty = &field.ty;
            quote!(<#ty as ::core::default::Default>::default())
        }
        Some(DefaultValue::Function(function)) => quote!(#function()),
        None => {
            assert!(options.default && !field.required, "default was checked");
            let member = &field.member;
            quote!(__wc3_mdl_write_defaults.#member)
        }
    }
}
