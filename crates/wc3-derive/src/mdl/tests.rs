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
        "enums support",
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
}

#[test]
fn delegated_properties_keep_name_validation_and_own_their_policies() {
    for modifier in [
        "default",
        "required",
        "skip_if = \"skip\"",
        "read_with = \"read\"",
        "write_with = \"write\"",
    ] {
        let input: DeriveInput = syn::parse_str(&format!(
            "#[mdl(block = \"A\")] struct Bad {{ #[mdl(property = \"Value\", delegate, {modifier})] value: u32 }}"
        )).unwrap();
        rejects(input, "delegate owns");
    }
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(property = "Value", delegate)]
                first: u32,
                #[mdl(property = "Value")]
                second: u32,
            }
        ),
        "duplicate MDL field name",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(property = "Bad Name", delegate)]
                value: u32,
            }
        ),
        "MDL identifier",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(property = "Value", delegate, property = "Value")]
                value: u32,
            }
        ),
        "exactly one",
    );
}

#[test]
fn delegate_requires_an_ordinary_property_and_cannot_repeat() {
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(header, delegate)]
                value: u32,
            }
        ),
        "delegate requires property",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(static_property = "Value", delegate)]
                value: u32,
            }
        ),
        "delegate requires property",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(property = "Value", delegate, delegate)]
                value: u32,
            }
        ),
        "duplicate delegate",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(property_codec = "Value")]
                value: u32,
            }
        ),
        "unknown MDL field attribute",
    );
}

#[test]
fn structural_fields_reject_conflicting_policies_and_names() {
    for (kind, ty, modifier) in [
        ("flatten", "Common", "default"),
        ("repeated = \"Child\"", "Vec<Child>", "default"),
        ("counted = \"Items\"", "Vec<Item>", "skip_if = \"skip\""),
        ("block = \"Target\"", "Target", "read_with = \"read\""),
    ] {
        let input: DeriveInput = syn::parse_str(&format!(
            "#[mdl(block = \"A\")] struct Bad {{ #[mdl({kind}, {modifier})] value: {ty} }}"
        ))
        .unwrap();
        rejects(input, "structural fields");
    }
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(counted = "Items")]
                items: u32,
            }
        ),
        "requires Vec",
    );
    rejects(
        parse_quote!(
            #[mdl(fields, block = "A")]
            struct Bad {
                #[mdl(property = "Id")]
                id: u32,
            }
        ),
        "choose exactly one",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(repeated = "Child")]
                children: Vec<Child>,
                #[mdl(block = "Child")]
                child: Child,
            }
        ),
        "duplicate MDL field name",
    );
}

#[test]
fn enums_reject_invalid_modes_names_and_shapes() {
    for (source, expected) in [
        ("enum Bad { A }", "require value or tagged"),
        ("#[mdl(value, tagged)] enum Bad { A }", "exactly one"),
        ("#[mdl(value)] enum Bad {}", "at least one"),
        ("#[mdl(value)] enum Bad { A(u32) }", "only unit variants"),
        (
            "#[mdl(value)] enum Bad { A { value: u32 } }",
            "unit or single-field tuple",
        ),
        (
            "#[mdl(tagged)] enum Bad { #[mdl(property = \"A\")] A(u32, u32) }",
            "unit or single-field tuple",
        ),
        (
            "#[mdl(value)] enum Bad { #[mdl(name = \"Same\")] A, #[mdl(name = \"Same\")] B }",
            "duplicate MDL enum variant name",
        ),
        (
            "#[mdl(value)] enum Bad { #[mdl(name = \"Bad Name\")] A }",
            "MDL identifier",
        ),
        ("#[mdl(value)] enum Bad { Nan }", "numeric literals"),
        (
            "#[mdl(value)] enum Bad { #[mdl(name = factory())] A }",
            "constant paths",
        ),
        (
            "#[mdl(value)] enum Bad { #[mdl(name = 1)] A }",
            "constant paths",
        ),
    ] {
        rejects(syn::parse_str(source).unwrap(), expected);
    }
}

#[test]
fn tagged_enum_framing_matches_variant_storage() {
    for (source, expected) in [
        ("#[mdl(tagged)] enum Bad { A }", "require explicit"),
        (
            "#[mdl(tagged)] enum Bad { #[mdl(name = \"A\")] A(u32) }",
            "name with delegate",
        ),
        (
            "#[mdl(tagged)] enum Bad { #[mdl(flag = \"A\")] A(u32) }",
            "flag for unit",
        ),
        (
            "#[mdl(tagged)] enum Bad { #[mdl(property = \"A\")] A }",
            "flag for unit",
        ),
        (
            "#[mdl(tagged)] enum Bad { #[mdl(name = \"A\", delegate)] A }",
            "single-field tuple",
        ),
        (
            "#[mdl(tagged)] enum Bad { #[mdl(block = \"A\", delegate)] A(u32) }",
            "delegate requires name",
        ),
        (
            "#[mdl(tagged)] enum Bad { #[mdl(name = \"A\", delegate, delegate)] A(u32) }",
            "duplicate delegate",
        ),
        (
            "#[mdl(tagged)] enum Bad { #[mdl(property = \"A\", block = \"B\")] A(u32) }",
            "exactly one variant",
        ),
        (
            "#[mdl(tagged)] enum Bad { #[mdl(property = \"A\")] A(#[mdl(default)] u32) }",
            "payload fields",
        ),
    ] {
        rejects(syn::parse_str(source).unwrap(), expected);
    }
}

