use crate::error::ModelError;
use bevy::asset::{io::Reader, AssetLoader, AssetPath, LoadContext, RenderAssetUsages};
use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::borrow::Cow;
use wc3::blp::{Blp, Blp2Content, DxtFormat};

/// Bevy loader for `.blp` images, registered by [`crate::Wc3BevyPlugin`].
///
/// Decodes mipmap level zero and honors the image loader's sRGB and sampler
/// settings. Linear BC5 uses two-channel GPU storage, preserving normal-map
/// interpretation; other images use RGBA. BLP0 reads its sibling `.b00` payload through the root's
/// asset source and tracks it as a loader dependency. Missing payloads fail
/// loading; other authored mip levels are not read or uploaded.
#[derive(Default, TypePath)]
pub struct BlpImageLoader;

impl AssetLoader for BlpImageLoader {
    type Asset = Image;
    type Settings = ImageLoaderSettings;
    type Error = ModelError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &ImageLoaderSettings,
        context: &mut LoadContext<'_>,
    ) -> Result<Image, ModelError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(ModelError::from_source)?;
        let mut blp = Blp::read(&bytes).map_err(ModelError::from_source)?;
        let external;
        if let Blp::Blp0(texture) = &mut blp {
            let path = AssetPath::from(context.path().path().with_extension("b00"))
                .with_source(context.path().source().clone_owned());
            external = context
                .read_asset_bytes(path.clone())
                .await
                .map_err(|error| ModelError::new(format!("BLP0 mipmap 0 at {path}: {error}")))?;
            texture.mipmaps[0] = Some(Cow::Borrowed(&external));
        }
        decode_image(&blp, settings)
    }

    fn extensions(&self) -> &[&str] {
        &["blp"]
    }
}

fn decode_image(blp: &Blp<'_>, settings: &ImageLoaderSettings) -> Result<Image, ModelError> {
    let two_channel = !settings.is_srgb
        && matches!(blp, Blp::Blp2(texture) if matches!(texture.content, Blp2Content::Dxt { format: DxtFormat::Bc5, .. }));
    let rgba = blp.decode_mip(0).map_err(ModelError::from_source)?;
    let extent = Extent3d {
        width: rgba.width(),
        height: rgba.height(),
        depth_or_array_layers: 1,
    };
    let mut pixels = rgba.into_raw();
    if two_channel {
        let count = pixels.len() / 4;
        for index in 0..count {
            pixels[index * 2] = pixels[index * 4];
            pixels[index * 2 + 1] = pixels[index * 4 + 1];
        }
        pixels.truncate(count * 2);
    }
    let mut image = Image::new(
        extent,
        TextureDimension::D2,
        pixels,
        if two_channel {
            TextureFormat::Rg8Unorm
        } else if settings.is_srgb {
            TextureFormat::Rgba8UnormSrgb
        } else {
            TextureFormat::Rgba8Unorm
        },
        RenderAssetUsages::default(),
    );
    image.sampler = settings.sampler.clone();
    Ok(image)
}

#[cfg(test)]
#[path = "blp_tests.rs"]
mod tests;
