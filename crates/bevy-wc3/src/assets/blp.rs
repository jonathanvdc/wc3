use crate::assets::model::ModelError;
use bevy::asset::{io::Reader, AssetLoader, LoadContext, RenderAssetUsages};
use bevy::image::ImageLoaderSettings;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use wc3::blp::BlpRef;

/// Bevy loader for `.blp` images, registered by [`crate::Wc3BevyPlugin`].
///
/// Decodes the first mip level to RGBA and honors the image loader's sRGB and
/// sampler settings. Authored mip levels are not uploaded.
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
        _context: &mut LoadContext<'_>,
    ) -> Result<Image, ModelError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|error| ModelError(error.to_string()))?;
        let blp = BlpRef::read(&bytes).map_err(|error| ModelError(error.to_string()))?;
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
