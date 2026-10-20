use super::decode_image;
use bevy::image::ImageLoaderSettings;
use bevy::render::render_resource::TextureFormat;
use std::borrow::Cow;
use wc3::blp::{Blp, Blp2, Blp2Content, Blp2Header, DxtFormat};

fn texture(format: DxtFormat) -> Blp<'static> {
    static PALETTE: [u8; 1024] = [0; 1024];
    let mut block = vec![0; 16];
    block[0] = 17;
    block[8] = 29;
    Blp::Blp2(Blp2 {
        header: Blp2Header {
            width: 4,
            height: 4,
            alpha_bits: 0,
            mipmap_flags: 1,
        },
        content: Blp2Content::Dxt {
            format,
            palette_region: (&PALETTE).into(),
        },
        mipmaps: std::array::from_fn(|index| (index == 0).then(|| Cow::Owned(block.clone()))),
    })
}
#[test]
fn linear_bc5_keeps_two_channel_gpu_format_for_normal_map_interpretation() {
    let settings = ImageLoaderSettings {
        is_srgb: false,
        ..Default::default()
    };
    let image = decode_image(&texture(DxtFormat::Bc5), &settings).unwrap();
    assert_eq!(image.texture_descriptor.format, TextureFormat::Rg8Unorm);
    assert_eq!(image.data.unwrap(), [17, 29].repeat(16));
}
#[test]
fn bc5_explicit_color_transfer_keeps_rgba_srgb_and_classic_blocks_keep_rgba_linear() {
    let image = decode_image(&texture(DxtFormat::Bc5), &ImageLoaderSettings::default()).unwrap();
    assert_eq!(
        image.texture_descriptor.format,
        TextureFormat::Rgba8UnormSrgb
    );
    assert_eq!(image.data.unwrap(), [17, 29, 0, 255].repeat(16));
    let image = decode_image(
        &texture(DxtFormat::Dxt5),
        &ImageLoaderSettings {
            is_srgb: false,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(image.texture_descriptor.format, TextureFormat::Rgba8Unorm);
    assert_eq!(image.data.unwrap().len(), 64);
}
