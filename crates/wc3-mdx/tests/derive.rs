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
