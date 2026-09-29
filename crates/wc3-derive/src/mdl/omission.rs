//! The writer's omission decision and the default restored by the reader.
use super::attributes::{Container, DefaultValue, Field, Kind};
use proc_macro2::TokenStream;
use quote::quote;

pub(super) fn needs_check(field: &Field) -> bool {
    matches!(field.kind, Kind::Animatable(_))
        || (field.skip_if.is_some()
            && matches!(field.kind, Kind::Property(_) | Kind::StaticProperty(_)))
}

pub(super) fn predicate(field: &Field) -> TokenStream {
    let access = field.borrow(quote!(self));
    let mut condition = if field.animation_only() {
        quote!(false)
    } else {
        quote!(true)
    };
    if field.slot.is_some() {
        let get = field.get.as_ref().expect("optional getter was checked");
        condition = quote!(#condition && #get(self).is_some());
    }
    if let Some(function) = &field.skip_if {
        condition = quote!(#condition && !#function(#access));
    }
    if matches!(field.kind, Kind::Animatable(_)) {
        condition = quote!(#condition && !::wc3::model::mdl::WriteAnimationProperty::has_animation(#access));
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
            field.value(quote!(&__wc3_mdl_write_defaults))
        }
    }
}