#[test]
fn validates_unique_collections_and_reconstruction_hooks() {
    rejects(
        parse_quote! {
            #[mdl(block = "Record")]
            struct Bad { #[mdl(property = "Value", unique_by = "key")] value: u32 }
        },
        "unique_by requires repeated",
    );
    rejects(
        parse_quote! {
            #[mdl(block = "Record")]
            struct Bad { #[mdl(repeated(Child, Child))] children: Vec<u32> }
        },
        "duplicate",
    );
    rejects(
        parse_quote! {
            #[mdl(entry, after_read = "finish")]
            struct Bad(u32);
        },
        "after_read",
    );
    rejects(
        parse_quote! {
            #[mdl(block = "Record", after_read = "a", after_read = "b")]
            struct Bad {}
        },
        "duplicate",
    );
}

#[test]
fn bare_static_requires_a_static_animatable_channel() {
    rejects(
        parse_quote! {
            #[mdl(block = "Record")]
            struct Bad { #[mdl(property = "Value", bare_static)] value: f32 }
        },
        "bare_static requires",
    );
}

#[test]
fn validates_projection_channels_and_flattened_flags() {
    rejects(
        parse_quote! {
            #[mdl(block = "Record")]
            struct Bad { #[mdl(project())] data: Data }
        },
        "project needs",
    );
    rejects(
        parse_quote! {
            #[mdl(block = "Record")]
            struct Bad { #[mdl(project(#[mdl(property = "A")] a: u32))] data: (u32,) }
        },
        "named struct type",
    );
    rejects(
        parse_quote! {
            #[mdl(block = "Record")]
            struct Bad { #[mdl(project(#[mdl(property = "A")] a: u32, #[mdl(property = "B")] a: u32))] data: Data }
        },
        "duplicate projected member",
    );

    rejects(
        parse_quote! {
            #[mdl(block = "Record")]
            struct Bad { #[mdl(flatten, extra_flags(get = "get", A = 1))] data: Data }
        },
        "get and set",
    );
    rejects(
        parse_quote! {
            #[mdl(block = "Record")]
            struct Bad { #[mdl(property = "A", extra_flags(get = "get", set = "set", Flag = 1))] data: Data }
        },
        "extra_flags requires flatten",
    );
    rejects(
        parse_quote! {
            #[mdl(block = "Record")]
            struct Bad { #[mdl(flatten, extra_flags(get = "get", set = "set", A = 1, B = 1))] data: Data }
        },
        "distinct nonzero single",
    );
}

#[test]
fn virtual_fields_validate_storage_access_and_schema_composition() {
    rejects(
        parse_quote!(
            #[mdl(block = "A", virtual_fields())]
            struct A {}
        ),
        "needs at least one",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A", virtual_fields(
                #[mdl(property = "Value", default, get = "Self::value")]
                value: u32,
            ))]
            struct A {}
        ),
        "exactly one of set or slot",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A", virtual_fields(
                #[mdl(property = "Value", default, get = "Self::value", set = "Self::set", slot = "Self::slot")]
                value: u32,
            ))]
            struct A {}
        ),
        "exactly one of set or slot",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A", virtual_fields(
                #[mdl(property = "Value", get = "Self::value", slot = "Self::slot")]
                value: u32,
            ))]
            struct A {}
        ),
        "explicit default",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A", virtual_fields(
                #[mdl(header, get = "Self::value", set = "Self::set")]
                value: u32,
            ))]
            struct A {}
        ),
        "cannot be headers",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A", virtual_fields(
                #[mdl(property = "Value", get = "Self::value", set = "Self::set")]
                value: u32,
            ))]
            struct A {
                #[mdl(property = "Value")]
                stored: u32,
            }
        ),
        "duplicate MDL field name",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A", virtual_fields(
                #[mdl(property = "Value", get = "Self::value", set = "Self::set")]
                value: u32,
            ))]
            struct A {
                #[mdl(skip, default)]
                value: u32,
            }
        ),
        "duplicate virtual field member",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct A {
                #[mdl(property = "Value", get = "Self::value", set = "Self::set")]
                value: u32,
            }
        ),
        "require virtual_fields",
    );
    rejects(
        parse_quote!(
            #[mdl(entry, virtual_fields(
                #[mdl(property = "Value", get = "Self::value", set = "Self::set")]
                value: u32,
            ))]
            struct A(u32);
        ),
        "requires blocks or field groups",
    );
}

