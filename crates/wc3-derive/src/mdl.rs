//! MDL attribute validation and generation of direct struct codecs.
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    meta::ParseNestedMeta, parse_quote, Data, DeriveInput, Error, Field as SynField, Fields,
    GenericArgument, GenericParam, Ident, LitInt, LitStr, Path, PathArguments, Result, Token, Type,
};

#[derive(Default)]
struct Container {
    block: Option<LitStr>,
    property: Option<LitStr>,
    entry: bool,
    default: bool,
    write_order: Option<Vec<Ident>>,
    validate_read: Option<Path>,
    validate_write: Option<Path>,
}

enum Kind {
    Header,
    Property(LitStr),
    StaticProperty(LitStr),
    Animatable(LitStr),
    Tracks,
    Flag(LitStr),
    Flags(Vec<(LitStr, u32)>),
    Skip,
}

enum DefaultValue {
    Trait,
    Function(Path),
}

struct Field {
    member: Ident,
    local: Ident,
    ty: Type,
    kind: Kind,
    default: Option<DefaultValue>,
    skip_if: Option<Path>,
    read_with: Option<Path>,
    write_with: Option<Path>,
    track: Option<Path>,
    enabled_if: Option<Path>,
    enable_with: Option<Path>,
    allow_bits: u32,
    required: bool,
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

fn container(input: &DeriveInput) -> Result<Container> {
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

fn field(field: &SynField, index: usize) -> Result<Field> {
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
        Kind::Skip => {
            if skip_if.is_some() || read_with.is_some() || write_with.is_some() {
                return Err(Error::new_spanned(
                    field,
                    "skipped fields cannot have codec hooks or skip_if",
                ));
            }
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

fn track_element(ty: &Type) -> Result<&Type> {
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

pub(crate) fn expand(input: DeriveInput, reading: bool) -> TokenStream {
    match expand_checked(input, reading) {
        Ok(tokens) => tokens,
        Err(error) => error.to_compile_error(),
    }
}

fn expand_checked(input: DeriveInput, reading: bool) -> Result<TokenStream> {
    let options = container(&input)?;
    if options.block.is_none() {
        return expand_value(input, options, reading);
    }
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
    let tracks = tracks.first().copied();
    let mut ordered = fields.iter().collect::<Vec<_>>();
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
            .filter(|field| matches!(field.kind, Kind::Header | Kind::Skip))
            .collect();
        ordered.extend(order.iter().map(|name| {
            body.iter()
                .find(|field| field.member == *name)
                .copied()
                .expect("order was checked")
        }));
    }
    if names.len() > 64 {
        return Err(Error::new_spanned(
            &input.ident,
            "MDL derives support at most 64 body fields",
        ));
    }
    let block = options.block.expect("block was checked");
    let name = &input.ident;
    let mut generics = input.generics.clone();
    for field in &fields {
        let ty = &field.ty;
        if matches!(
            field.kind,
            Kind::Header | Kind::Property(_) | Kind::StaticProperty(_) | Kind::Animatable(_)
        ) {
            if reading && field.read_with.is_none() {
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#ty: ::wc3::model::mdl::Read));
            } else if !reading && field.write_with.is_none() {
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#ty: ::wc3::model::mdl::Write));
            }
        }
        if !reading && matches!(field.kind, Kind::Tracks) {
            let element = track_element(ty)?;
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#element: ::wc3::model::mdl::Write));
        }
        if matches!(field.kind, Kind::Flags(_)) {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::wc3::model::mdl::BitRange<u32>));
            if reading {
                generics
                    .make_where_clause()
                    .predicates
                    .push(parse_quote!(#ty: ::wc3::model::mdl::BitRangeMut<u32>));
                if !options.default || field.default.is_some() {
                    generics
                        .make_where_clause()
                        .predicates
                        .push(parse_quote!(#ty: ::core::default::Default));
                }
            }
        }
        if reading
            && !matches!(field.kind, Kind::Flags(_))
            && matches!(field.default, Some(DefaultValue::Trait))
        {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::core::default::Default));
        }
    }
    let write_defaults = options.default
        && fields.iter().any(|field| {
            field.default.is_none() && matches!(field.kind, Kind::Flag(_) | Kind::Flags(_))
        });
    if (reading && options.default) || (!reading && write_defaults) {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(Self: ::core::default::Default));
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    if reading {
        // Runtime paths are qualified in generated code to avoid shadowing the
        // caller's field types, generic parameters, and imports.
        let mut headers = Vec::new();
        let mut locals = Vec::new();
        let mut arms = Vec::new();
        let mut static_arms = Vec::new();
        let mut enable_calls = Vec::new();
        let mut members = Vec::new();
        let mut bit = 0u32;
        for field in &fields {
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
                None if options.default && !field.required => {
                    Some(quote!(__wc3_mdl_defaults.#member))
                }
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
                locals.push(quote!(let mut #local: ::core::option::Option<#ty> = ::core::option::Option::None;));
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
        let validate = options.validate_read.map(|function| quote! {
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
    } else {
        let mut headers = Vec::new();
        let mut writes = Vec::new();
        let mut required_flags = Vec::new();
        for field in ordered {
            let Field {
                member,
                kind,
                write_with,
                skip_if,
                ..
            } = field;
            let value = match write_with {
                Some(function) => quote!(#function(&self.#member, __wc3_mdl_writer)?;),
                None => quote!(__wc3_mdl_writer.write(&self.#member)?;),
            };
            match kind {
                Kind::Header => headers.push(quote! { __wc3_mdl_writer.raw(" ")?; #value }),
                Kind::Skip => {}
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
                    let mut condition = quote!(true);
                    if let Some(function) = skip_if {
                        condition = quote!(#condition && !#function(&self.#member));
                    }
                    if let Kind::Animatable(_) = kind {
                        let variant = field.track.as_ref().expect("track was checked");
                        let collection = &tracks.expect("tracks was checked").member;
                        condition = quote!(#condition && !self.#collection.iter().any(|track| matches!(track, #variant(_))));
                        if let Some(function) = &field.enabled_if {
                            condition = quote!(#condition && #function(self));
                        }
                    }
                    writes.push(quote!(if #condition { #write }));
                }
                Kind::Tracks => {
                    writes.push(
                        quote!(for track in &self.#member { __wc3_mdl_writer.write(track)?; }),
                    );
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
            .map(|function| quote!(#function(self)?;));
        let initialize_defaults = write_defaults.then(
            || quote!(let __wc3_mdl_write_defaults: Self = ::core::default::Default::default();),
        );
        let sink = sink_name(&input);
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
}

fn sink_name(input: &DeriveInput) -> Ident {
    let mut sink = format_ident!("__Wc3MdlSink");
    let mut suffix = 0;
    while input.generics.params.iter().any(|param| match param {
        GenericParam::Type(param) => param.ident == sink,
        GenericParam::Const(param) => param.ident == sink,
        _ => false,
    }) {
        suffix += 1;
        sink = format_ident!("__Wc3MdlSink{suffix}");
    }
    sink
}

fn expand_value(input: DeriveInput, options: Container, reading: bool) -> Result<TokenStream> {
    if options.default {
        return Err(Error::new_spanned(
            &input.ident,
            "container default is only supported on blocks",
        ));
    }
    if options.write_order.is_some() {
        return Err(Error::new_spanned(
            &input.ident,
            "write_order is only supported on blocks",
        ));
    }
    let value = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => &fields.unnamed[0],
            _ => {
                return Err(Error::new_spanned(
                    &input.ident,
                    "MDL property and entry derives require a single-field tuple struct",
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
    if let Some(attr) = value.attrs.iter().find(|attr| attr.path().is_ident("mdl")) {
        return Err(Error::new_spanned(
            attr,
            "value wrappers do not support MDL field attributes",
        ));
    }
    let ty = &value.ty;
    let name = &input.ident;
    let mut generics = input.generics.clone();
    if reading {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::wc3::model::mdl::Read));
    } else {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#ty: ::wc3::model::mdl::Write));
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    if reading {
        let start = options.validate_read.as_ref().map(|_| {
            quote! {
                __wc3_mdl_parser.peek()?;
                let __wc3_mdl_start = __wc3_mdl_parser.position();
            }
        });
        let prefix = options
            .property
            .map(|property| quote!(__wc3_mdl_parser.expect_ident(#property)?;));
        let validate = options.validate_read.map(|function| quote!(#function(&__wc3_mdl_value, ::wc3::model::mdl::Span::new(__wc3_mdl_start, __wc3_mdl_parser.position()))?;));
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdl::Read for #name #ty_generics #where_clause {
                fn read_mdl(__wc3_mdl_parser: &mut ::wc3::model::mdl::Parser<'_>) -> ::core::result::Result<Self, ::wc3::model::mdl::ReadError> {
                    #start
                    #prefix
                    let __wc3_mdl_value = Self(__wc3_mdl_parser.read_property::<#ty>()?);
                    #validate
                    ::core::result::Result::Ok(__wc3_mdl_value)
                }
            }
        })
    } else {
        let sink = sink_name(&input);
        let validate = options
            .validate_write
            .map(|function| quote!(#function(self)?;));
        let write = match options.property {
            Some(property) => quote!(__wc3_mdl_writer.property(#property, &self.0)),
            None => quote!(__wc3_mdl_writer.entry(&self.0)),
        };
        Ok(quote! {
            impl #impl_generics ::wc3::model::mdl::Write for #name #ty_generics #where_clause {
                fn write_mdl<#sink: ::std::io::Write>(&self, __wc3_mdl_writer: &mut ::wc3::model::mdl::MdlWriter<#sink>) -> ::core::result::Result<(), ::wc3::model::mdl::WriteError> {
                    #validate
                    #write
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::expand_checked;
    use quote::{format_ident, quote};
    use syn::{parse_quote, DeriveInput};

    fn rejects(input: DeriveInput, message: &str) {
        for reading in [true, false] {
            let error = expand_checked(input.clone(), reading).unwrap_err();
            assert!(error.to_string().contains(message), "{error}");
        }
    }

    #[test]
    fn rejects_missing_malformed_and_duplicate_attributes() {
        rejects(
            parse_quote!(
                struct Missing {}
            ),
            "require",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "Bad Name")]
                struct Bad {}
            ),
            "MDL identifier",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "nan")]
                struct Bad {}
            ),
            "numeric literals",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A", block = "B")]
                struct Bad {}
            ),
            "duplicate block",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A", nope)]
                struct Bad {}
            ),
            "expected block",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    value: u32,
                }
            ),
            "explicit MDL",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(property = "Value", default, default)]
                    value: u32,
                }
            ),
            "duplicate default",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(property = "Value", read_with = "a", read_with = "b")]
                    value: u32,
                }
            ),
            "duplicate attribute",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(property = "Value", nope)]
                    value: u32,
                }
            ),
            "unknown MDL field",
        );
    }

    #[test]
    fn rejects_conflicting_field_forms_and_unsafe_omission() {
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(header, property = "Value")]
                    value: u32,
                }
            ),
            "exactly one",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(header, default)]
                    value: u32,
                }
            ),
            "headers cannot",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(skip)]
                    value: u32,
                }
            ),
            "explicit default",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(skip, default, write_with = "write")]
                    value: u32,
                }
            ),
            "skipped fields cannot",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(property = "Value", skip_if = "empty")]
                    value: u32,
                }
            ),
            "skip_if requires",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flag = "Enabled")]
                    value: u32,
                }
            ),
            "type bool",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flag = "Enabled", default = "yes")]
                    value: bool,
                }
            ),
            "false default",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flag = "Enabled", read_with = "read")]
                    value: bool,
                }
            ),
            "flags cannot",
        );
    }

    #[test]
    fn rejects_duplicate_names_across_properties_and_flags() {
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(property = "Value")]
                    a: u32,
                    #[mdl(flag = "Value", default)]
                    b: bool,
                }
            ),
            "duplicate MDL field name",
        );
    }

    #[test]
    fn rejects_invalid_packed_flags_and_output_order() {
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flags())]
                    flags: u32,
                }
            ),
            "expected nested attribute",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flags(A = 0))]
                    flags: u32,
                }
            ),
            "single u32 bits",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flags(A = 3))]
                    flags: u32,
                }
            ),
            "single u32 bits",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flags(A = 1, B = 1))]
                    flags: u32,
                }
            ),
            "duplicate flag mask",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flags(A = 1, A = 2))]
                    flags: u32,
                }
            ),
            "duplicate MDL field name",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flags(A = 1))]
                    flags: u32,
                    #[mdl(property = "A")]
                    value: u32,
                }
            ),
            "duplicate MDL field name",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flags(A = 1), default = "factory")]
                    flags: u32,
                }
            ),
            "zero default",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A", write_order(missing))]
                struct Bad {
                    #[mdl(property = "Value")]
                    value: u32,
                }
            ),
            "every body field",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A", write_order(value, value))]
                struct Bad {
                    #[mdl(property = "Value")]
                    value: u32,
                }
            ),
            "duplicate field in write_order",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A", write_order(name, value))]
                struct Bad {
                    #[mdl(header)]
                    name: u32,
                    #[mdl(property = "Value")]
                    value: u32,
                }
            ),
            "excluding headers",
        );
    }

    #[test]
    fn rejects_conflicting_and_malformed_tuple_forms() {
        rejects(
            parse_quote!(
                #[mdl(block = "A", entry)]
                struct Bad {}
            ),
            "exactly one",
        );
        rejects(
            parse_quote!(
                #[mdl(property = "A", entry)]
                struct Bad(u32);
            ),
            "exactly one",
        );
        rejects(
            parse_quote!(
                #[mdl(entry)]
                struct Bad(u32, u32);
            ),
            "single-field tuple",
        );
        rejects(
            parse_quote!(
                #[mdl(property = "A")]
                struct Bad {
                    value: u32,
                }
            ),
            "single-field tuple",
        );
        rejects(
            parse_quote!(
                #[mdl(entry)]
                struct Bad(#[mdl(default)] u32);
            ),
            "field attributes",
        );
        rejects(
            parse_quote!(
                #[mdl(entry, write_order(value))]
                struct Bad(u32);
            ),
            "only supported on blocks",
        );
    }

    #[test]
    fn rejects_non_structs_tuple_structs_and_more_than_64_body_fields() {
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                enum Bad {
                    A,
                }
            ),
            "structs only",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad(u32);
            ),
            "named-field structs",
        );
        let fields = (0..65).map(|index| {
            let field = format_ident!("field_{index}");
            let mdl_name = format!("Field{index}");
            quote!(#[mdl(property = #mdl_name)] #field: u32)
        });
        rejects(
            syn::parse2(quote!(#[mdl(block = "A")] struct Bad { #(#fields,)* })).unwrap(),
            "at most 64",
        );
    }
    #[test]
    fn rejects_invalid_static_and_animation_attributes() {
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(static_property = "Value", skip_if = "empty")]
                    value: u32,
                }
            ),
            "skip_if requires",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(animatable = "Alpha", default)]
                    alpha: f32,
                }
            ),
            "track attribute",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(animatable = "Alpha", track = "Track::Alpha", default)]
                    alpha: f32,
                }
            ),
            "exactly one tracks",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(property = "Value", track = "Track::Alpha")]
                    value: u32,
                }
            ),
            "require an animatable",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(
                        animatable = "Alpha",
                        track = "Track::Alpha",
                        default,
                        enabled_if = "enabled"
                    )]
                    alpha: f32,
                    #[mdl(tracks)]
                    tracks: Vec<Track>,
                }
            ),
            "supplied together",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(
                        animatable = "Alpha",
                        track = "Track::Alpha",
                        default,
                        read_with = "read"
                    )]
                    alpha: f32,
                    #[mdl(tracks)]
                    tracks: Vec<Track>,
                }
            ),
            "value codec hooks",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(tracks)]
                    tracks: Vec<Track>,
                }
            ),
            "tracks requires animatable",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(tracks, default)]
                    tracks: Vec<Track>,
                }
            ),
            "tracks cannot",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(tracks)]
                    tracks: Option<Track>,
                }
            ),
            "Vec<TrackEnum>",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(animatable = "Alpha", track = "Track::Alpha", default)]
                    alpha: f32,
                    #[mdl(tracks)]
                    a: Vec<Track>,
                    #[mdl(tracks)]
                    b: Vec<Track>,
                }
            ),
            "exactly one tracks",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(animatable = "Alpha", track = "Track::Alpha", default)]
                    alpha: f32,
                    #[mdl(animatable = "Other", track = "Track::Alpha", default)]
                    other: f32,
                    #[mdl(tracks)]
                    tracks: Vec<Track>,
                }
            ),
            "duplicate track variant",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(static_property = "Value")]
                    a: f32,
                    #[mdl(property = "Value")]
                    b: f32,
                }
            ),
            "duplicate MDL field name",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(static_property = "Value")]
                    a: f32,
                    #[mdl(property = "static")]
                    b: f32,
                }
            ),
            "static is reserved",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(property = "Value", allow_bits = 2)]
                    a: u32,
                }
            ),
            "requires packed flags",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(flags(A = 1), allow_bits = 1)]
                    a: u32,
                }
            ),
            "must not overlap",
        );
    }
    #[test]
    fn rejects_invalid_record_defaults() {
        rejects(
            parse_quote!(
                #[mdl(block = "A", default, default)]
                struct Bad {}
            ),
            "duplicate container default",
        );
        rejects(
            parse_quote!(
                #[mdl(entry, default)]
                struct Bad(u32);
            ),
            "only supported on blocks",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A")]
                struct Bad {
                    #[mdl(property = "Value", required)]
                    value: u32,
                }
            ),
            "only needed with a container default",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A", default)]
                struct Bad {
                    #[mdl(property = "Value", required, default)]
                    value: u32,
                }
            ),
            "without defaults",
        );
        rejects(
            parse_quote!(
                #[mdl(block = "A", default)]
                struct Bad {
                    #[mdl(animatable = "Alpha", track = "Track::Alpha", required)]
                    alpha: f32,
                    #[mdl(tracks)]
                    tracks: Vec<Track>,
                }
            ),
            "required is only supported on properties",
        );
    }
}
