//! Codec bounds, kept separate from emitted parsing and writing code.
use super::attributes::{vec_element, Container, DefaultValue, Field, Kind};
use super::{omission, schema::Schema};
use syn::{parse_quote, DeriveInput, Generics, Result};

pub(super) fn uses_writer_defaults(options: &Container, fields: &[Field]) -> bool {
    options.default
        && fields.iter().any(|field| {
            field.default.is_none()
                && (matches!(field.kind, Kind::Flag(_) | Kind::Flags(_))
                    || omission::needs_check(field))
        })
}

pub(super) fn build(
    input: &DeriveInput,
    options: &Container,
    schema: &Schema,
    reading: bool,
) -> Result<Generics> {
    let fields = &schema.fields;
    let mut generics = input.generics.clone();
    for field in fields {
        let ty = &field.ty;
        if matches!(field.kind, Kind::Flatten | Kind::Block(_)) {
            let bound = if reading {
                parse_quote!(#ty: ::wc3::model::mdl::ReadFields)
            } else {
                parse_quote!(#ty: ::wc3::model::mdl::WriteFields)
            };
            generics.make_where_clause().predicates.push(bound);
        }
        if matches!(field.kind, Kind::Repeated(_) | Kind::Counted(_)) {
            let element = vec_element(ty)?;
            let bound = if reading {
                parse_quote!(#element: ::wc3::model::mdl::Read)
            } else {
                parse_quote!(#element: ::wc3::model::mdl::Write)
            };
            generics.make_where_clause().predicates.push(bound);
        }
        if matches!(field.kind, Kind::DelegatedProperty(_)) {
            let bound = if reading {
                parse_quote!(#ty: ::wc3::model::mdl::ReadProperty)
            } else {
                parse_quote!(#ty: ::wc3::model::mdl::WriteProperty)
            };
            generics.make_where_clause().predicates.push(bound);
        }
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
        if !reading && omission::needs_check(field) {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::wc3::model::mdl::ValueEq));
        }
        if !reading && matches!(field.kind, Kind::Tracks) {
            let element = vec_element(ty)?;
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
        if (reading || omission::needs_check(field))
            && !matches!(field.kind, Kind::Flags(_))
            && matches!(field.default, Some(DefaultValue::Trait))
        {
            generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#ty: ::core::default::Default));
        }
    }
    let write_defaults = uses_writer_defaults(options, fields);
    if (reading && options.default) || (!reading && write_defaults) {
        generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(Self: ::core::default::Default));
    }
    Ok(generics)
}
