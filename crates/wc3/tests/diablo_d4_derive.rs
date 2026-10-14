//! Public custom-record coverage for the native Diablo IV derive contract.
use d4::codec::{Decoder, Encoder, Space};
use d4::records::{Polymorphic, TexAnim};
use d4::{Read as _, RecordLayout as _, Write as _};
use std::{array::from_fn, borrow::Cow};
use wc3::model::common;
use wc3::model::diablo::d4;
use zerovec::ZeroVec;

#[derive(Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout)]
struct Empty {}

#[test]
fn empty_named_records_infer_zero_width() {
    assert_eq!(Empty::SIZE, 0);
    let source = Empty {};
    let mut encoder = Encoder::new(Empty::SIZE);
    source.write(&mut encoder, 0, Space::Main).unwrap();
    let (body, payload) = encoder.finish();
    assert!(body.is_empty() && payload.is_empty());
    assert_eq!(Empty::read(&Decoder::new(&body, None), 0).unwrap(), source);
}

#[derive(Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout)]
struct PackedWire {
    #[d4(value)]
    tag: u8,
    #[d4(value)]
    value: u32,
}

#[test]
fn sequential_fields_use_encoded_widths_without_implicit_padding() {
    let source = PackedWire {
        tag: 3,
        value: 0x12345678,
    };
    let mut encoder = Encoder::new(PackedWire::SIZE);
    source.write(&mut encoder, 0, Space::Main).unwrap();
    let (body, payload) = encoder.finish();
    assert_eq!(body, [3, 0x78, 0x56, 0x34, 0x12]);
    assert!(payload.is_empty());
    assert_eq!(
        PackedWire::read(&Decoder::new(&body, None), 0).unwrap(),
        source
    );
}

#[derive(Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout)]
struct EmptyArray {
    #[d4(inline_array)]
    values: [u8; 0],
}

#[test]
fn empty_scalar_arrays_have_an_inline_encoding() {
    let empty = EmptyArray { values: [] };
    let mut encoder = Encoder::new(EmptyArray::SIZE);
    empty.write(&mut encoder, 0, Space::Main).unwrap();
    let (body, payload) = encoder.finish();
    assert!(body.is_empty() && payload.is_empty());
    assert_eq!(
        EmptyArray::read(&Decoder::new(&body, None), 0).unwrap(),
        empty
    );
}

#[test]
fn scalar_helpers_reject_zero_width_values() {
    assert!(Decoder::new(&[], None).value::<[u8; 0]>(0).is_err());
    let mut encoder = Encoder::new(0);
    assert!(encoder.value(0, &[0u8; 0], Space::Main).is_err());
}

#[test]
fn leaf_array_helpers_reject_zero_width_elements_in_both_directions() {
    assert!(Decoder::new(&[0; 16], None)
        .array::<[u8; 0]>(0, false)
        .is_err());
    let mut encoder = Encoder::new(16);
    let values = ZeroVec::<[u8; 0]>::new();
    assert!(encoder.array(0, &values, Space::Main, false).is_err());
}

#[derive(Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout)]
struct EmptyElements {
    #[d4(array)]
    values: Vec<EmptyArray>,
}

#[test]
fn dynamic_arrays_reject_zero_stride_in_both_directions() {
    let mut encoder = Encoder::new(EmptyElements::SIZE);
    let records = EmptyElements {
        values: vec![EmptyArray { values: [] }],
    };
    assert!(records.write(&mut encoder, 0, Space::Main).is_err());
    assert!(EmptyElements::read(&Decoder::new(&[0; 16], None), 0).is_err());
}

#[derive(Clone, Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout)]
struct Tree<T> {
    #[d4(array)]
    children: Vec<Tree<T>>,
    #[d4(value)]
    value: T,
}

