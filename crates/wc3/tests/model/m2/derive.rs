use m2::codec::{Decoder, Encoder};
use m2::{Read as _, RecordLayout as _, Write as _};
use wc3::model::m2;
use wc3::model::m2::records::ModelFlags;

#[derive(Debug, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
struct Child<'data> {
    payload: m2::ZeroVec<'data, u16>,
    suffix: u32,
}
#[derive(Debug, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
struct Parent<'data> {
    prefix: u8,
    children: Vec<Child<'data>>,
    tail: [u32; 2],
}
#[derive(Debug, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
struct Tuple<T>(T, u16);
#[derive(Debug, PartialEq, m2::Read, m2::Write, m2::RecordLayout)]
struct Empty;

#[test]
fn derived_nested_arrays_preserve_later_metadata_and_borrow_unaligned_input() {
    let value = Parent {
        prefix: 7,
        children: vec![
            Child {
                payload: m2::ZeroVec::alloc_from_slice(&[11, 12, 13]),
                suffix: 0x12345678,
            },
            Child {
                payload: m2::ZeroVec::alloc_from_slice(&[21, 22]),
                suffix: 0xabcdef01,
            },
        ],
        tail: [0x10203040, 0x50607080],
    };
    let mut encoder = Encoder::new(264, ModelFlags::default());
    value.write(&mut encoder).unwrap();
    assert_eq!(encoder.position(), Parent::size(264, 0));
    let bytes = encoder.finish();
    // A deliberately unaligned address space must still borrow its leaf values.
    let mut unaligned = vec![0];
    unaligned.extend_from_slice(&bytes);
    let mut decoder = Decoder::new(&unaligned[1..], 264, ModelFlags::default());
    let mut decoded = Parent::read(&mut decoder).unwrap();
    assert_eq!(decoded, value);
    assert!(!decoded.children[0].payload.is_owned());
    assert_eq!(decoder.position(), Parent::size(264, 0));
    decoded.children[0].payload = m2::ZeroVec::alloc_from_slice(&[31, 32, 33, 34, 35]);
    let mut encoder = Encoder::new(264, ModelFlags::default());
    decoded.write(&mut encoder).unwrap();
    let rewritten = encoder.finish();
    assert_eq!(
        Parent::read(&mut Decoder::new(&rewritten, 264, ModelFlags::default())).unwrap(),
        decoded
    );
}

#[test]
fn generic_tuple_and_unit_records_compose() {
    let value = Tuple(0x12345678u32, 0xabcd);
    let mut encoder = Encoder::new(264, ModelFlags::default());
    value.write(&mut encoder).unwrap();
    Empty.write(&mut encoder).unwrap();
    assert_eq!(Tuple::<u32>::size(264, 0), 6);
    let bytes = encoder.finish();
    assert_eq!(bytes, [0x78, 0x56, 0x34, 0x12, 0xcd, 0xab]);
    let mut decoder = Decoder::new(&bytes, 264, ModelFlags::default());
    assert_eq!(Tuple::<u32>::read(&mut decoder).unwrap(), value);
    assert_eq!(Empty::read(&mut decoder).unwrap(), Empty);
}

struct ReadOnly(u32);
impl<'data> m2::Read<'data> for ReadOnly {
    fn read(decoder: &mut Decoder<'data>) -> Result<Self, m2::ReadError> {
        Ok(Self(decoder.u32()?))
    }
}
#[derive(m2::Read)]
struct ReadOnlyRecord(ReadOnly);
struct WriteOnly(u32);
impl m2::Write for WriteOnly {
    fn write(&self, encoder: &mut Encoder) -> Result<(), m2::WriteError> {
        encoder.u32(self.0)
    }
}
impl m2::RecordLayout for WriteOnly {
    fn size(_: u32, _: u32) -> usize {
        4
    }
}
#[derive(m2::Write, m2::RecordLayout)]
struct WriteOnlyRecord(WriteOnly);

#[test]
fn derives_keep_read_and_write_requirements_independent() {
    let mut encoder = Encoder::new(264, ModelFlags::default());
    encoder.write(&WriteOnlyRecord(WriteOnly(23))).unwrap();
    let bytes = encoder.finish();
    let value: ReadOnlyRecord = Decoder::new(&bytes, 264, ModelFlags::default())
        .read()
        .unwrap();
    assert_eq!(value.0 .0, 23);
}

#[test]
fn nonempty_zero_width_record_arrays_are_rejected() {
    let mut encoder = Encoder::new(264, ModelFlags::default());
    assert!(encoder.write_records(&[Empty]).is_err());
    let decoder = Decoder::new(&[], 264, ModelFlags::default());
    assert!(decoder
        .records::<Empty>(m2::codec::ArrayRef {
            count: 1,
            offset: 0
        })
        .is_err());
}

#[test]
fn native_record_layout_is_selected_by_the_type() {
    use m2::records::{Bone, Sequence, SplitBlendTime};
    use m2::{V272PreSplit, V256, V260, V264, V272};
    // A decoder's numeric context cannot switch a typed record's representation.
    assert_eq!(Bone::<V256>::size(274, 0), 108);
    assert_eq!(Bone::<V260>::size(256, 0), 112);
    assert_eq!(Bone::<V264>::size(256, 0), 88);
    let single = Sequence::<V272PreSplit> {
        blend_time: 0x12345678,
        ..Default::default()
    };
    let split = Sequence::<V272> {
        blend_time: SplitBlendTime {
            incoming: 0x5678,
            outgoing: 0x1234,
        },
        ..Default::default()
    };
    let mut encoder = Encoder::new(0, ModelFlags::default());
    single.write(&mut encoder).unwrap();
    let single_bytes = encoder.finish();
    let mut encoder = Encoder::new(0, ModelFlags::default());
    split.write(&mut encoder).unwrap();
    assert_eq!(encoder.finish(), single_bytes);
    let single_decoded =
        Sequence::<V272PreSplit>::read(&mut Decoder::new(&single_bytes, 0, ModelFlags::default()))
            .unwrap();
    let split_decoded =
        Sequence::<V272>::read(&mut Decoder::new(&single_bytes, 0, ModelFlags::default())).unwrap();
    assert_eq!(single_decoded.blend_time, 0x12345678);
    assert_eq!(
        split_decoded.blend_time,
        SplitBlendTime {
            incoming: 0x5678,
            outgoing: 0x1234
        }
    );
}

#[test]
fn document_traits_and_dynamic_layout_selection_preserve_framing() {
    let model = m2::Model::<m2::Flat<m2::V272PreSplit>>::new();
    let mut encoder = Encoder::new(0, ModelFlags::default());
    encoder.write(&model).unwrap();
    let bytes = encoder.finish();
    let decoded = m2::Model::<m2::Flat<m2::V272PreSplit>>::read(&mut Decoder::new(
        &bytes,
        0,
        ModelFlags::default(),
    ))
    .unwrap();
    assert_eq!(decoded, model);
    let dynamic = m2::DynamicModel::decode_m2_with_layout::<m2::V272PreSplit>(&bytes).unwrap();
    assert!(matches!(dynamic, m2::DynamicModel::Flat272PreSplit(_)));
    assert!(m2::DynamicModel::decode_m2_with_layout::<m2::V264>(&bytes).is_err());
    let mut written = Vec::new();
    m2::to_writer(&mut written, &dynamic).unwrap();
    assert_eq!(written, bytes);
}