#[test]
fn dialect_attributes_reject_ambiguous_or_unmapped_names() {
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct A {
                #[mdl(property = "Value", hive_name = "Value")]
                value: u32,
            }
        ),
        "distinct spelling",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct A {
                #[mdl(skip, default, hive_name = "Other")]
                value: u32,
            }
        ),
        "requires a property",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct A {
                #[mdl(property = "Value", hive_name = "Other")]
                value: u32,
                #[mdl(property = "Other")]
                other: u32,
            }
        ),
        "duplicate MDL field name",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct A {
                #[mdl(flags(A = 1), hive_flags(B = 2))]
                bits: u32,
            }
        ),
        "engine mapping",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct A {
                #[mdl(flags(A = 1, B = 2), hive_flags(A = 2))]
                bits: u32,
            }
        ),
        "same bit",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct A {
                #[mdl(flags(A = 1, B = 2), hive_flags(C = 1, C = 2))]
                bits: u32,
            }
        ),
        "duplicate MDL field name",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct A {
                #[mdl(flags(A = 1), hive_flags(B = 1, C = 1))]
                bits: u32,
            }
        ),
        "distinct nonzero",
    );
}

#[test]
fn value_enums_validate_unknown_variant_shape_and_context() {
    for source in [
        "#[mdl(value)] enum Bad { #[mdl(unknown)] Unknown }",
        "#[mdl(value)] enum Bad { #[mdl(unknown)] Unknown(u32, u32) }",
        "#[mdl(value)] enum Bad { #[mdl(unknown)] A(u32), #[mdl(unknown)] B(u32) }",
        "#[mdl(tagged)] enum Bad { #[mdl(unknown)] Unknown(u32) }",
        "#[mdl(value)] enum Bad { #[mdl(unknown)] #[mdl(name = \"Other\")] Unknown(u32) }",
    ] {
        rejects(syn::parse_str(source).unwrap(), "unknown requires");
    }
}

#[test]
fn hive_skip_bits_require_mapped_packed_flags() {
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(property = "Value", hive_skip_bits = 1)]
                value: u32,
            }
        ),
        "hive_skip_bits requires packed flags",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(flags(Visible = 1), hive_skip_bits = 2)]
                flags: u32,
            }
        ),
        "hive_skip_bits must use mapped flags",
    );
    rejects(
        parse_quote!(
            #[mdl(block = "A")]
            struct Bad {
                #[mdl(flags(Visible = 1), hive_skip_bits = 1, hive_skip_bits = 1)]
                flags: u32,
            }
        ),
        "duplicate hive_skip_bits",
    );
}

#[test]
fn animated_properties_are_inferred_from_storage() {
    let inputs: [DeriveInput; 3] = [
        parse_quote! { #[mdl(block = "Record")] struct Record { #[mdl(property = "Color", default)] color: Animatable<Color> } },
        parse_quote! { #[mdl(block = "Record")] struct Record { #[mdl(property = "Translation")] translation: Option<Track<Vec3>> } },
        parse_quote! { #[mdl(block = "Record")] struct Record { #[mdl(property = "FocusDistanceKeys", constant = "DOFDistance")] focus_distance: Option<Track<f32>> } },
    ];
    for input in inputs {
        for reading in [true, false] {
            assert!(!expand_checked(input.clone(), reading).unwrap().is_empty());
        }
    }
    rejects(
        parse_quote! { #[mdl(block = "Record")] struct Bad { #[mdl(property = "Color")] color: Animatable<Color> } },
        "require a field or container default",
    );
    rejects(
        parse_quote! { #[mdl(block = "Record")] struct Bad { #[mdl(property = "Color", default, read_with = "read")] color: Animatable<Color> } },
        "value codec hooks",
    );
    rejects(
        parse_quote! { #[mdl(block = "Record")] struct Bad { #[mdl(property = "Color", default, enabled_if = "enabled")] color: Animatable<Color> } },
        "supplied together",
    );
    rejects(
        parse_quote! { #[mdl(block = "Record")] struct Bad { #[mdl(property = "Visibility", bare_static)] visibility: Option<Track<f32>> } },
        "bare_static requires",
    );
    rejects(
        parse_quote! { #[mdl(block = "Record")] struct Bad { #[mdl(property = "Color", default, constant = "ColorValue")] color: Animatable<Color> } },
        "constant requires",
    );
    rejects(
        parse_quote! { #[mdl(block = "Record")] struct Bad { #[mdl(property = "Keys", constant = "Value")] keys: Option<Track<f32>>, #[mdl(property = "Value")] value: u32 } },
        "duplicate MDL field name",
    );
}