#[test]
fn recursive_record_arrays_compose_without_recursive_trait_bounds() {
    let tree = Tree {
        value: 1u32,
        children: vec![Tree {
            value: 2,
            children: vec![Tree {
                value: 3,
                children: vec![],
            }],
        }],
    };
    let mut encoder = Encoder::new(Tree::<u32>::SIZE);
    tree.write(&mut encoder, 0, Space::Main).unwrap();
    let (body, payload) = encoder.finish();
    assert!(payload.is_empty());
    let decoded = Tree::read(&Decoder::new(&body, None), 0).unwrap();
    assert_eq!(tree, decoded);
}

#[derive(Clone, Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout, common::Ownership)]
struct Scalar {
    #[d4(value)]
    kind: u16,
    #[d4(value)]
    reserved: [u8; 2],
    #[d4(value)]
    marker: u32,
}

#[derive(Clone, Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout, common::Ownership)]
struct Leaf<'data> {
    #[d4(value)]
    reserved_prefix: [u8; 8],
    #[d4(array)]
    values: ZeroVec<'data, u16>,
    #[d4(value)]
    reserved_suffix: [u8; 4],
    #[d4(value)]
    tail: u32,
}

#[derive(Clone, Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout, common::Ownership)]
struct InlinePair<T> {
    #[d4(inline_array)]
    values: [T; 2],
}

#[derive(Clone, Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout, common::Ownership)]
struct FixedScalars<const N: usize> {
    #[d4(inline_array)]
    values: [Scalar; N],
}

#[derive(Clone, Debug, PartialEq, d4::Read, d4::Write, d4::RecordLayout, common::Ownership)]
struct Root<'data> {
    // Fields follow native wire order, including explicit reserved gaps.
    scalar: Scalar,
    #[d4(value)]
    reserved_prefix: [u8; 8],
    #[d4(array)]
    main_children: Vec<Leaf<'data>>,
    #[d4(array, external)]
    payload_children: Vec<Leaf<'data>>,
    #[d4(array, external)]
    payload_values: ZeroVec<'data, u32>,
    #[d4(bytes)]
    bytes: Cow<'data, [u8]>,
    inline: InlinePair<Scalar>,
    #[d4(polymorphic = 911604182)]
    main_polymorphic: Vec<Polymorphic<'data>>,
    #[d4(polymorphic = 911604182, external)]
    payload_polymorphic: Vec<Polymorphic<'data>>,
    #[d4(value)]
    reserved_suffix: [u8; 12],
    #[d4(value)]
    tail: u32,
}

