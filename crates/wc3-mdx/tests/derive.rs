use wc3_mdx::{Cursor, Encoder, Readable, Writable};

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
    encoder.write(&packet);
    encoder.write(&Pair(3, 4));
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
