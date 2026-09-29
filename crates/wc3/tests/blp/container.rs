use wc3::blp::{Blp, Blp1ContentRef, Blp2ContentRef, BlpRef, DxtFormat, ReadErrorKind, WriteError};

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

#[test]
fn blp1_indexed_borrows_mips_and_canonicalizes_offsets() {
    let mut bytes = vec![0; 1194];
    bytes[..4].copy_from_slice(b"BLP1");
    put_u32(&mut bytes, 4, 1);
    put_u32(&mut bytes, 8, 8);
    put_u32(&mut bytes, 12, 2);
    put_u32(&mut bytes, 16, 2);
    put_u32(&mut bytes, 20, 5);
    put_u32(&mut bytes, 28, 1184);
    put_u32(&mut bytes, 92, 10);
    bytes[156] = 11;
    bytes[1184..1194].copy_from_slice(&[0, 1, 2, 3, 10, 20, 30, 40, 99, 100]);

    let borrowed = BlpRef::read(&bytes).unwrap();
    let BlpRef::Blp1(value) = &borrowed else {
        panic!("expected BLP1");
    };
    assert_eq!(value.mipmaps[0].unwrap().as_ptr(), bytes[1184..].as_ptr());
    assert!(matches!(
        value.content,
        Blp1ContentRef::Indexed { palette } if palette[0] == 11
    ));
    let encoded = borrowed.write().unwrap();
    assert_eq!(&encoded[..4], b"BLP1");
    assert_eq!(
        u32::from_le_bytes(encoded[28..32].try_into().unwrap()),
        1180
    );
    assert_eq!(&encoded[1180..], &bytes[1184..]);
    assert_eq!(encoded.len(), 1190);

    let mut owned = borrowed.to_owned();
    let Blp::Blp1(ref mut value) = owned else {
        panic!("expected BLP1");
    };
    value.mipmaps[0] = Some(vec![7, 8]);
    let edited = owned.write().unwrap();
    assert_eq!(&edited[1180..], &[7, 8]);
    assert_eq!(BlpRef::read(&edited).unwrap().to_owned(), owned);
}

#[test]
fn blp2_jpeg_region_and_dxt_format_round_trip() {
    let mut bytes = vec![0xCC; 1176];
    bytes[..4].copy_from_slice(b"BLP2");
    put_u32(&mut bytes, 4, 1);
    bytes[8] = 0;
    bytes[9] = 8;
    bytes[10] = 9;
    bytes[11] = 0x11;
    put_u32(&mut bytes, 12, 1);
    put_u32(&mut bytes, 16, 1);
    for entry in &mut bytes[20..148] {
        *entry = 0;
    }
    put_u32(&mut bytes, 20, 1172);
    put_u32(&mut bytes, 84, 4);
    put_u32(&mut bytes, 148, 3);
    bytes[152..155].copy_from_slice(&[1, 2, 3]);
    bytes[1172..].copy_from_slice(&[4, 5, 6, 7]);

    let borrowed = BlpRef::read(&bytes).unwrap();
    let BlpRef::Blp2(value) = &borrowed else {
        panic!("expected BLP2");
    };
    assert!(matches!(
        value.content,
        Blp2ContentRef::Jpeg {
            alpha_type: 9,
            shared_header: [1, 2, 3],
            ..
        }
    ));
    assert_eq!(borrowed.write().unwrap(), bytes);
    assert_eq!(borrowed.to_owned().write().unwrap(), bytes);

    bytes[8] = 2;
    bytes[10] = 7;
    let dxt = BlpRef::read(&bytes).unwrap();
    assert!(matches!(
        dxt,
        BlpRef::Blp2(ref value) if matches!(value.content, Blp2ContentRef::Dxt { format: DxtFormat::Dxt5, .. })
    ));
    assert_eq!(dxt.write().unwrap(), bytes);
}

#[test]
fn rejects_out_of_bounds_mip_and_invalid_encoding() {
    let mut bytes = vec![0; 1172];
    bytes[..4].copy_from_slice(b"BLP2");
    put_u32(&mut bytes, 4, 1);
    bytes[8] = 3;
    put_u32(&mut bytes, 12, 1);
    put_u32(&mut bytes, 16, 1);
    put_u32(&mut bytes, 20, 1172);
    put_u32(&mut bytes, 84, 4);
    assert_eq!(
        BlpRef::read(&bytes).unwrap_err().kind,
        ReadErrorKind::InvalidRange { level: 0 }
    );
    bytes[8] = 99;
    assert!(matches!(
        BlpRef::read(&bytes).unwrap_err().kind,
        ReadErrorKind::InvalidValue {
            field: "color encoding",
            value: 99
        }
    ));
}

#[test]
fn writer_requires_base_mip() {
    let mut bytes = vec![0; 1173];
    bytes[..4].copy_from_slice(b"BLP2");
    put_u32(&mut bytes, 4, 1);
    bytes[8] = 3;
    put_u32(&mut bytes, 12, 1);
    put_u32(&mut bytes, 16, 1);
    put_u32(&mut bytes, 20, 1172);
    put_u32(&mut bytes, 84, 1);
    let mut owned = BlpRef::read(&bytes).unwrap().to_owned();
    let Blp::Blp2(ref mut value) = owned else {
        panic!("expected BLP2");
    };
    value.mipmaps[0] = None;
    assert_eq!(owned.write(), Err(WriteError::MissingMipmap { level: 0 }));
}

#[test]
fn blp1_jpeg_and_blp2_indexed_bgra_round_trip() {
    let mut jpeg = vec![0; 166];
    jpeg[..4].copy_from_slice(b"BLP1");
    put_u32(&mut jpeg, 12, 1);
    put_u32(&mut jpeg, 16, 1);
    put_u32(&mut jpeg, 28, 164);
    put_u32(&mut jpeg, 92, 2);
    put_u32(&mut jpeg, 156, 4);
    jpeg[160..164].copy_from_slice(&[1, 2, 3, 4]);
    jpeg[164..].copy_from_slice(&[5, 6]);
    let parsed = BlpRef::read(&jpeg).unwrap();
    assert!(matches!(
        parsed,
        BlpRef::Blp1(ref value) if matches!(value.content, Blp1ContentRef::Jpeg { shared_header: [1, 2, 3, 4] })
    ));
    assert_eq!(parsed.write().unwrap(), jpeg);

    let mut blp2 = vec![0; 1176];
    blp2[..4].copy_from_slice(b"BLP2");
    put_u32(&mut blp2, 4, 1);
    blp2[8] = 1;
    blp2[10] = 8;
    put_u32(&mut blp2, 12, 1);
    put_u32(&mut blp2, 16, 1);
    put_u32(&mut blp2, 20, 1172);
    put_u32(&mut blp2, 84, 4);
    blp2[148..152].copy_from_slice(&[9, 8, 7, 6]);
    blp2[1172..].copy_from_slice(&[1, 2, 3, 4]);
    let indexed = BlpRef::read(&blp2).unwrap();
    assert!(matches!(
        indexed,
        BlpRef::Blp2(ref value) if matches!(value.content, Blp2ContentRef::Indexed { alpha_type: 8, palette } if palette[..4] == [9, 8, 7, 6])
    ));
    assert_eq!(indexed.write().unwrap(), blp2);

    blp2[8] = 4;
    let bgra = BlpRef::read(&blp2).unwrap();
    assert!(matches!(
        bgra,
        BlpRef::Blp2(ref value) if matches!(value.content, Blp2ContentRef::Bgra { encoding: 4, .. })
    ));
    assert_eq!(bgra.write().unwrap(), blp2);
}
