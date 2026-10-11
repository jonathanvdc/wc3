use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use wc3::blp::{Blp, BlpEncoder, BlpRef, EncodeFormat, EncodeOptions};

#[test]
fn encode_feature_writes_container_without_decode_feature() {
    let image = RgbaImage::from_pixel(2, 2, Rgba([12, 34, 56, 78]));
    let options = EncodeOptions {
        format: EncodeFormat::Bgra,
        mipmaps: false,
        ..Default::default()
    };
    let bytes = Blp::encode_image(&image, options).unwrap().write().unwrap();
    assert!(matches!(BlpRef::read(&bytes).unwrap(), BlpRef::Blp2(_)));
}

#[test]
fn image_encoder_is_available_without_decode_feature() {
    let image = RgbaImage::from_pixel(2, 2, Rgba([12, 34, 56, 78]));
    let mut bytes = Vec::new();
    BlpEncoder::new(&mut bytes)
        .write_image(image.as_raw(), 2, 2, ExtendedColorType::Rgba8)
        .unwrap();
    assert!(matches!(BlpRef::read(&bytes).unwrap(), BlpRef::Blp1(_)));
}

#[test]
fn blp0_encoding_returns_external_payloads_without_decode_feature() {
    use wc3::blp::{BlpVersion, IndexedAlpha};
    let image = image::RgbaImage::from_pixel(1, 1, image::Rgba([40, 80, 120, 160]));
    let blp = Blp::encode_image(
        &image,
        EncodeOptions {
            format: EncodeFormat::Indexed {
                version: BlpVersion::Blp0,
                alpha: IndexedAlpha::Bit8,
            },
            ..Default::default()
        },
    )
    .unwrap();
    let output = blp.write_files().unwrap();
    assert_eq!(&output.root[..4], b"BLP0");
    assert_eq!(output.mipmaps[0].as_ref().unwrap().len(), 2);
    assert!(output.mipmaps[1..].iter().all(Option::is_none));
}