fn scalar(seed: u16) -> Scalar {
    Scalar {
        marker: 0xabcd0000 | u32::from(seed),
        kind: seed,
        reserved: [0x71, 0x82],
    }
}
fn leaf(values: &[u16], tail: u32) -> Leaf<'static> {
    Leaf {
        tail,
        values: ZeroVec::alloc_from_slice(values),
        reserved_prefix: [0x17; 8],
        reserved_suffix: [0x28; 4],
    }
}
fn polymorphic(scale: f32) -> Polymorphic<'static> {
    let mut reserved_0 = [0; 8];
    reserved_0[..4].copy_from_slice(&911604182u32.to_le_bytes());
    Polymorphic::TexAnim(Box::new(TexAnim {
        reserved_0,
        type_: 17,
        u_scale: scale,
        v_scale: scale + 1.,
        ..Default::default()
    }))
}
fn source() -> Root<'static> {
    Root {
        tail: 0x12345678,
        payload_values: ZeroVec::alloc_from_slice(&[0x11223344, 0x55667788]),
        payload_children: vec![leaf(&[21, 22, 23], 31)],
        main_children: vec![leaf(&[1, 2], 11), leaf(&[3], 12)],
        bytes: Cow::Borrowed(b"native bytes\0"),
        scalar: scalar(5),
        reserved_prefix: [0x39; 8],
        inline: InlinePair {
            values: [scalar(6), scalar(7)],
        },
        main_polymorphic: vec![polymorphic(1.25)],
        payload_polymorphic: vec![polymorphic(9.5)],
        reserved_suffix: [0x4a; 12],
    }
}
fn encode<T: d4::Write>(value: &T) -> (Vec<u8>, Vec<u8>) {
    let mut encoder = Encoder::new(T::SIZE);
    value.write(&mut encoder, 0, Space::Main).unwrap();
    encoder.finish()
}
fn word(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

#[test]
fn sequential_fields_preserve_native_wire_order_reserved_gaps_and_nested_generic_records() {
    let original = source();
    let (main, payload) = encode(&original);
    assert_eq!(Root::SIZE, 160);
    assert_eq!(&main[..8], &[5, 0, 0x71, 0x82, 5, 0, 0xcd, 0xab]);
    assert_eq!(&main[..2], &5u16.to_le_bytes());
    assert_eq!(&main[2..4], &[0x71, 0x82]);
    assert_eq!(word(&main, 4), 0xabcd0005);
    assert_eq!(&main[8..16], &[0x39; 8]);
    assert_eq!(word(&main, 84), 0xabcd0006);
    assert_eq!(word(&main, 92), 0xabcd0007);
    assert_eq!(&main[144..156], &[0x4a; 12]);
    assert_eq!(word(&main, 156), original.tail);
    assert_eq!(
        Root::read(&Decoder::new(&main, Some(&payload)), 0).unwrap(),
        original
    );
}

#[test]
fn external_nested_descriptors_use_payload_relative_addresses_and_borrow_unaligned_leaves() {
    let original = source();
    let (main, payload) = encode(&original);
    let main_children = word(&main, 24) as usize;
    let payload_children = word(&main, 40) as usize;
    let main_leaf = word(&main, main_children + 16) as usize;
    let payload_leaf = word(&payload, payload_children + 16) as usize;
    assert!(main_children >= Root::SIZE);
    assert!(main_leaf >= Root::SIZE);
    assert_eq!(&main[main_leaf..main_leaf + 4], &[1, 0, 2, 0]);
    assert_eq!(
        &payload[payload_leaf..payload_leaf + 6],
        &[21, 0, 22, 0, 23, 0]
    );
    assert_eq!(
        Root::read(&Decoder::new(&main, None), 0).unwrap_err(),
        d4::ReadError::MissingPayload
    );
    let mut unaligned_main = vec![0];
    unaligned_main.extend_from_slice(&main);
    let mut unaligned_payload = vec![0];
    unaligned_payload.extend_from_slice(&payload);
    let parsed = Root::read(
        &Decoder::new(&unaligned_main[1..], Some(&unaligned_payload[1..])),
        0,
    )
    .unwrap();
    assert_eq!(parsed, original);
    assert!(!parsed.main_children[0].values.is_owned());
    assert!(!parsed.payload_children[0].values.is_owned());
    assert!(!parsed.payload_values.is_owned());
    assert!(matches!(parsed.bytes, Cow::Borrowed(_)));
}

#[test]
fn editing_main_array_relocates_following_payloads_without_corrupting_external_children() {
    let (main, payload) = encode(&source());
    let mut parsed = Root::read(&Decoder::new(&main, Some(&payload)), 0).unwrap();
    let old_bytes_offset = word(&main, 72);
    parsed.main_children.push(leaf(&[50, 51, 52, 53, 54], 99));
    parsed.main_children[0].values = ZeroVec::alloc_from_slice(&[100, 101, 102, 103]);
    let (rewritten_main, rewritten_payload) = encode(&parsed);
    assert_ne!(word(&rewritten_main, 72), old_bytes_offset);
    assert_eq!(rewritten_payload, payload);
    assert_eq!(
        Root::read(&Decoder::new(&rewritten_main, Some(&rewritten_payload)), 0).unwrap(),
        parsed
    );
}

#[test]
fn common_ownership_detaches_borrowed_buffers_and_reborrows_owned_storage() {
    let owned: Root<'static> = {
        let (main, payload) = encode(&source());
        Root::read(&Decoder::new(&main, Some(&payload)), 0)
            .unwrap()
            .into_owned()
    };
    assert!(owned.main_children[0].values.is_owned());
    assert!(owned.payload_children[0].values.is_owned());
    assert!(owned.payload_values.is_owned());
    assert!(matches!(owned.bytes, Cow::Owned(_)));
    let borrowed = owned.as_borrowed();
    assert!(!borrowed.main_children[0].values.is_owned());
    assert!(!borrowed.payload_children[0].values.is_owned());
    assert_eq!(borrowed, owned);
    assert_eq!(borrowed.bytes.as_ptr(), owned.bytes.as_ptr());
}

