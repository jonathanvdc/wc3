//! Attribute parsing and validation within a field or container.
use quote::format_ident;
use syn::{
    meta::ParseNestedMeta, DeriveInput, Error, Field as SynField, GenericArgument, Ident, LitInt,
    LitStr, Path, PathArguments, Result, Token, Type,
};

#[derive(Default)]
pub(super) struct Container {
    pub(super) block: Option<LitStr>,
    pub(super) property: Option<LitStr>,
    pub(super) entry: bool,
    pub(super) default: bool,
    pub(super) write_order: Option<Vec<Ident>>,
    pub(super) validate_read: Option<Path>,
    pub(super) validate_write: Option<Path>,
}

pub(super) enum Kind {
    Header,
    Property(LitStr),
    StaticProperty(LitStr),
    Animatable(LitStr),
    Tracks,
    Flag(LitStr),
    Flags(Vec<(LitStr, u32)>),
    Skip,
}

pub(super) enum DefaultValue {
    Trait,
    Function(Path),
}

pub(super) struct Field {
    pub(super) member: Ident,
    pub(super) local: Ident,
    pub(super) ty: Type,
    pub(super) kind: Kind,
    pub(super) default: Option<DefaultValue>,
    pub(super) skip_if: Option<Path>,
    pub(super) read_with: Option<Path>,
    pub(super) write_with: Option<Path>,
    pub(super) track: Option<Path>,
    pub(super) enabled_if: Option<Path>,
    pub(super) enable_with: Option<Path>,
    pub(super) allow_bits: u32,
    pub(super) required: bool,
}

fn identifier(name: &LitStr) -> Result<()> {
    let value = name.value();
    let mut bytes = value.bytes();
    if !bytes
        .next()
        .is_some_and(|b| b.is_ascii_alphabetic() || b == b'_')
        || !bytes.all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err(Error::new_spanned(name, "expected an MDL identifier"));
    }
    if value.eq_ignore_ascii_case("nan") || value.eq_ignore_ascii_case("inf") {
        return Err(Error::new_spanned(
            name,
            "numeric literals are not MDL identifiers",
        ));
    }
    Ok(())
}

fn path(meta: &ParseNestedMeta<'_>, target: &mut Option<Path>) -> Result<()> {
    if target.is_some() {
        return Err(meta.error("duplicate attribute"));
    }
    let value: LitStr = meta.value()?.parse()?;
    *target = Some(value.parse()?);
    Ok(())
}

pub(super) fn container(input: &DeriveInput) -> Result<Container> {
    let mut result = Container::default();
    for attr in &input.attrs {
        if !attr.path().is_ident("mdl") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("block") {
                if result.block.is_some() {
                    return Err(meta.error("duplicate block"));
                }
                if result.property.is_some() || result.entry { return Err(meta.error("choose exactly one of block, property, or entry")); }
                let name = meta.value()?.parse()?;
                identifier(&name)?;
                result.block = Some(name);
                Ok(())
            } else if meta.path.is_ident("property") {
                if result.block.is_some() || result.property.is_some() || result.entry { return Err(meta.error("choose exactly one of block, property, or entry")); }
                let name = meta.value()?.parse()?;
                identifier(&name)?;
                result.property = Some(name);
                Ok(())
            } else if meta.path.is_ident("entry") {
                if result.block.is_some() || result.property.is_some() || result.entry { return Err(meta.error("choose exactly one of block, property, or entry")); }
                result.entry = true;
                Ok(())
            } else if meta.path.is_ident("default") {
                if result.default { return Err(meta.error("duplicate container default")); }
                result.default = true;
                Ok(())
            } else if meta.path.is_ident("write_order") {
                if result.write_order.is_some() { return Err(meta.error("duplicate write_order")); }
                let mut order = Vec::new();
                meta.parse_nested_meta(|field| {
                    let name = field.path.get_ident().ok_or_else(|| field.error("expected a field name"))?.clone();
                    if order.contains(&name) { return Err(field.error("duplicate field in write_order")); }
                    order.push(name);
                    Ok(())
                })?;
                result.write_order = Some(order);
                Ok(())
            } else if meta.path.is_ident("validate_read") {
                path(&meta, &mut result.validate_read)
            } else if meta.path.is_ident("validate_write") {
                path(&meta, &mut result.validate_write)
            } else {
                Err(meta.error("expected block, property, entry, default, write_order, validate_read, or validate_write"))
            }
        })?;
    }
    if result.block.is_none() && result.property.is_none() && !result.entry {
        return Err(Error::new_spanned(
            &input.ident,
            "MDL derives require a block, property, or entry attribute",
        ));
    }
    Ok(result)
}

