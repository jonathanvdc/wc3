//! Validation across fields and the normalized layout used by code generation.
use super::attributes::{field, Container, Field, Kind};
use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{
    parenthesized, punctuated::Punctuated, Data, DeriveInput, Error, Field as SynField, Fields,
    Ident, PathArguments, Result, Token, Type,
};

pub(super) struct Projection {
    pub(super) member: Ident,
    pub(super) ty: Type,
}
pub(super) struct Schema {
    pub(super) fields: Vec<Field>,
    pub(super) ordered: Vec<usize>,
    pub(super) projections: Vec<Projection>,
}

impl Schema {
    pub(super) fn visit_names(&self, reading: bool) -> TokenStream {
        let mut calls = Vec::new();
        for field in &self.fields {
            if let Some(extra) = &field.extra_flags {
                for (name, _) in &extra.flags {
                    calls.push(quote!(visitor(#name, false);));
                }
            }
            for (name, _) in &field.channels {
                calls.push(quote!(visitor(#name, false);));
            }
            match &field.kind {
                Kind::Flatten => {
                    let ty = &field.ty;
                    let trait_name = if reading {
                        quote!(::wc3::model::mdl::ReadFields)
                    } else {
                        quote!(::wc3::model::mdl::WriteFields)
                    };
                    calls.push(quote!(<#ty as #trait_name>::visit_mdl_names(visitor);));
                }
                Kind::Property(name)
                | Kind::DelegatedProperty(name)
                | Kind::StaticProperty(name)
                | Kind::Animatable(name)
                | Kind::Flag(name)
                | Kind::Block(name)
                | Kind::Counted(name) => {
                    let static_form =
                        matches!(field.kind, Kind::StaticProperty(_) | Kind::Animatable(_))
                            && !field.animated_only;
                    calls.push(quote!(visitor(#name, #static_form);));
                }
                Kind::Repeated(names) => {
                    for name in names {
                        calls.push(quote!(visitor(#name, false);));
                    }
                }
                Kind::Flags(flags) => {
                    for (name, _) in flags {
                        calls.push(quote!(visitor(#name, false);));
                    }
                }
                _ => {}
            }
        }
        quote!(#(#calls)*)
    }
    pub(super) fn accepts(&self) -> TokenStream {
        let mut conditions = Vec::new();
        for field in &self.fields {
            if let Some(extra) = &field.extra_flags {
                for (value, _) in &extra.flags {
                    conditions.push(quote!(!static_form && name == #value));
                }
            }
            for (value, _) in &field.channels {
                conditions.push(quote!(!static_form && name == #value));
            }
            match &field.kind {
                Kind::Flatten => {
                    let ty = &field.ty;
                    conditions.push(quote!(<#ty as ::wc3::model::mdl::ReadFields>::accepts_mdl_field(name, static_form)));
                }
                Kind::StaticProperty(value) => {
                    conditions.push(quote!(static_form && name == #value))
                }
                Kind::Animatable(value) => {
                    let allow_static = !field.animated_only;
                    conditions.push(quote!(name == #value && (!static_form || #allow_static)));
                }
                Kind::Property(value)
                | Kind::DelegatedProperty(value)
                | Kind::Flag(value)
                | Kind::Block(value)
                | Kind::Counted(value) => conditions.push(quote!(!static_form && name == #value)),
                Kind::Repeated(names) => {
                    for value in names {
                        conditions.push(quote!(!static_form && name == #value));
                    }
                }
                Kind::Flags(flags) => {
                    for (value, _) in flags {
                        conditions.push(quote!(!static_form && name == #value));
                    }
                }
                _ => {}
            }
        }
        quote!(false #(|| (#conditions))*)
    }
    pub(super) fn tracks(&self) -> Option<&Field> {
        self.fields
            .iter()
            .find(|field| matches!(field.kind, Kind::Tracks))
    }
    pub(super) fn animated(&self) -> impl Iterator<Item = &Field> {
        self.fields
            .iter()
            .filter(|field| matches!(field.kind, Kind::Animatable(_)))
    }
    pub(super) fn has_static(&self) -> bool {
        self.fields.iter().any(|field| {
            matches!(field.kind, Kind::StaticProperty(_) | Kind::Animatable(_))
                && !field.animated_only
        })
    }
}

pub(super) fn parse(input: &DeriveInput, options: &Container) -> Result<Schema> {
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    &input.ident,
                    "MDL derives support named-field structs only",
                ))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input.ident,
                "MDL derives support structs only",
            ))
        }
    };
    let mut normalized = Vec::new();
    let mut projections = Vec::new();
    for source in fields {
        let mut projected = None;
        let mut ordinary = false;
        for attr in &source.attrs {
            if !attr.path().is_ident("mdl") {
                continue;
            }
            let starts_project = attr
                .meta
                .require_list()?
                .tokens
                .clone()
                .into_iter()
                .next()
                .is_some_and(
                    |token| matches!(token, TokenTree::Ident(ident) if ident == "project"),
                );
            if !starts_project {
                ordinary = true;
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("project") {
                    if projected.is_some() {
                        return Err(meta.error("duplicate project"));
                    }
                    let content;
                    parenthesized!(content in meta.input);
                    projected = Some(Punctuated::<SynField, Token![,]>::parse_terminated_with(
                        &content,
                        SynField::parse_named,
                    )?);
                } else {
                    ordinary = true;
                    // Other metadata is parsed by the normal field parser below.
                    let _: TokenStream = meta.input.parse()?;
                }
                Ok(())
            })?;
        }
        if let Some(projected) = projected {
            if ordinary {
                return Err(Error::new_spanned(
                    source,
                    "project cannot have other MDL field attributes",
                ));
            }
            if projected.is_empty() {
                return Err(Error::new_spanned(
                    source,
                    "project needs at least one field",
                ));
            }
            if !matches!(source.ty, Type::Path(_)) {
                return Err(Error::new_spanned(
                    &source.ty,
                    "project requires a named struct type",
                ));
            }
            let parent = source.ident.clone().expect("named field");
            let mut names = Vec::new();
            for source in projected {
                let mut projected = field(&source, normalized.len())?;
                if names.contains(&projected.member) {
                    return Err(Error::new_spanned(source, "duplicate projected member"));
                }
                if matches!(projected.kind, Kind::Tracks) {
                    return Err(Error::new_spanned(
                        source,
                        "projected tracks are not supported; link to the parent's tracks",
                    ));
                }
                names.push(projected.member.clone());
                projected.parent = Some(parent.clone());
                normalized.push(projected);
            }
            projections.push(Projection {
                member: parent,
                ty: source.ty.clone(),
            });
        } else {
            normalized.push(field(source, normalized.len())?);
        }
    }
    let fields = normalized;
    for field in &fields {
        let has_default = field.default.is_some() || (options.default && !field.required);
        if matches!(field.kind, Kind::Animatable(_)) && !has_default {
            return Err(Error::new_spanned(
                &field.member,
                "animatable fields require default and track attributes or a container default",
            ));
        }
        if matches!(field.kind, Kind::Skip) && !has_default {
            return Err(Error::new_spanned(
                &field.member,
                "skipped fields need an explicit default or container default",
            ));
        }
        if field.skip_if.is_some() && !has_default {
            return Err(Error::new_spanned(
                &field.member,
                "skip_if requires an explicit default or container default",
            ));
        }
        if field.required && !options.default {
            return Err(Error::new_spanned(
                &field.member,
                "required is only needed with a container default",
            ));
        }
    }
    let mut names = Vec::new();
    for field in &fields {
        let mut field_names = match &field.kind {
            Kind::Property(name)
            | Kind::Block(name)
            | Kind::Counted(name)
            | Kind::DelegatedProperty(name)
            | Kind::StaticProperty(name)
            | Kind::Animatable(name)
            | Kind::Flag(name) => vec![name],
            Kind::Repeated(names) => names.iter().collect(),
            Kind::Flags(flags) => flags.iter().map(|(name, _)| name).collect(),
            _ => Vec::new(),
        };
        if let Some(extra) = &field.extra_flags {
            field_names.extend(extra.flags.iter().map(|(name, _)| name));
        }
        field_names.extend(field.channels.iter().map(|(name, _)| name));
        for name in field_names {
            if names.contains(&name.value()) {
                return Err(Error::new_spanned(name, "duplicate MDL field name"));
            }
            names.push(name.value());
        }
    }
    let tracks = fields
        .iter()
        .filter(|field| matches!(field.kind, Kind::Tracks))
        .collect::<Vec<_>>();
    let animated = fields
        .iter()
        .filter(|field| matches!(field.kind, Kind::Animatable(_)))
        .collect::<Vec<_>>();
    let has_channels = tracks.iter().any(|field| !field.channels.is_empty());
    if tracks.len() > 1
        || (!animated.is_empty() && tracks.len() != 1)
        || (animated.is_empty() && !has_channels && !tracks.is_empty())
    {
        return Err(Error::new_spanned(&input.ident, "animatable fields require exactly one tracks collection, and tracks requires animatable fields"));
    }
    let mut variants = Vec::new();
    for field in &animated {
        let variant = field.track.as_ref().expect("track was checked");
        let variant_key = quote!(#variant).to_string();
        if variants.contains(&variant_key) {
            return Err(Error::new_spanned(variant, "duplicate track variant"));
        }
        variants.push(variant_key);
    }
    for field in &tracks {
        for (_, variant) in &field.channels {
            let key = quote!(#variant).to_string();
            if variants.contains(&key) {
                return Err(Error::new_spanned(variant, "duplicate track variant"));
            }
            variants.push(key);
        }
    }
    let has_static = fields.iter().any(|field| {
        matches!(field.kind, Kind::StaticProperty(_) | Kind::Animatable(_)) && !field.animated_only
    });
    if has_static && names.iter().any(|name| name == "static") {
        return Err(Error::new_spanned(
            &input.ident,
            "static is reserved when static properties are present",
        ));
    }

    let mut ordered = (0..fields.len()).collect::<Vec<_>>();
    if let Some(order) = &options.write_order {
        let body = fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                (!matches!(field.kind, Kind::Header | Kind::Skip)).then_some(index)
            })
            .collect::<Vec<_>>();
        let mut body_order = Vec::new();
        for name in order {
            if projections
                .iter()
                .any(|projection| projection.member == *name)
            {
                body_order.extend(fields.iter().enumerate().filter_map(|(index, field)| {
                    (field.parent.as_ref() == Some(name)
                        && !matches!(field.kind, Kind::Header | Kind::Skip))
                    .then_some(index)
                }));
            } else if let Some(index) = fields.iter().position(|field| {
                field.parent.is_none()
                    && field.member == *name
                    && !matches!(field.kind, Kind::Header | Kind::Skip)
            }) {
                body_order.push(index);
            } else {
                return Err(Error::new_spanned(name, "write_order must list every body field exactly once, excluding headers and skipped fields"));
            }
        }
        if body_order.len() != body.len() || body.iter().any(|index| !body_order.contains(index)) {
            return Err(Error::new_spanned(&input.ident, "write_order must list every body field exactly once, excluding headers and skipped fields"));
        }
        ordered = fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                matches!(field.kind, Kind::Header | Kind::Skip).then_some(index)
            })
            .collect();
        ordered.extend(body_order);
    }

    if names.len() > 64 {
        return Err(Error::new_spanned(
            &input.ident,
            "MDL derives support at most 64 body fields",
        ));
    }
    Ok(Schema {
        fields,
        ordered,
        projections,
    })
}

pub(super) fn constructor(ty: &Type) -> TokenStream {
    let Type::Path(ty) = ty else {
        unreachable!("validated projection type")
    };
    let mut path = ty.path.clone();
    for segment in &mut path.segments {
        if let PathArguments::AngleBracketed(arguments) = &mut segment.arguments {
            arguments.colon2_token = Some(Default::default());
        }
    }
    quote!(#path)
}
