//! Attribute parsing and validation within a field or container.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    meta::ParseNestedMeta, parenthesized, punctuated::Punctuated, DeriveInput, Error,
    Field as SynField, GenericArgument, Ident, LitInt, LitStr, Path, PathArguments, Result, Token,
    Type,
};

#[derive(Default)]
pub(super) struct Container {
    pub(super) virtual_fields: Option<Vec<SynField>>,
    pub(super) block: Option<LitStr>,
    pub(super) property: Option<LitStr>,
    pub(super) entry: bool,
    pub(super) fields: bool,
    pub(super) default: bool,
    pub(super) write_order: Option<Vec<Ident>>,
    pub(super) validate_read: Option<Path>,
    pub(super) after_read: Option<Path>,
    pub(super) validate_write: Option<Path>,
}

pub(super) enum Kind {
    Header,
    Flatten,
    Block(LitStr),
    Repeated(Vec<LitStr>),
    Counted(LitStr),
    Property(LitStr),
    DelegatedProperty(LitStr),
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

pub(super) struct ExtraFlags {
    pub(super) get: Path,
    pub(super) set: Path,
    pub(super) flags: Vec<(LitStr, u32)>,
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
    pub(super) unique_by: Option<Path>,
    pub(super) animated_only: bool,
    pub(super) bare_static: bool,
    pub(super) parent: Option<Ident>,
    pub(super) channels: Vec<(LitStr, Path)>,
    pub(super) extra_flags: Option<ExtraFlags>,
    pub(super) get: Option<Path>,
    pub(super) set: Option<Path>,
    pub(super) slot: Option<Path>,
    pub(super) virtual_field: bool,
}

impl Field {
    pub(super) fn value(&self, receiver: TokenStream) -> TokenStream {
        match &self.get {
            Some(get) if self.slot.is_some() => {
                let default = match self
                    .default
                    .as_ref()
                    .expect("optional accessor default was checked")
                {
                    DefaultValue::Trait => quote!(::core::default::Default::default),
                    DefaultValue::Function(function) => quote!(#function),
                };
                quote!(#get(#receiver).unwrap_or_else(#default))
            }
            Some(get) => quote!(#get(#receiver)),
            None => {
                let access = self.access();
                quote!((#receiver).#access)
            }
        }
    }
    pub(super) fn access(&self) -> TokenStream {
        let member = &self.member;
        match &self.parent {
            Some(parent) => quote!(#parent.#member),
            None => quote!(#member),
        }
    }
}

pub(super) fn identifier(name: &LitStr) -> Result<()> {
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
            if meta.path.is_ident("virtual_fields") {
                if result.virtual_fields.is_some() { return Err(meta.error("duplicate virtual_fields")); }
                let content;
                parenthesized!(content in meta.input);
                let fields = Punctuated::<SynField, Token![,]>::parse_terminated_with(&content, SynField::parse_named)?;
                if fields.is_empty() { return Err(meta.error("virtual_fields needs at least one field")); }
                result.virtual_fields = Some(fields.into_iter().collect());
                Ok(())
            } else if meta.path.is_ident("block") {
                if result.block.is_some() {
                    return Err(meta.error("duplicate block"));
                }
                if result.property.is_some() || result.entry || result.fields { return Err(meta.error("choose exactly one of block, property, entry, or fields")); }
                let name = meta.value()?.parse()?;
                identifier(&name)?;
                result.block = Some(name);
                Ok(())
            } else if meta.path.is_ident("property") {
                if result.block.is_some() || result.property.is_some() || result.entry || result.fields { return Err(meta.error("choose exactly one of block, property, entry, or fields")); }
                let name = meta.value()?.parse()?;
                identifier(&name)?;
                result.property = Some(name);
                Ok(())
            } else if meta.path.is_ident("entry") {
                if result.block.is_some() || result.property.is_some() || result.entry || result.fields { return Err(meta.error("choose exactly one of block, property, entry, or fields")); }
                result.entry = true;
                Ok(())
            } else if meta.path.is_ident("fields") {
                if result.block.is_some() || result.property.is_some() || result.entry || result.fields {
                    return Err(meta.error("choose exactly one of block, property, entry, or fields"));
                }
                result.fields = true;
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
            } else if meta.path.is_ident("after_read") {
                path(&meta, &mut result.after_read)
            } else if meta.path.is_ident("validate_read") {
                path(&meta, &mut result.validate_read)
            } else if meta.path.is_ident("validate_write") {
                path(&meta, &mut result.validate_write)
            } else {
                Err(meta.error("expected block, property, entry, fields, default, write_order, virtual_fields, after_read, validate_read, or validate_write"))
            }
        })?;
    }
    if result.block.is_none() && result.property.is_none() && !result.entry && !result.fields {
        return Err(Error::new_spanned(
            &input.ident,
            "MDL derives require a block, property, entry, or fields attribute",
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
    let mut delegate = false;
    let mut unique_by = None;
    let mut animated_only = false;
    let mut bare_static = false;
    let mut channels = Vec::new();
    let mut channels_seen = false;
    let mut extra_flags = None;
    let mut get = None;
    let mut set = None;
    let mut slot = None;
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
                || meta.path.is_ident("flatten")
                || meta.path.is_ident("block")
                || meta.path.is_ident("repeated")
                || meta.path.is_ident("counted")
            {
                if kind.is_some() {
                    return Err(meta.error(
                        "a field must have exactly one of header, property, block, flatten, repeated, counted, static_property, animatable, tracks, flag, flags, or skip",
                    ));
                }
                kind = Some(if meta.path.is_ident("header") {
                    Kind::Header
                } else if meta.path.is_ident("flatten") {
                    Kind::Flatten
                } else if meta.path.is_ident("skip") {
                    Kind::Skip
                } else if meta.path.is_ident("tracks") {
                    Kind::Tracks
                } else if meta.path.is_ident("repeated") {
                    let mut names = Vec::new();
                    if meta.input.peek(Token![=]) {
                        let name = meta.value()?.parse()?;
                        identifier(&name)?;
                        names.push(name);
                    } else {
                        meta.parse_nested_meta(|item| {
                            let ident = item.path.get_ident().ok_or_else(|| item.error("expected an MDL name"))?;
                            let name = LitStr::new(&ident.to_string(), ident.span());
                            identifier(&name)?;
                            names.push(name);
                            Ok(())
                        })?;
                    }
                    if names.is_empty() { return Err(meta.error("repeated needs at least one name")); }
                    Kind::Repeated(names)
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
                    if meta.path.is_ident("block") {
                        Kind::Block(name)
                    } else if meta.path.is_ident("counted") {
                        Kind::Counted(name)
                    } else if meta.path.is_ident("property") {
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
            } else if meta.path.is_ident("get") {
                path(&meta, &mut get)
            } else if meta.path.is_ident("set") {
                path(&meta, &mut set)
            } else if meta.path.is_ident("slot") {
                path(&meta, &mut slot)
            } else if meta.path.is_ident("delegate") {
                if delegate { return Err(meta.error("duplicate delegate")); }
                delegate = true;
                Ok(())
            } else if meta.path.is_ident("channels") {
                if channels_seen { return Err(meta.error("duplicate channels")); }
                channels_seen = true;
                meta.parse_nested_meta(|item| {
                    let ident = item.path.get_ident().ok_or_else(|| item.error("expected a channel name"))?;
                    let name = LitStr::new(&ident.to_string(), ident.span());
                    identifier(&name)?;
                    let value: LitStr = item.value()?.parse()?;
                    channels.push((name, value.parse()?));
                    Ok(())
                })?;
                if channels.is_empty() { return Err(meta.error("channels needs at least one channel")); }
                Ok(())
            } else if meta.path.is_ident("extra_flags") {
                if extra_flags.is_some() { return Err(meta.error("duplicate extra_flags")); }
                let mut get = None;
                let mut set = None;
                let mut flags = Vec::new();
                let mut bits = 0u32;
                meta.parse_nested_meta(|item| {
                    if item.path.is_ident("get") { return path(&item, &mut get); }
                    if item.path.is_ident("set") { return path(&item, &mut set); }
                    let ident = item.path.get_ident().ok_or_else(|| item.error("expected a flag name"))?;
                    let name = LitStr::new(&ident.to_string(), ident.span());
                    identifier(&name)?;
                    let mask: LitInt = item.value()?.parse()?;
                    let mask = mask.base10_parse::<u32>()?;
                    if !mask.is_power_of_two() || bits & mask != 0 { return Err(item.error("flag masks must be distinct nonzero single u32 bits")); }
                    bits |= mask;
                    flags.push((name, mask));
                    Ok(())
                })?;
                if flags.is_empty() { return Err(meta.error("extra_flags needs at least one flag")); }
                extra_flags = Some(ExtraFlags {
                    get: get.ok_or_else(|| meta.error("extra_flags requires get and set"))?,
                    set: set.ok_or_else(|| meta.error("extra_flags requires get and set"))?, flags,
                });
                Ok(())
            } else if meta.path.is_ident("bare_static") {
                if bare_static { return Err(meta.error("duplicate bare_static")); }
                bare_static = true;
                Ok(())
            } else if meta.path.is_ident("animated_only") {
                if animated_only { return Err(meta.error("duplicate animated_only")); }
                animated_only = true;
                Ok(())
            } else if meta.path.is_ident("unique_by") {
                path(&meta, &mut unique_by)
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
            "each field needs an explicit MDL header, property, block, flatten, repeated, counted, static_property, animatable, tracks, flag, flags, or skip attribute",
        )
    })?;
    let kind = if delegate {
        match kind {
            Kind::Property(name) => Kind::DelegatedProperty(name),
            _ => {
                return Err(Error::new_spanned(
                    field,
                    "delegate requires property = \"Name\"",
                ))
            }
        }
    } else {
        kind
    };
    if matches!(kind, Kind::DelegatedProperty(_))
        && (default.is_some()
            || skip_if.is_some()
            || read_with.is_some()
            || write_with.is_some()
            || required)
    {
        return Err(Error::new_spanned(field, "delegate owns defaults, requirements, omission, and codec framing; it cannot have default, required, skip_if, read_with, or write_with"));
    }
    if extra_flags.is_some() && !matches!(kind, Kind::Flatten) {
        return Err(Error::new_spanned(field, "extra_flags requires flatten"));
    }
    if channels_seen && !matches!(kind, Kind::Tracks) {
        return Err(Error::new_spanned(field, "channels requires tracks"));
    }
    if bare_static && (!matches!(kind, Kind::Animatable(_)) || animated_only) {
        return Err(Error::new_spanned(
            field,
            "bare_static requires an animatable field with a static form",
        ));
    }
    if animated_only && !matches!(kind, Kind::Animatable(_)) {
        return Err(Error::new_spanned(
            field,
            "animated_only requires animatable",
        ));
    }
    if unique_by.is_some() && !matches!(kind, Kind::Repeated(_)) {
        return Err(Error::new_spanned(field, "unique_by requires repeated"));
    }
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
        if enable_with.is_some() && enabled_if.is_none() {
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
    if matches!(
        kind,
        Kind::Flatten | Kind::Repeated(_) | Kind::Counted(_) | Kind::Block(_)
    ) {
        if skip_if.is_some()
            || read_with.is_some()
            || write_with.is_some()
            || (matches!(kind, Kind::Flatten | Kind::Repeated(_))
                && (default.is_some() || required))
        {
            return Err(Error::new_spanned(field, "structural fields do not support codec hooks or skip_if; flatten and repeated also own their initialization and presence policy"));
        }
        if matches!(kind, Kind::Repeated(_) | Kind::Counted(_)) {
            vec_element(&field.ty)?;
        }
    }
    if matches!(kind, Kind::Tracks) {
        if default.is_some() || skip_if.is_some() || read_with.is_some() || write_with.is_some() {
            return Err(Error::new_spanned(
                field,
                "tracks cannot have defaults, omission predicates, or codec hooks",
            ));
        }
        vec_element(&field.ty)?;
    }
    if required
        && (default.is_some()
            || skip_if.is_some()
            || !matches!(
                kind,
                Kind::Property(_) | Kind::StaticProperty(_) | Kind::Block(_) | Kind::Counted(_)
            ))
    {
        return Err(Error::new_spanned(
            field,
            "required is only supported on properties, static properties, blocks, and counted lists without defaults or omission predicates",
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
        unique_by,
        animated_only,
        bare_static,
        parent: None,
        channels,
        extra_flags,
        get,
        set,
        slot,
        virtual_field: false,
    })
}

pub(super) fn vec_element(ty: &Type) -> Result<&Type> {
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
    Err(Error::new_spanned(
        ty,
        "tracks or collection requires Vec<Item>",
    ))
}
