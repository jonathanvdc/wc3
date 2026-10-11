use std::borrow::Cow;
use wc3::blp::{Blp, Blp1Content, Blp2Content, DxtFormat, ReadErrorKind, WriteError};

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

    let borrowed = Blp::read(&bytes).unwrap();
    let Blp::Blp1(value) = &borrowed else {
        panic!("expected BLP1");
    };
    assert_eq!(
        value.mipmaps[0].as_ref().unwrap().as_ptr(),
        bytes[1184..].as_ptr()
    );
    assert!(matches!(
        &value.content,
        Blp1Content::Indexed { palette } if palette[0] == 11
    ));
    let encoded = borrowed.write().unwrap();
    assert_eq!(&encoded[..4], b"BLP1");
    assert_eq!(
        u32::from_le_bytes(encoded[28..32].try_into().unwrap()),
        1180
    );
    assert_eq!(&encoded[1180..], &bytes[1184..]);
    assert_eq!(encoded.len(), 1190);

    let mut owned = borrowed.into_owned();
    let Blp::Blp1(ref mut value) = owned else {
        panic!("expected BLP1");
    };
    value.mipmaps[0] = Some(Cow::Owned(vec![7, 8]));
    let edited = owned.write().unwrap();
    assert_eq!(&edited[1180..], &[7, 8]);
    assert_eq!(Blp::read(&edited).unwrap().into_owned(), owned);
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

    let borrowed = Blp::read(&bytes).unwrap();
    let Blp::Blp2(value) = &borrowed else {
        panic!("expected BLP2");
    };
    assert!(matches!(
        &value.content,
        Blp2Content::Jpeg {
            alpha_type: 9,
            shared_header,
            ..
        } if shared_header.as_ref() == [1, 2, 3]
    ));
    assert_eq!(borrowed.write().unwrap(), bytes);
    assert_eq!(borrowed.into_owned().write().unwrap(), bytes);

    bytes[8] = 2;
    bytes[10] = 7;
    let dxt = Blp::read(&bytes).unwrap();
    assert!(matches!(
        dxt,
        Blp::Blp2(ref value) if matches!(&value.content, Blp2Content::Dxt { format: DxtFormat::Dxt5, .. })
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
        Blp::read(&bytes).unwrap_err().kind,
        ReadErrorKind::InvalidRange { level: 0 }
    );
    bytes[8] = 99;
    assert!(matches!(
        Blp::read(&bytes).unwrap_err().kind,
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
    let mut owned = Blp::read(&bytes).unwrap().into_owned();
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
    let parsed = Blp::read(&jpeg).unwrap();
    assert!(matches!(
        parsed,
        Blp::Blp1(ref value) if matches!(&value.content, Blp1Content::Jpeg { shared_header } if shared_header.as_ref() == [1, 2, 3, 4])
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
    let indexed = Blp::read(&blp2).unwrap();
    assert!(matches!(
        indexed,
        Blp::Blp2(ref value) if matches!(&value.content, Blp2Content::Indexed { alpha_type: 8, palette } if palette[..4] == [9, 8, 7, 6])
    ));
    assert_eq!(indexed.write().unwrap(), blp2);

    blp2[8] = 4;
    let bgra = Blp::read(&blp2).unwrap();
    assert!(matches!(
        bgra,
        Blp::Blp2(ref value) if matches!(&value.content, Blp2Content::Bgra { encoding: 4, .. })
    ));
    assert_eq!(bgra.write().unwrap(), blp2);
}

#[test]
fn blp0_root_and_sparse_external_mips_round_trip() {
    use wc3::blp::{Blp0, Blp0Content, Blp0Header, WriteError};

    let texture = Blp0 {
        header: Blp0Header {
            width: 2,
            height: 1,
            alpha_bits: 8,
            extra: 4,
            has_mipmaps: 1,
        },
        content: Blp0Content::Indexed {
            palette: Box::new([0; 1024]).into(),
        },
        mipmaps: std::array::from_fn(|level| match level {
            0 => Some(Cow::Owned(vec![0, 1, 255, 0])),
            3 => Some(Cow::Owned(vec![1, 127])),
            _ => None,
        }),
    };
    assert_eq!(texture.write(), Err(WriteError::ExternalMipmapsRequired));
    let output = texture.write_files().unwrap();
    assert_eq!(output.root.len(), 28 + 1024);
    let mut parsed = Blp::read(&output.root).unwrap();
    let Blp::Blp0(value) = &mut parsed else {
        panic!("expected BLP0")
    };
    assert!(value.mipmaps.iter().all(Option::is_none));
    value.mipmaps = output
        .mipmaps
        .each_ref()
        .map(|mip| mip.as_deref().map(Cow::Borrowed));
    assert_eq!(value.as_ref().into_owned(), texture);
    assert_eq!(parsed.write_files().unwrap(), output);
    for end in 0..output.root.len() {
        assert!(Blp::read(&output.root[..end]).is_err());
    }
}
