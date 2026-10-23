use super::*;

#[test]
fn image_clones_share_immutable_storage_and_compare_by_pixels() {
    let original = pixel([12, 34, 56, 78]);
    let cloned = original.clone();
    assert!(Arc::ptr_eq(&original.pixels, &cloned.pixels));
    assert_eq!(original.pixels().as_ptr(), cloned.pixels().as_ptr());
    assert_eq!(original, cloned);
    assert_eq!(original, pixel([12, 34, 56, 78]));
}

#[test]
fn pixel_extraction_reuses_unique_storage_and_copies_shared_storage() {
    let original = pixel([12, 34, 56, 78]);
    let allocation = original.pixels().as_ptr();
    let mut extracted = original.clone().into_pixels();
    assert_eq!(extracted, [12, 34, 56, 78]);
    assert_ne!(allocation, extracted.as_ptr());
    extracted[0] = 99;
    assert_eq!(original.pixels(), [12, 34, 56, 78]);
    let unique = original.into_pixels();
    assert_eq!(allocation, unique.as_ptr());
    assert_eq!(unique, [12, 34, 56, 78]);
}

#[test]
fn bevy_image_extraction_keeps_dimensions_format_and_exact_bytes() {
    let original = RgbaImage::new(2, 1, vec![12, 34, 56, 78, 90, 123, 45, 67]).unwrap();
    let allocation = original.pixels().as_ptr();
    let shared = original.clone().into_image();
    assert_eq!(shared.texture_descriptor.size.width, 2);
    assert_eq!(shared.texture_descriptor.size.height, 1);
    assert_eq!(
        shared.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(shared.asset_usage, RenderAssetUsages::default());
    assert_eq!(shared.data.as_deref(), Some(original.pixels()));
    assert_ne!(shared.data.as_ref().unwrap().as_ptr(), allocation);
    let unique = original.into_image();
    assert_eq!(unique, shared);
    assert_eq!(unique.data.as_ref().unwrap().as_ptr(), allocation);
}

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
        blend: Default::default(),
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
        blend: Default::default(),
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
        blend: Default::default(),
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

#[test]
fn encoded_blend_modes_preserve_distinct_color_and_alpha_equations() {
    let source = pixel([180, 70, 210, 128]);
    let atlas = Atlas {
        width: 1,
        height: 1,
        clear: [30, 160, 90, 96],
    };
    for (blend, expected) in [
        (AtlasBlend::Replace, [180, 70, 210, 128]),
        (AtlasBlend::Alpha, [105, 115, 150, 112]),
        (AtlasBlend::AlphaStraight, [105, 115, 150, 176]),
        (AtlasBlend::Multiply, [28, 138, 87, 87]),
        (AtlasBlend::Overlay, [77, 124, 144, 112]),
        (AtlasBlend::Screen, [110, 173, 158, 112]),
    ] {
        let layer = AtlasLayer {
            image: &source,
            source: rect(1, 1),
            destination: rect(1, 1),
            opacity: 255,
            blend,
        };
        assert_eq!(
            compose_atlas(atlas, &[layer]).unwrap().pixels(),
            &expected,
            "{blend:?}"
        );
    }
}

#[test]
fn replace_keeps_hidden_rgb_and_zero_opacity_clears_alpha() {
    let source = pixel([180, 70, 210, 0]);
    for opacity in [0, 255] {
        let layer = AtlasLayer {
            image: &source,
            source: rect(1, 1),
            destination: rect(1, 1),
            opacity,
            blend: AtlasBlend::Replace,
        };
        assert_eq!(
            compose_atlas(
                Atlas {
                    width: 1,
                    height: 1,
                    clear: [255; 4]
                },
                &[layer]
            )
            .unwrap()
            .pixels(),
            &[180, 70, 210, 0]
        );
    }
}
