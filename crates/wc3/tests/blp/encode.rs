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
        ..Default::default()
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
            ..Default::default()
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
            ..Default::default()
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

#[test]
fn mipmap_filter_changes_generated_levels_but_not_supplied_levels() {
    use wc3::blp::MipmapFilter;

    let image = source();
    let options = EncodeOptions {
        format: EncodeFormat::Bgra,
        mipmap_filter: MipmapFilter::Nearest,
        ..Default::default()
    };
    let nearest = Blp::encode_image(&image, options).unwrap();
    let triangle = Blp::encode_image(
        &image,
        EncodeOptions {
            mipmap_filter: MipmapFilter::Triangle,
            ..options
        },
    )
    .unwrap();
    assert_eq!(nearest.decode_mip(0).unwrap(), image);
    assert_ne!(
        nearest.decode_mip(1).unwrap(),
        triangle.decode_mip(1).unwrap()
    );
    assert_eq!(
        nearest.decode_mip(1).unwrap(),
        image::imageops::resize(&image, 4, 4, MipmapFilter::Nearest)
    );
    let supplied: Vec<_> = (0..4)
        .map(|level| nearest.decode_mip(level).unwrap())
        .collect();
    assert_eq!(
        Blp::encode_mipmaps(&supplied, options)
            .unwrap()
            .write()
            .unwrap(),
        Blp::encode_mipmaps(
            &supplied,
            EncodeOptions {
                mipmap_filter: MipmapFilter::Lanczos3,
                ..options
            }
        )
        .unwrap()
        .write()
        .unwrap(),
    );
}

#[test]
fn dxt_algorithms_produce_decodable_blocks_and_affect_compression() {
    use wc3::blp::DxtAlgorithm;

    let image = source();
    for format in [
        EncodeFormat::Dxt1 { alpha: true },
        EncodeFormat::Dxt3,
        EncodeFormat::Dxt5,
    ] {
        let mut outputs = Vec::new();
        for dxt_algorithm in [
            DxtAlgorithm::RangeFit,
            DxtAlgorithm::ClusterFit,
            DxtAlgorithm::IterativeClusterFit,
        ] {
            let encoded = Blp::encode_image(
                &image,
                EncodeOptions {
                    format,
                    mipmaps: false,
                    dxt_algorithm,
                    ..Default::default()
                },
            )
            .unwrap();
            let decoded = encoded.decode_mip(0).unwrap();
            assert_eq!(decoded.dimensions(), image.dimensions());
            for (actual, expected) in decoded.pixels().zip(image.pixels()) {
                assert_eq!(actual[3], expected[3]);
            }
            outputs.push(encoded.write().unwrap());
        }
        assert_ne!(outputs[0], outputs[1], "{format:?}");
    }
}

#[test]
fn blp0_encodings_decode_from_external_payloads() {
    use image::DynamicImage;
    use wc3::blp::{BlpDecoder, DecodeError};

    let image = source();
    for format in [
        EncodeFormat::Indexed {
            version: BlpVersion::Blp0,
            alpha: IndexedAlpha::Opaque,
        },
        EncodeFormat::Indexed {
            version: BlpVersion::Blp0,
            alpha: IndexedAlpha::Bit1,
        },
        EncodeFormat::Indexed {
            version: BlpVersion::Blp0,
            alpha: IndexedAlpha::Bit4,
        },
        EncodeFormat::Indexed {
            version: BlpVersion::Blp0,
            alpha: IndexedAlpha::Bit8,
        },
        EncodeFormat::Jpeg {
            version: BlpVersion::Blp0,
            alpha: true,
            quality: 100,
        },
        EncodeFormat::Jpeg {
            version: BlpVersion::Blp0,
            alpha: false,
            quality: 100,
        },
    ] {
        let encoded = Blp::encode_image(
            &image,
            EncodeOptions {
                format,
                ..Default::default()
            },
        )
        .unwrap();
        let output = encoded.write_files().unwrap();
        let mut parsed = BlpRef::read(&output.root).unwrap();
        assert!(matches!(
            parsed.decode_mip(0),
            Err(DecodeError::MissingMipmap { level: 0 })
        ));
        let BlpRef::Blp0(value) = &mut parsed else {
            panic!("expected BLP0")
        };
        value.mipmaps = output.mipmaps.each_ref().map(|mip| mip.as_deref());
        for level in 0..4 {
            let decoded = parsed.decode_mip(level).unwrap();
            assert_eq!(decoded, encoded.decode_mip(level).unwrap());
            let via_image =
                DynamicImage::from_decoder(BlpDecoder::from_blp(parsed.clone(), level).unwrap())
                    .unwrap()
                    .into_rgba8();
            assert_eq!(decoded, via_image);
        }
        assert_eq!(parsed.write_files().unwrap(), output);
    }
}

