use image::{Rgba, RgbaImage};
use wc3::blp::{Blp, BlpRef, EncodeFormat, EncodeOptions};

#[test]
fn encode_feature_writes_container_without_decode_feature() {
    let image = RgbaImage::from_pixel(2, 2, Rgba([12, 34, 56, 78]));
    let options = EncodeOptions {
        format: EncodeFormat::Blp2Bgra,
        mipmaps: false,
        ..Default::default()
    };
    let bytes = Blp::encode_image(&image, options).unwrap().write().unwrap();
    assert!(matches!(BlpRef::read(&bytes).unwrap(), BlpRef::Blp2(_)));
}
