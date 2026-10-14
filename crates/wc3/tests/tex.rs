use std::borrow::Cow;
use wc3::tex::{d3, d4, Error};

fn put(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn d3_fixture() -> Vec<u8> {
    let mut bytes = vec![0; 628];
    for (at, value) in [
        (0, 0xdeadbeef),
        (4, 47),
        (16, 123),
        (32, 0),
        (36, 1),
        (40, 1),
        (44, 1),
        (56, 624),
        (60, 4),
    ] {
        put(&mut bytes, at, value);
    }
    bytes[624..].copy_from_slice(&[3, 2, 1, 4]);
    bytes
}
fn d4_fixture() -> Vec<u8> {
    let mut bytes = vec![0; 144];
    for (at, value) in [
        (0, 0xdeadbeef),
        (4, 0xf9cd83e7),
        (16, 123),
        (28, 1),
        (40, 1),
        (88, 120),
        (92, 8),
    ] {
        put(&mut bytes, at, value);
    }
    bytes[36..38].copy_from_slice(&1u16.to_le_bytes());
    bytes[38..40].copy_from_slice(&1u16.to_le_bytes());
    bytes[44] = 1;
    put(&mut bytes, 136, 0);
    put(&mut bytes, 140, 0x80000100);
    bytes
}
#[test]
fn d3_borrows_and_relocates() {
    let bytes = d3_fixture();
    let mut tex = d3::Texture::<d3::V47>::read(&bytes).unwrap();
    assert_eq!(tex.header.sno_id, 123);
    assert_eq!((tex.width(), tex.height()), (1, 1));
    assert!(matches!(tex.mipmaps[0].data, Cow::Borrowed(_)));
    tex.frames.push(d3::Frame {
        uv: [0., 0., 1., 1.],
        name: [b'x'; 64],
    });
    let out = tex.write().unwrap();
    assert!(out.len() > bytes.len());
    let reread = d3::Texture::<d3::V47>::read(&out).unwrap();
    assert_eq!(reread.mipmaps[0].data.as_ref(), &[3, 2, 1, 4]);
    assert_eq!(reread.frames, tex.frames);
    assert_eq!(reread.into_owned().as_ref().mipmaps.len(), 1);
}
#[test]
fn d3_rejects_bad_revision_and_ranges() {
    let mut bytes = d3_fixture();
    put(&mut bytes, 4, 46);
    assert_eq!(
        d3::Texture::<d3::V47>::read(&bytes).unwrap_err(),
        Error::UnsupportedRevision(46)
    );
    put(&mut bytes, 4, 47);
    put(&mut bytes, 56, u32::MAX);
    assert!(d3::Texture::<d3::V47>::read(&bytes).is_err());
    let mut tex = d3::Texture::<d3::V47>::new(1, 1, 1, 0).unwrap();
    tex.mipmaps.push(d3::Mip {
        face_size: 4,
        data: Cow::Borrowed(&[0]),
    });
    assert!(tex.write().is_err());
}
#[test]
fn d3_unknown_format_is_preserved() {
    let mut bytes = d3_fixture();
    put(&mut bytes, 32, 0x12345678);
    let tex = d3::Texture::<d3::V47>::read(&bytes).unwrap();
    let out = tex.write().unwrap();
    assert_eq!(
        d3::Texture::<d3::V47>::read(&out).unwrap().format(),
        0x12345678
    );
}
#[test]
fn d4_metadata_is_independent_of_payload() {
    let bytes = d4_fixture();
    let tex = d4::Texture::<d4::HashF9cd83e7>::read(&bytes).unwrap();
    assert_eq!((tex.width(), tex.height(), tex.record.sno_id), (1, 1, 123));
    assert_eq!(tex.pixel_ranges[0].size(), 256);
    assert_eq!(tex.pixel_ranges[0].resolve(&[7; 256]).unwrap(), &[7; 256]);
    assert!(tex.pixel_ranges[0].resolve(&[0; 255]).is_err());
    let out = tex.write().unwrap();
    let reread = d4::Texture::<d4::HashF9cd83e7>::read(&out).unwrap();
    assert_eq!(reread.pixel_ranges, tex.pixel_ranges);
}
#[test]
fn d4_rejects_unestablished_hash_and_bad_arrays() {
    let mut bytes = d4_fixture();
    put(&mut bytes, 4, 0xf9cd83e6);
    assert_eq!(
        d4::Texture::<d4::HashF9cd83e7>::read(&bytes).unwrap_err(),
        Error::UnsupportedRevision(0xf9cd83e6)
    );
    put(&mut bytes, 4, 0xf9cd83e7);
    put(&mut bytes, 92, 7);
    assert!(d4::Texture::<d4::HashF9cd83e7>::read(&bytes).is_err());
}
#[cfg(feature = "tex-decode")]
#[test]
fn d3_bgra_and_d4_padded_rgba() {
    let bytes = d3_fixture();
    let tex = d3::Texture::<d3::V47>::read(&bytes).unwrap();
    assert_eq!(tex.decode_mip(0, 0).unwrap().as_raw(), &[1, 2, 3, 4]);
    let bytes = d4_fixture();
    let tex = d4::Texture::<d4::HashF9cd83e7>::read(&bytes).unwrap();
    let mut payload = [0; 256];
    payload[..4].copy_from_slice(&[1, 2, 3, 4]);
    assert_eq!(
        tex.decode_range(0, 0, &payload).unwrap().as_raw(),
        &[1, 2, 3, 4]
    );
}
#[cfg(all(feature = "tex-decode", feature = "tex-encode"))]
#[test]
fn d3_bc_fields_and_partial_blocks_roundtrip() {
    let image = image::RgbaImage::from_pixel(7, 5, image::Rgba([255, 0, 0, 255]));
    for format in [9, 10, 11, 12] {
        let mut tex = d3::Texture::<d3::V47>::new(7, 5, 1, format).unwrap();
        tex.mipmaps.push(tex.encode_mip(&image).unwrap());
        let bytes = tex.write().unwrap();
        let decoded = d3::Texture::<d3::V47>::read(&bytes)
            .unwrap()
            .decode_mip(0, 0)
            .unwrap();
        assert_eq!(decoded.dimensions(), (7, 5));
        assert!(
            decoded
                .pixels()
                .all(|p| p[0] > 245 && p[1] < 10 && p[2] < 10 && p[3] > 245),
            "format {format}: {:?}",
            decoded
                .pixels()
                .find(|p| p[0] <= 245 || p[1] >= 10 || p[2] >= 10 || p[3] <= 245)
        );
    }
}
#[cfg(feature = "tex-decode")]
#[test]
fn d4_bc4_bc5_and_bc7_are_distinguished_by_format() {
    // Constant BC4 endpoints: red=200; BC5 second channel green=100.
    for (format, block, expected) in [
        (41, vec![200, 200, 0, 0, 0, 0, 0, 0], [200, 0, 0, 255]),
        (
            42,
            vec![200, 200, 0, 0, 0, 0, 0, 0, 100, 100, 0, 0, 0, 0, 0, 0],
            [200, 100, 0, 255],
        ),
    ] {
        let mut tex = d4::Texture::<d4::HashF9cd83e7>::new(d4::Record::new(4, 4, format));
        tex.pixel_ranges.push(d4::PayloadRange {
            offset: 0,
            size_and_flags: 256,
        });
        let mut payload = vec![0; 256];
        payload[..block.len()].copy_from_slice(&block);
        let decoded = tex.decode_range(0, 0, &payload).unwrap();
        assert!(decoded.pixels().all(|p| p.0 == expected));
    }
    let mut tex = d4::Texture::<d4::HashF9cd83e7>::new(d4::Record::new(4, 4, 43));
    tex.pixel_ranges.push(d4::PayloadRange {
        offset: 0,
        size_and_flags: 256,
    });
    assert_eq!(
        tex.decode_range(0, 0, &[0; 256]).unwrap_err(),
        Error::UnsupportedPixelFormat(43)
    );
}
#[cfg(all(feature = "tex-decode", feature = "tex-encode"))]
#[test]
fn d4_native_row_padding_is_encoded_and_decoded() {
    let image = image::RgbaImage::from_pixel(7, 5, image::Rgba([255, 0, 0, 255]));
    for format in [0, 1, 9, 11, 12] {
        let mut tex = d4::Texture::<d4::HashF9cd83e7>::new(d4::Record::new(7, 5, format));
        let payload = tex.encode_mip(&image).unwrap();
        assert_eq!(payload.len() % 256, 0);
        tex.pixel_ranges.push(d4::PayloadRange {
            offset: 0,
            size_and_flags: payload.len() as u32,
        });
        let decoded = tex.decode_range(0, 0, &payload).unwrap();
        assert!(decoded
            .pixels()
            .all(|p| p[0] > 245 && p[1] < 10 && p[2] < 10 && p[3] > 245));
    }
}
#[cfg(feature = "tex-decode")]
#[test]
fn d4_cube_face_ranges_are_explicit_and_face_major() {
    let mut record = d4::Record::new(1, 1, 1);
    record.faces = 6;
    record.resource_type = 1;
    let mut tex = d4::Texture::<d4::HashF9cd83e7>::new(record);
    let mut payload = vec![0; 6 * 256];
    for face in 0..6 {
        let at = face * 256;
        payload[at..at + 4].copy_from_slice(&[face as u8, 20, 30, 255]);
        for level in 0..11 {
            tex.pixel_ranges.push(d4::PayloadRange {
                offset: at as u32,
                size_and_flags: if level == 0 { 256 } else { 0 },
            });
        }
    }
    for face in 0..6 {
        assert_eq!(
            tex.decode_range(face * 11, 0, &payload)
                .unwrap()
                .get_pixel(0, 0)
                .0,
            [face as u8, 20, 30, 255]
        );
    }
}
#[cfg(feature = "tex-decode")]
#[test]
fn d3_bc5_native_shuffle_uses_the_documented_bc3_field_planes() {
    // Native ATI2 follows the DXT5 shuffle, even though its second half is
    // another alpha block: its eight bytes are split into 4+4 planes.
    let mut payload = vec![0; 48];
    payload[16..20].copy_from_slice(&[10, 10, 30, 30]);
    payload[32..40].copy_from_slice(&[20, 20, 0, 0, 40, 40, 0, 0]);
    let mut tex = d3::Texture::<d3::V47>::new(8, 4, 1, 43).unwrap();
    tex.mipmaps.push(d3::Mip {
        face_size: 32,
        data: Cow::Owned(payload),
    });
    let decoded = tex.decode_mip(0, 0).unwrap();
    for y in 0..4 {
        for x in 0..8 {
            assert_eq!(
                decoded.get_pixel(x, y).0,
                if x < 4 {
                    [10, 20, 0, 255]
                } else {
                    [30, 40, 0, 255]
                }
            );
        }
    }
}

#[test]
fn d4_rejects_runtime_pointers_and_negative_metadata_descriptors() {
    for at in [64, 80, 96] {
        let mut bytes = d4_fixture();
        bytes[16 + at] = 1;
        assert!(d4::Texture::<d4::HashF9cd83e7>::read(&bytes).is_err());
        let mut bytes = d4_fixture();
        put(&mut bytes, 16 + at + 8, u32::MAX);
        assert!(d4::Texture::<d4::HashF9cd83e7>::read(&bytes).is_err());
        let mut bytes = d4_fixture();
        put(&mut bytes, 16 + at + 12, u32::MAX);
        assert!(d4::Texture::<d4::HashF9cd83e7>::read(&bytes).is_err());
    }
}
#[test]
fn d4_retains_only_explicit_reserved_bytes_and_rebuilds_pointer_slots() {
    let mut bytes = d4_fixture();
    bytes[16 + 31] = 17;
    bytes[16 + 112..16 + 120].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let mut texture = d4::Texture::<d4::HashF9cd83e7>::read(&bytes).unwrap();
    texture.record.reserved_31 = 18;
    let output = texture.write().unwrap();
    assert_eq!(output[16 + 31], 18);
    assert_eq!(&output[16 + 112..16 + 120], &[1, 2, 3, 4, 5, 6, 7, 8]);
    for at in [64, 80, 96] {
        assert_eq!(&output[16 + at..16 + at + 8], &[0; 8]);
    }
}

#[cfg(feature = "tex-encode")]
#[test]
fn d4_encoding_rejects_empty_images_before_pixel_codec_dispatch() {
    for format in [1, 9, 11, 12] {
        let texture = d4::Texture::<d4::HashF9cd83e7>::new(d4::Record::new(1, 1, format));
        for (width, height) in [(0, 1), (1, 0), (0, 0)] {
            assert!(texture
                .encode_mip(&image::RgbaImage::new(width, height))
                .is_err());
        }
    }
}