#[test]
fn blp0_image_encoder_rejects_single_file_output() {
    use image::{ExtendedColorType, ImageEncoder};
    let image = source();
    let mut output = Vec::new();
    let options = EncodeOptions {
        format: EncodeFormat::Jpeg {
            version: BlpVersion::Blp0,
            alpha: true,
            quality: 90,
        },
        ..Default::default()
    };
    assert!(BlpEncoder::with_options(&mut output, options)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8
        )
        .is_err());
    assert!(output.is_empty());
}

#[test]
fn blp0_shared_jpeg_prefix_and_malformed_external_data() {
    use wc3::blp::{Blp0Content, DecodeError};
    let image = RgbaImage::from_pixel(8, 2, Rgba([220, 40, 80, 160]));
    let mut encoded = Blp::encode_image(
        &image,
        EncodeOptions {
            format: EncodeFormat::Jpeg {
                version: BlpVersion::Blp0,
                alpha: true,
                quality: 100,
            },
            ..Default::default()
        },
    )
    .unwrap();
    let Blp::Blp0(texture) = &mut encoded else {
        panic!("expected BLP0")
    };
    // Move the common SOI marker into root metadata, leaving mipmap JPEG chunks.
    for mip in texture.mipmaps.iter_mut().flatten() {
        assert_eq!(mip.drain(..2).collect::<Vec<_>>(), [255, 216]);
    }
    texture.content = Blp0Content::Jpeg {
        shared_header: vec![255, 216],
    };
    let output = encoded.write_files().unwrap();
    let mut parsed = BlpRef::read(&output.root).unwrap();
    let BlpRef::Blp0(texture) = &mut parsed else {
        panic!("expected BLP0")
    };
    texture.mipmaps = output.mipmaps.each_ref().map(|mip| mip.as_deref());
    for (level, dimensions) in [(0, (8, 2)), (1, (4, 1)), (2, (2, 1)), (3, (1, 1))] {
        let decoded = parsed.decode_mip(level).unwrap();
        assert_eq!(decoded.dimensions(), dimensions);
        for (actual, expected) in decoded
            .get_pixel(0, 0)
            .0
            .into_iter()
            .zip([220, 40, 80, 160])
        {
            assert!(actual.abs_diff(expected) <= 5);
        }
    }
    let BlpRef::Blp0(texture) = &mut parsed else {
        unreachable!()
    };
    texture.mipmaps[0] = Some(&[0]);
    assert!(matches!(parsed.decode_mip(0), Err(DecodeError::Jpeg(_))));
}

#[test]
fn blp0_single_file_hook_reports_missing_external_mipmap() {
    use image::ImageReader;
    use std::io::Cursor;
    use wc3::blp::{register_decoding_hook, BlpDecoder};
    let encoded = Blp::encode_image(
        &source(),
        EncodeOptions {
            format: EncodeFormat::Indexed {
                version: BlpVersion::Blp0,
                alpha: IndexedAlpha::Bit8,
            },
            ..Default::default()
        },
    )
    .unwrap()
    .write_files()
    .unwrap();
    assert!(BlpDecoder::new(&encoded.root)
        .err()
        .unwrap()
        .to_string()
        .contains("missing BLP mipmap 0"));
    register_decoding_hook();
    let error = ImageReader::new(Cursor::new(encoded.root))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap_err();
    assert!(error.to_string().contains("missing BLP mipmap 0"));
}