#[test]
fn derived_polymorphic_descriptors_check_their_native_prefix_tables() {
    let (mut main, payload) = encode(&source());
    let parsed = Root::read(&Decoder::new(&main, Some(&payload)), 0).unwrap();
    assert_eq!(parsed.main_polymorphic, vec![polymorphic(1.25)]);
    assert_eq!(parsed.payload_polymorphic, vec![polymorphic(9.5)]);
    let table_offset = word(&main, 104) as usize;
    main[table_offset] = 1;
    assert!(Root::read(&Decoder::new(&main, Some(&payload)), 0).is_err());
}

#[test]
fn whole_record_guards_reject_truncation_and_overflow_before_field_arithmetic() {
    let (main, payload) = encode(&source());
    assert!(Root::read(&Decoder::new(&main[..Root::SIZE - 1], Some(&payload)), 0).is_err());
    assert_eq!(
        Root::read(&Decoder::new(&main, Some(&payload)), usize::MAX).unwrap_err(),
        d4::ReadError::Overflow
    );
    let mut short = Encoder::new(Root::SIZE - 1);
    assert!(source().write(&mut short, 0, Space::Main).is_err());
    let mut encoder = Encoder::new(Root::SIZE);
    assert_eq!(
        source()
            .write(&mut encoder, usize::MAX, Space::Main)
            .unwrap_err(),
        d4::WriteError::Overflow
    );
}

#[test]
fn const_generic_record_sizes_follow_empty_and_nonempty_inline_arrays() {
    let empty = FixedScalars::<0> { values: [] };
    assert_eq!(FixedScalars::<0>::SIZE, 0);
    let (main, payload) = encode(&empty);
    assert!(main.is_empty());
    assert!(payload.is_empty());
    assert_eq!(
        FixedScalars::<0>::read(&Decoder::new(&main, None), 0).unwrap(),
        empty
    );
    let pair = FixedScalars::<2> {
        values: from_fn(|index| scalar(index as u16 + 30)),
    };
    assert_eq!(FixedScalars::<2>::SIZE, 16);
    let (main, payload) = encode(&pair);
    assert_eq!(main.len(), 16);
    assert!(payload.is_empty());
    assert_eq!(word(&main, 4), 0xabcd001e);
    assert_eq!(word(&main, 12), 0xabcd001f);
    assert_eq!(
        FixedScalars::<2>::read(&Decoder::new(&main, None), 0).unwrap(),
        pair
    );
}

#[test]
fn generic_record_width_is_inferred_from_its_actual_field_types() {
    assert_eq!(Tree::<u32>::SIZE, 20);
    assert_eq!(Tree::<u64>::SIZE, 24);
    let tree = Tree {
        children: vec![Tree {
            children: vec![],
            value: 0x1122334455667788u64,
        }],
        value: 0x8877665544332211u64,
    };
    let (body, payload) = encode(&tree);
    assert!(payload.is_empty());
    assert_eq!(&body[16..24], &0x8877665544332211u64.to_le_bytes());
    let child = word(&body, 8) as usize;
    assert_eq!(
        &body[child + 16..child + 24],
        &0x1122334455667788u64.to_le_bytes()
    );
    assert_eq!(
        Tree::<u64>::read(&Decoder::new(&body, None), 0).unwrap(),
        tree
    );
}
