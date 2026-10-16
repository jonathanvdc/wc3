use crate::error::ModelError;
use bevy::asset::{io::Reader, AssetLoader, AssetPath, LoadContext, RenderAssetUsages};
use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::borrow::Cow;
use wc3::blp::Blp;

/// Bevy loader for `.blp` images, registered by [`crate::Wc3BevyPlugin`].
///
/// Decodes mipmap level zero to RGBA and honors the image loader's sRGB and
/// sampler settings. BLP0 reads its sibling `.b00` payload through the root's
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
            .map_err(|error| ModelError(error.to_string()))?;
        let mut blp = Blp::read(&bytes).map_err(|error| ModelError(error.to_string()))?;
        let external;
        if let Blp::Blp0(texture) = &mut blp {
            let path = AssetPath::from(context.path().path().with_extension("b00"))
                .with_source(context.path().source().clone_owned());
            external = context
                .read_asset_bytes(path.clone())
                .await
                .map_err(|error| ModelError(format!("BLP0 mipmap 0 at {path}: {error}")))?;
            texture.mipmaps[0] = Some(Cow::Borrowed(&external));
        }
        let rgba = blp
            .decode_mip(0)
            .map_err(|error| ModelError(error.to_string()))?;
        let mut image = Image::new(
            Extent3d {
                width: rgba.width(),
                height: rgba.height(),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            rgba.into_raw(),
            if settings.is_srgb {
                TextureFormat::Rgba8UnormSrgb
            } else {
                TextureFormat::Rgba8Unorm
            },
            RenderAssetUsages::default(),
        );
        image.sampler = settings.sampler.clone();
        Ok(image)
    }

    fn extensions(&self) -> &[&str] {
        &["blp"]
    }
}
