//! Codec bounds, kept separate from emitted parsing and writing code.
use super::attributes::{vec_element, Container, DefaultValue, Field, Kind};
use super::{omission, schema::Schema};
use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use syn::{parse_quote, DeriveInput, Generics, Result};

pub(super) fn uses_writer_defaults(options: &Container, fields: &[Field]) -> bool {
    options.default
        && fields.iter().any(|field| {
            field.default.is_none()
                && (matches!(field.kind, Kind::Flag(_) | Kind::Flags(_))
                    || omission::needs_check(field))
        })
}

fn mentions_parameter(tokens: TokenStream, parameters: &[String]) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(name) => parameters.iter().any(|parameter| name == parameter),
        TokenTree::Group(group) => mentions_parameter(group.stream(), parameters),
        _ => false,
    })
}

pub(super) fn build(
    input: &DeriveInput,
    options: &Container,
    schema: &Schema,
    reading: bool,
) -> Result<Generics> {
    let fields = &schema.fields;
    let mut parameters: Vec<_> = input
        .generics
        .type_params()
        .map(|p| p.ident.to_string())
        .collect();
    parameters.push("Self".into());
    let mut generics = input.generics.clone();
    for field in fields {
        let ty = &field.ty;
        // A concrete virtual read type already has a known writer implementation.
        // Repeating its trait bound hides the associated State when the getter
        // returns a borrowed view with that same state.
        if matches!(field.kind, Kind::Flatten | Kind::Block(_))
            && (reading
                || !field.virtual_field
                || mentions_parameter(ty.to_token_stream(), &parameters))
        {
            let bound = if reading {
                parse_quote!(#ty: ::wc3::model::mdl::ReadFields)
            } else {
                parse_quote!(#ty: ::wc3::model::mdl::WriteFields)
            };
            generics.make_where_clause().predicates.push(bound);
        }
        if matches!(field.kind, Kind::Repeated(_) | Kind::Counted(_))
            && (reading || !field.virtual_field)
        {
            let element = vec_element(ty)?;
            let bound = if reading {
                parse_quote!(#element: ::wc3::model::mdl::Read)
            } else {
                parse_quote!(#element: ::wc3::model::mdl::Write)
            };
            generics.make_where_clause().predicates.push(bound);
        }
        if matches!(field.kind, Kind::DelegatedProperty(_)) && (reading || !field.virtual_field) {
            let bound = if reading {
                parse_quote!(#ty: ::wc3::model::mdl::ReadProperty)
            } else {
                parse_quote!(#ty: ::wc3::model::mdl::WriteProperty)
            };
            generics.make_where_clause().predicates.push(bound);
        }
        if matches!(field.kind, Kind::Animatable(_)) && (reading || !field.virtual_field) {
            let bound = if reading {
                parse_quote!(#ty: ::wc3::model::mdl::ReadAnimationProperty)
            } else {
                parse_quote!(#ty: ::wc3::model::mdl::WriteAnimationProperty)
            };
            generics.make_where_clause().predicates.push(bound);
        }
        if matches!(
            field.kind,
            Kind::Header | Kind::Property(_) | Kind::StaticProperty(_)
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
                if !options.default || field.default.is_some() || field.virtual_field {
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
