//! Validation across fields and the normalized layout used by code generation.
use super::attributes::{field, Container, Field, Kind};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result};

pub(super) struct Schema {
    pub(super) fields: Vec<Field>,
    pub(super) ordered: Vec<usize>,
}

impl Schema {
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
        self.fields
            .iter()
            .any(|field| matches!(field.kind, Kind::StaticProperty(_) | Kind::Animatable(_)))
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
    let fields = fields
        .iter()
        .enumerate()
        .map(|(i, value)| field(value, i))
        .collect::<Result<Vec<_>>>()?;
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
        let field_names = match &field.kind {
            Kind::Property(name)
            | Kind::StaticProperty(name)
            | Kind::Animatable(name)
            | Kind::Flag(name) => vec![name],
            Kind::Flags(flags) => flags.iter().map(|(name, _)| name).collect(),
            _ => Vec::new(),
        };
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
    if tracks.len() > 1
        || (!animated.is_empty() && tracks.len() != 1)
        || (animated.is_empty() && !tracks.is_empty())
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
    let has_static = fields
        .iter()
        .any(|field| matches!(field.kind, Kind::StaticProperty(_) | Kind::Animatable(_)));
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
            .filter(|field| !matches!(field.kind, Kind::Header | Kind::Skip))
            .collect::<Vec<_>>();
        if order.len() != body.len()
            || order
                .iter()
                .any(|name| !body.iter().any(|field| field.member == *name))
        {
            return Err(Error::new_spanned(&input.ident, "write_order must list every body field exactly once, excluding headers and skipped fields"));
        }
        ordered = fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                matches!(field.kind, Kind::Header | Kind::Skip).then_some(index)
            })
            .collect();
        ordered.extend(order.iter().map(|name| {
            fields
                .iter()
                .position(|field| field.member == *name)
                .expect("order was checked")
        }));
    }
    if names.len() > 64 {
        return Err(Error::new_spanned(
            &input.ident,
            "MDL derives support at most 64 body fields",
        ));
    }
    Ok(Schema { fields, ordered })
}
