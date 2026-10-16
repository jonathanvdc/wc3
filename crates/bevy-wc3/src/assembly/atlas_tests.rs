use super::*;
fn pixel(color: [u8; 4]) -> RgbaImage {
    RgbaImage::new(1, 1, color.to_vec()).unwrap()
}
fn rect(width: u32, height: u32) -> PixelRect {
    PixelRect {
        x: 0,
        y: 0,
        width,
        height,
    }
}
#[test]
fn source_over_accounts_for_both_alphas_and_layer_order() {
    let blue = pixel([0, 0, 255, 128]);
    let red = pixel([255, 0, 0, 128]);
    let layer = |image| AtlasLayer {
        image,
        source: rect(1, 1),
        destination: rect(1, 1),
        opacity: 255,
    };
    let atlas = Atlas {
        width: 1,
        height: 1,
        clear: [0; 4],
    };
    assert_eq!(
        compose_atlas(atlas, &[layer(&blue), layer(&red)])
            .unwrap()
            .pixels(),
        &[170, 0, 85, 192]
    );
    assert_eq!(
        compose_atlas(atlas, &[layer(&red), layer(&blue)])
            .unwrap()
            .pixels(),
        &[85, 0, 170, 192]
    );
}
#[test]
fn crop_scale_and_opacity_preserve_explicit_destination() {
    let image = RgbaImage::new(2, 1, vec![255, 0, 0, 255, 0, 255, 0, 255]).unwrap();
    let layer = AtlasLayer {
        image: &image,
        source: PixelRect {
            x: 1,
            y: 0,
            width: 1,
            height: 1,
        },
        destination: PixelRect {
            x: 1,
            y: 0,
            width: 2,
            height: 2,
        },
        opacity: 128,
    };
    let result = compose_atlas(
        Atlas {
            width: 3,
            height: 2,
            clear: [0, 0, 255, 255],
        },
        &[layer],
    )
    .unwrap();
    assert_eq!(
        result.pixels(),
        &[
            0, 0, 255, 255, 0, 128, 127, 255, 0, 128, 127, 255, 0, 0, 255, 255, 0, 128, 127, 255,
            0, 128, 127, 255
        ]
    );
}
#[test]
fn invalid_storage_and_overflowing_rectangles_are_rejected() {
    assert_eq!(RgbaImage::new(1, 1, vec![]), Err(AtlasError::PixelCount));
    assert_eq!(RgbaImage::new(0, 1, vec![]), Err(AtlasError::Dimensions));
    let image = pixel([0; 4]);
    let layer = AtlasLayer {
        image: &image,
        source: rect(1, 1),
        destination: PixelRect {
            x: u32::MAX,
            y: 0,
            width: 2,
            height: 1,
        },
        opacity: 255,
    };
    assert_eq!(
        compose_atlas(
            Atlas {
                width: 1,
                height: 1,
                clear: [0; 4]
            },
            &[layer]
        ),
        Err(AtlasError::Rectangle)
    );
}
