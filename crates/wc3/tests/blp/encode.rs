use image::{Rgba, RgbaImage};
use wc3::blp::{Blp, BlpRef, DxtFormat, EncodeFormat, EncodeOptions};

fn source() -> RgbaImage {
    RgbaImage::from_fn(8, 8, |x, y| {
        Rgba([
            x as u8 * 23,
            y as u8 * 27,
            (x + y) as u8 * 11,
            if x < 4 { 255 } else { 0 },
        ])
    })
}

#[test]
fn all_encodings_write_read_and_decode_every_mip() {
    let image = source();
    for (format, alpha_bits) in [
        (EncodeFormat::Blp1Indexed, 8),
        (EncodeFormat::Blp2Indexed, 4),
        (EncodeFormat::Blp1Jpeg, 8),
        (EncodeFormat::Blp2Jpeg, 0),
        (EncodeFormat::Blp2Dxt(DxtFormat::Dxt1), 1),
        (EncodeFormat::Blp2Dxt(DxtFormat::Dxt3), 4),
        (EncodeFormat::Blp2Dxt(DxtFormat::Dxt5), 8),
        (EncodeFormat::Blp2Bgra, 8),
    ] {
        let options = EncodeOptions {
            format,
            alpha_bits,
            ..Default::default()
        };
        let encoded = Blp::encode_image(&image, options).unwrap();
        let bytes = encoded.write().unwrap();
        let parsed = BlpRef::read(&bytes).unwrap();
        for level in 0..4 {
            let decoded = parsed
                .decode_mip(level)
                .unwrap_or_else(|error| panic!("{format:?} mip {level}: {error}"));
            assert_eq!(decoded.dimensions(), (8 >> level, 8 >> level));
        }
    }
}

#[test]
fn bgra_round_trip_is_exact() {
    let image = source();
    let options = EncodeOptions {
        format: EncodeFormat::Blp2Bgra,
        ..Default::default()
    };
    let bytes = Blp::encode_image(&image, options).unwrap().write().unwrap();
    assert_eq!(BlpRef::read(&bytes).unwrap().decode_mip(0).unwrap(), image);
}

#[test]
fn jpeg_preserves_component_order_and_alpha() {
    let image = RgbaImage::from_pixel(8, 8, Rgba([220, 40, 80, 160]));
    for format in [EncodeFormat::Blp1Jpeg, EncodeFormat::Blp2Jpeg] {
        let options = EncodeOptions {
            format,
            alpha_bits: 8,
            mipmaps: false,
            jpeg_quality: 100,
        };
        let bytes = Blp::encode_image(&image, options).unwrap().write().unwrap();
        let pixel = BlpRef::read(&bytes)
            .unwrap()
            .decode_mip(0)
            .unwrap()
            .get_pixel(0, 0)
            .0;
        for (actual, expected) in pixel.into_iter().zip([220, 40, 80, 160]) {
            assert!(actual.abs_diff(expected) <= 5, "{format:?}: {pixel:?}");
        }
    }
}

#[test]
fn dxt_handles_partial_blocks_and_alpha() {
    let image = RgbaImage::from_fn(5, 3, |x, _| {
        Rgba([200, 30, 60, if x == 0 { 0 } else { 255 }])
    });
    for (format, alpha_bits, expected_size) in [
        (DxtFormat::Dxt1, 1, 16),
        (DxtFormat::Dxt3, 4, 32),
        (DxtFormat::Dxt5, 8, 32),
    ] {
        let options = EncodeOptions {
            format: EncodeFormat::Blp2Dxt(format),
            alpha_bits,
            mipmaps: false,
            ..Default::default()
        };
        let blp = Blp::encode_image(&image, options).unwrap();
        let Blp::Blp2(ref container) = blp else {
            panic!("expected BLP2");
        };
        assert_eq!(container.mipmaps[0].as_ref().unwrap().len(), expected_size);
        let decoded = BlpRef::read(&blp.write().unwrap())
            .unwrap()
            .decode_mip(0)
            .unwrap();
        assert_eq!(decoded.dimensions(), (5, 3));
        assert_eq!(decoded.get_pixel(0, 0)[3], 0);
        assert_eq!(decoded.get_pixel(4, 0)[3], 255);
    }
}

#[test]
fn rejects_invalid_alpha_and_mip_dimensions() {
    let image = source();
    let options = EncodeOptions {
        format: EncodeFormat::Blp2Dxt(DxtFormat::Dxt3),
        alpha_bits: 8,
        ..Default::default()
    };
    assert!(Blp::encode_image(&image, options).is_err());
    let options = EncodeOptions {
        format: EncodeFormat::Blp2Bgra,
        ..Default::default()
    };
    assert!(Blp::encode_mipmaps(&[image.clone(), image], options).is_err());
}
