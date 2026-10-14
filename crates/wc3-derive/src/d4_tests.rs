use super::{expand_checked, Operation};
use syn::{parse_quote, DeriveInput};

fn diagnostic(input: DeriveInput) -> String {
    expand_checked(input, Operation::Layout)
        .expect_err("invalid declaration must fail")
        .to_string()
}

#[test]
fn record_size_and_field_offset_attributes_are_rejected() {
    assert!(diagnostic(parse_quote!(
        #[d4(size = 4)]
        struct Sized {}
    ))
    .contains("record-level D4 attributes"));
    assert!(diagnostic(parse_quote!(
        struct Offset {
            #[d4(offset = 0, value)]
            field: u32,
        }
    ))
    .contains("unknown D4 field attribute"));
}

#[test]
fn incompatible_or_unknown_encodings_are_rejected() {
    assert!(diagnostic(parse_quote!(
        struct Conflict {
            #[d4(value, array)]
            field: u32,
        }
    ))
    .contains("conflicting field encodings"));
    assert!(diagnostic(parse_quote!(
        struct External {
            #[d4(value, external)]
            field: u32,
        }
    ))
    .contains("external requires"));
    assert!(diagnostic(parse_quote!(
        struct External {
            #[d4(bytes, external)]
            field: Vec<u8>,
        }
    ))
    .contains("external requires"));
    assert!(diagnostic(parse_quote!(
        struct Unknown {
            #[d4(magic)]
            field: u32,
        }
    ))
    .contains("unknown D4 field attribute"));
}

#[test]
fn container_encodings_require_matching_field_shapes() {
    assert!(diagnostic(parse_quote!(
        struct Wrong {
            #[d4(array)]
            field: u32,
        }
    ))
    .contains("array requires Vec"));
    assert!(diagnostic(parse_quote!(
        struct Wrong {
            #[d4(inline_array)]
            field: Vec<u32>,
        }
    ))
    .contains("inline_array requires"));
}

#[test]
fn tuple_and_enum_layouts_require_handwritten_codecs() {
    assert!(diagnostic(parse_quote!(
        struct Tuple(u32);
    ))
    .contains("named fields"));
    assert!(diagnostic(parse_quote!(
        enum Choice {
            A,
            B,
        }
    ))
    .contains("require a struct"));
}

#[test]
fn generic_layouts_infer_widths_and_accept_implicit_inline_fields() {
    let input: DeriveInput = parse_quote!(
        struct Generic<const N: usize> {
            #[d4(value)]
            field: [u64; N],
            tail: InlineRecord,
        }
    );
    for operation in [Operation::Layout, Operation::Read, Operation::Write] {
        assert!(expand_checked(input.clone(), operation).is_ok());
    }
}
