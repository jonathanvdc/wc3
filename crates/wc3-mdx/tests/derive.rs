use wc3_mdx::io::{Cursor, EncodeError, Encoder};
use wc3_mdx::{Readable, Writable};

#[derive(Debug, PartialEq, Readable, Writable)]
struct Header {
    count: u32,
    values: [u16; 2],
}

#[derive(Debug, PartialEq, Readable, Writable)]
struct Packet {
    prefix: u8,
    header: Header,
    suffix: [u8; 3],
}

#[derive(Debug, PartialEq, Readable, Writable)]
struct Pair(u16, u32);

struct FailingField;

impl Writable for FailingField {
    fn write_to(&self, _: &mut Encoder<'_>) -> Result<(), EncodeError> {
        Err(EncodeError::ChunkTooLarge {
            tag: *b"TEST",
            size: usize::MAX,
        })
    }
}

#[derive(Writable)]
struct FalliblePacket {
    prefix: u8,
    field: FailingField,
    suffix: u8,
}

#[test]
fn derived_codecs_follow_field_order_without_copying_structs() {
    let packet = Packet {
        prefix: 7,
        header: Header {
            count: 2,
            values: [0x1234, 0x5678],
        },
        suffix: [9, 10, 11],
    };
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes);
    encoder.write(&packet).unwrap();
    encoder.write(&Pair(3, 4)).unwrap();
    assert_eq!(
        bytes,
        [7, 2, 0, 0, 0, 0x34, 0x12, 0x78, 0x56, 9, 10, 11, 3, 0, 4, 0, 0, 0,]
    );
    let mut cursor = Cursor::new(&bytes);
    assert_eq!(cursor.read::<Packet>().unwrap(), packet);
    assert_eq!(cursor.read::<Pair>().unwrap(), Pair(3, 4));
    cursor.finish().unwrap();
    assert_eq!(packet.header.count, 2);
}

#[test]
fn derived_writer_propagates_field_errors() {
    let packet = FalliblePacket {
        prefix: 7,
        field: FailingField,
        suffix: 9,
    };
    let mut bytes = Vec::new();
    let error = Encoder::new(&mut bytes).write(&packet).unwrap_err();
    assert_eq!(
        error,
        EncodeError::ChunkTooLarge {
            tag: *b"TEST",
            size: usize::MAX,
        }
    );
    assert_eq!(bytes, [7]);
}

const TEST_TAG: [u8; 4] = *b"TEST";

#[derive(Debug, PartialEq, Readable, Writable)]
#[mdx(sized(tag = TEST_TAG))]
struct SizedPacket<T> {
    marker: std::marker::PhantomData<T>,
    header: u16,
    values: Vec<u32>,
}

#[test]
fn sized_derive_bounds_the_final_vector_to_each_record() {
    let first = SizedPacket::<()> {
        marker: std::marker::PhantomData,
        header: 7,
        values: vec![11, 12],
    };
    let second = SizedPacket::<()> {
        marker: std::marker::PhantomData,
        header: 8,
        values: vec![],
    };
    let mut bytes = Vec::new();
    let mut encoder = Encoder::new(&mut bytes);
    encoder.write(&first).unwrap();
    encoder.write(&second).unwrap();
    assert_eq!(&bytes[..6], &[14, 0, 0, 0, 7, 0]);
    assert_eq!(&bytes[14..20], &[6, 0, 0, 0, 8, 0]);
    let mut cursor = Cursor::new(&bytes);
    assert_eq!(cursor.read::<SizedPacket<()>>().unwrap(), first);
    assert_eq!(cursor.read::<SizedPacket<()>>().unwrap(), second);
    cursor.finish().unwrap();
}

#[test]
fn sized_derive_rejects_truncated_vector_item() {
    let error = SizedPacket::<()>::decode(&[8, 0, 0, 0, 7, 0, 11, 0]).unwrap_err();
    assert_eq!(
        error,
        wc3_mdx::DecodeError::UnexpectedEnd {
            offset: 6,
            needed: 4
        }
    );
}

#[derive(Debug, PartialEq, Readable, Writable)]
#[mdx(sized(tag = TEST_TAG))]
struct EmptyItems {
    values: Vec<EmptyItem>,
}

#[derive(Debug, PartialEq, Readable, Writable)]
struct EmptyItem;

#[test]
fn sized_derive_rejects_zero_width_items() {
    let error = EmptyItems::decode(&[5, 0, 0, 0, 42]).unwrap_err();
    assert_eq!(
        error,
        wc3_mdx::DecodeError::MalformedRecord {
            tag: TEST_TAG,
            offset: 4
        }
    );
}
