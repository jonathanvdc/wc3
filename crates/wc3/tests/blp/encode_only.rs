use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use wc3::blp::{Blp, BlpEncoder, BlpRef, EncodeFormat, EncodeOptions};

#[test]
fn encode_feature_writes_container_without_decode_feature() {
    let image = RgbaImage::from_pixel(2, 2, Rgba([12, 34, 56, 78]));
    let options = EncodeOptions {
        format: EncodeFormat::Bgra,
        mipmaps: false,
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