pub(super) fn field(field: &SynField, index: usize) -> Result<Field> {
    let mut kind = None;
    let mut default = None;
    let mut skip_if = None;
    let mut read_with = None;
    let mut write_with = None;
    let mut track = None;
    let mut enabled_if = None;
    let mut enable_with = None;
    let mut allow_bits = None;
    let mut required = false;
    for attr in &field.attrs {
        if !attr.path().is_ident("mdl") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("header")
                || meta.path.is_ident("property")
                || meta.path.is_ident("flag")
                || meta.path.is_ident("flags")
                || meta.path.is_ident("skip")
                || meta.path.is_ident("static_property")
                || meta.path.is_ident("animatable")
                || meta.path.is_ident("tracks")
            {
                if kind.is_some() {
                    return Err(meta.error(
                        "a field must have exactly one of header, property, static_property, animatable, tracks, flag, flags, or skip",
                    ));
                }
                kind = Some(if meta.path.is_ident("header") {
                    Kind::Header
                } else if meta.path.is_ident("skip") {
                    Kind::Skip
                } else if meta.path.is_ident("tracks") {
                    Kind::Tracks
                } else if meta.path.is_ident("flags") {
                    let mut flags = Vec::new();
                    let mut bits = 0u32;
                    meta.parse_nested_meta(|flag| {
                        let ident = flag
                            .path
                            .get_ident()
                            .ok_or_else(|| flag.error("expected a flag name"))?;
                        let name = LitStr::new(&ident.to_string(), ident.span());
                        identifier(&name)?;
                        let mask: LitInt = flag.value()?.parse()?;
                        let mask = mask.base10_parse::<u32>()?;
                        if !mask.is_power_of_two() {
                            return Err(flag.error("flag masks must be nonzero single u32 bits"));
                        }
                        if bits & mask != 0 {
                            return Err(flag.error("duplicate flag mask"));
                        }
                        bits |= mask;
                        flags.push((name, mask));
                        Ok(())
                    })?;
                    if flags.is_empty() {
                        return Err(meta.error("flags requires at least one mapping"));
                    }
                    Kind::Flags(flags)
                } else {
                    let name = meta.value()?.parse()?;
                    identifier(&name)?;
                    if meta.path.is_ident("property") {
                        Kind::Property(name)
                    } else if meta.path.is_ident("static_property") {
                        Kind::StaticProperty(name)
                    } else if meta.path.is_ident("animatable") {
                        Kind::Animatable(name)
                    } else {
                        Kind::Flag(name)
                    }
                });
                Ok(())
            } else if meta.path.is_ident("required") {
                if required { return Err(meta.error("duplicate required")); }
                required = true;
                Ok(())
            } else if meta.path.is_ident("track") {
                path(&meta, &mut track)
            } else if meta.path.is_ident("enabled_if") {
                path(&meta, &mut enabled_if)
            } else if meta.path.is_ident("enable_with") {
                path(&meta, &mut enable_with)
            } else if meta.path.is_ident("allow_bits") {
                if allow_bits.is_some() {
                    return Err(meta.error("duplicate allow_bits"));
                }
                let value: LitInt = meta.value()?.parse()?;
                allow_bits = Some(value.base10_parse::<u32>()?);
                Ok(())
            } else if meta.path.is_ident("default") {
                if default.is_some() {
                    return Err(meta.error("duplicate default"));
                }
                default = Some(if meta.input.peek(Token![=]) {
                    let value: LitStr = meta.value()?.parse()?;
                    DefaultValue::Function(value.parse()?)
                } else {
                    DefaultValue::Trait
                });
                Ok(())
            } else if meta.path.is_ident("skip_if") {
                path(&meta, &mut skip_if)
            } else if meta.path.is_ident("read_with") {
                path(&meta, &mut read_with)
            } else if meta.path.is_ident("write_with") {
                path(&meta, &mut write_with)
            } else {
                Err(meta.error("unknown MDL field attribute"))
            }
        })?;
    }
    let kind = kind.ok_or_else(|| {
        Error::new_spanned(
            field,
            "each field needs an explicit MDL header, property, static_property, animatable, tracks, flag, flags, or skip attribute",
        )
    })?;
    if matches!(kind, Kind::Animatable(_)) {
        if track.is_none() {
            return Err(Error::new_spanned(
                field,
                "animatable fields require a track attribute",
            ));
        }
        if read_with.is_some() || write_with.is_some() {
            return Err(Error::new_spanned(
                field,
                "animatable fields do not support value codec hooks",
            ));
        }
        if enabled_if.is_some() != enable_with.is_some() {
            return Err(Error::new_spanned(
                field,
                "enabled_if and enable_with must be supplied together",
            ));
        }
    } else if track.is_some() || enabled_if.is_some() || enable_with.is_some() {
        return Err(Error::new_spanned(
            field,
            "track and enable hooks require an animatable field",
        ));
    }
    if allow_bits.is_some() && !matches!(kind, Kind::Flags(_)) {
        return Err(Error::new_spanned(
            field,
            "allow_bits requires packed flags",
        ));
    }
    if let Kind::Flags(flags) = &kind {
        let mapped = flags.iter().fold(0, |bits, (_, mask)| bits | mask);
        if mapped & allow_bits.unwrap_or(0) != 0 {
            return Err(Error::new_spanned(
                field,
                "allow_bits must not overlap mapped flags",
            ));
        }
    }
    if matches!(kind, Kind::Tracks) {
        if default.is_some() || skip_if.is_some() || read_with.is_some() || write_with.is_some() {
            return Err(Error::new_spanned(
                field,
                "tracks cannot have defaults, omission predicates, or codec hooks",
            ));
        }
        track_element(&field.ty)?;
    }
    if required
        && (default.is_some()
            || skip_if.is_some()
            || !matches!(kind, Kind::Property(_) | Kind::StaticProperty(_)))
    {
        return Err(Error::new_spanned(
            field,
            "required is only supported on properties without defaults or omission predicates",
        ));
    }
    match &kind {
        Kind::Header if default.is_some() || skip_if.is_some() => {
            return Err(Error::new_spanned(
                field,
                "headers cannot have default or skip_if",
            ))
        }
        Kind::Skip if skip_if.is_some() || read_with.is_some() || write_with.is_some() => {
            return Err(Error::new_spanned(
                field,
                "skipped fields cannot have codec hooks or skip_if",
            ));
        }
        Kind::Flags(_)
            if read_with.is_some()
                || write_with.is_some()
                || skip_if.is_some()
                || matches!(default, Some(DefaultValue::Function(_))) =>
        {
            return Err(Error::new_spanned(
                field,
                "packed flags only support a zero default and no codec hooks or skip_if",
            ));
        }
        Kind::Flag(_) => {
            if !matches!(&field.ty, Type::Path(ty) if ty.path.is_ident("bool")) {
                return Err(Error::new_spanned(
                    &field.ty,
                    "MDL flags must have type bool",
                ));
            }
            if read_with.is_some() || write_with.is_some() || skip_if.is_some() {
                return Err(Error::new_spanned(
                    field,
                    "flags cannot have codec hooks or skip_if",
                ));
            }
            if matches!(default, Some(DefaultValue::Function(_))) {
                return Err(Error::new_spanned(
                    field,
                    "flags only support the false default (#[mdl(default)])",
                ));
            }
        }
        _ => {}
    }
    Ok(Field {
        member: field.ident.clone().expect("named fields were checked"),
        local: format_ident!("__wc3_mdl_field_{index}"),
        ty: field.ty.clone(),
        kind,
        default,
        skip_if,
        read_with,
        write_with,
        track,
        enabled_if,
        enable_with,
        allow_bits: allow_bits.unwrap_or(0),
        required,
    })
}

pub(super) fn track_element(ty: &Type) -> Result<&Type> {
    if let Type::Path(path) = ty {
        if let Some(segment) = path.path.segments.last() {
            if segment.ident == "Vec" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if args.args.len() == 1 {
                        if let Some(GenericArgument::Type(element)) = args.args.first() {
                            return Ok(element);
                        }
                    }
                }
            }
        }
    }
    Err(Error::new_spanned(ty, "tracks requires Vec<TrackEnum>"))
}
