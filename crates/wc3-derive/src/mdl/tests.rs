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
