use image::{Rgba, RgbaImage};
use wc3::blp::{Blp, BlpEncoder, BlpRef, BlpVersion, EncodeFormat, EncodeOptions, IndexedAlpha};

#[test]
fn image_encoder_writes_a_blp_file() {
    use image::{ExtendedColorType, ImageEncoder};

    let image = source();
    let mut bytes = Vec::new();
    let options = EncodeOptions {
        format: EncodeFormat::Bgra,
        mipmaps: false,
    };
    BlpEncoder::with_options(&mut bytes, options)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        )
        .unwrap();
    assert_eq!(BlpRef::read(&bytes).unwrap().decode_mip(0).unwrap(), image);

    let error = BlpEncoder::new(Vec::new())
        .write_image(&[0; 3], 1, 1, ExtendedColorType::Rgb8)
        .unwrap_err();
    assert!(matches!(error, image::ImageError::Unsupported(_)));
}

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
    for format in [
        EncodeFormat::Indexed {
            version: BlpVersion::Blp1,
            alpha: IndexedAlpha::Bit8,
        },
        EncodeFormat::Indexed {
            version: BlpVersion::Blp2,
            alpha: IndexedAlpha::Bit4,
        },
        EncodeFormat::Jpeg {
            version: BlpVersion::Blp1,
            alpha: true,
            quality: 90,
        },
        EncodeFormat::Jpeg {
            version: BlpVersion::Blp2,
            alpha: false,
            quality: 90,
        },
        EncodeFormat::Dxt1 { alpha: true },
        EncodeFormat::Dxt3,
        EncodeFormat::Dxt5,
        EncodeFormat::Bgra,
    ] {
        let options = EncodeOptions {
            format,
            ..Default::default()
        };
        let encoded = Blp::encode_image(&image, options).unwrap();
        let bytes = encoded.write().unwrap();
        let parsed = BlpRef::read(&bytes).unwrap();
        let expected = parsed.decode_mip(0).unwrap();
        let mut direct = vec![0; expected.as_raw().len()];
        parsed.decode_mip_into(0, &mut direct).unwrap();
        assert_eq!(direct, *expected.as_raw());
        let short_len = direct.len() - 1;
        assert!(parsed.decode_mip_into(0, &mut direct[..short_len]).is_err());
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
        format: EncodeFormat::Bgra,
        ..Default::default()
    };
    let bytes = Blp::encode_image(&image, options).unwrap().write().unwrap();
    assert_eq!(BlpRef::read(&bytes).unwrap().decode_mip(0).unwrap(), image);
}

#[test]
fn jpeg_preserves_component_order_and_alpha() {
    let image = RgbaImage::from_pixel(8, 8, Rgba([220, 40, 80, 160]));
    for version in [BlpVersion::Blp1, BlpVersion::Blp2] {
        let format = EncodeFormat::Jpeg {
            version,
            alpha: true,
            quality: 100,
        };
        let options = EncodeOptions {
            format,
            mipmaps: false,
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
    for (format, expected_size) in [
        (EncodeFormat::Dxt1 { alpha: true }, 16),
        (EncodeFormat::Dxt3, 32),
        (EncodeFormat::Dxt5, 32),
    ] {
        let options = EncodeOptions {
            format,
            mipmaps: false,
        };
        let blp = Blp::encode_image(&image, options).unwrap();
        let Blp::Blp2(ref container) = blp else {
            panic!("expected BLP2")
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
fn rejects_invalid_quality_and_mip_dimensions() {
    let image = source();
    let options = EncodeOptions {
        format: EncodeFormat::Jpeg {
            version: BlpVersion::Blp2,
            alpha: true,
            quality: 0,
        },
        ..Default::default()
    };
    assert!(Blp::encode_image(&image, options).is_err());
    let options = EncodeOptions {
        format: EncodeFormat::Bgra,
        ..Default::default()
    };
    assert!(Blp::encode_mipmaps(&[image.clone(), image], options).is_err());
}
