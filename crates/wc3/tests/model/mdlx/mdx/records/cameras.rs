use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;

use wc3::model::formats::mdlx::scene::{Camera, CameraVariant};
use wc3::model::formats::mdlx::Document;

#[test]
fn camera_variants_round_trip() {
    let variants = [
        CameraVariant::Variant0,
        CameraVariant::Variant1([1; 12]),
        CameraVariant::Variant2([2; 12]),
        CameraVariant::Variant3,
        CameraVariant::Unknown(0x80),
    ];
    for variant in variants {
        let mut camera = Camera::<wc3::model::formats::mdlx::V1200>::new("Portrait").unwrap();
        camera.variant = variant;
        camera.target_position = [4.0, 5.0, 6.0];
        let mut model = Document::<wc3::model::formats::mdlx::V1200>::new();
        model.set_cameras([camera]);
        let bytes = model.encode_mdx().unwrap();
        let parsed = Document::<wc3::model::formats::mdlx::V1200>::decode_mdx(&bytes).unwrap();
        assert_eq!(parsed.cameras().next().unwrap().variant, variant);
        assert_eq!(
            parsed.cameras().next().unwrap().target_position,
            [4.0, 5.0, 6.0]
        );
        assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    }
}
