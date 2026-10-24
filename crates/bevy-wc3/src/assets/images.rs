//! Per-load image sharing across model and external programme inputs.
use crate::error::{ImageLoadError, ModelError, PreparationContext, PreparationOperation};
use crate::textures::{ImageKey, TextureColorSpace};
use bevy::asset::{
    io::{AssetReaderError, SliceReader},
    AssetPath, LoadContext, ReadAssetBytesError,
};
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::{Handle, Image};
use std::collections::{HashMap, HashSet};
use wc3::model::{AddressMode, Sampler};

#[derive(Default)]
pub(super) struct ImageLoadCache {
    variants: HashMap<ImageKey<AssetPath<'static>>, Handle<Image>>,
    encoded: HashMap<AssetPath<'static>, Vec<u8>>,
    missing: HashSet<AssetPath<'static>>,
}
impl ImageLoadCache {
    pub(super) async fn resolve(
        &mut self,
        context: &mut LoadContext<'_>,
        candidates: Vec<AssetPath<'static>>,
        color_space: TextureColorSpace,
        sampler: Sampler,
    ) -> Result<Option<Handle<Image>>, ModelError> {
        for path in candidates {
            let key = ImageKey {
                source: path.clone(),
                color_space,
                sampler,
            };
            if let Some(handle) = self.variants.get(&key) {
                return Ok(Some(handle.clone()));
            }
            if self.missing.contains(&path) {
                continue;
            }
            if !self.encoded.contains_key(&path) {
                match context.read_asset_bytes(path.clone()).await {
                    Ok(bytes) => {
                        self.encoded.insert(path.clone(), bytes);
                    }
                    Err(ReadAssetBytesError::AssetReaderError(AssetReaderError::NotFound(_))) => {
                        self.missing.insert(path);
                        continue;
                    }
                    Err(error) => {
                        return Err(ModelError::from_source(error)
                            .with_context(PreparationContext::Operation(
                                PreparationOperation::ImageRead,
                            ))
                            .with_context(PreparationContext::Asset(path)));
                    }
                }
            }
            let mut descriptor = ImageSamplerDescriptor::linear();
            descriptor.address_mode_u = address(sampler.u);
            descriptor.address_mode_v = address(sampler.v);
            let mut reader = SliceReader::new(&self.encoded[&path]);
            let loaded = context
                .load_builder()
                .with_settings::<ImageLoaderSettings>(move |settings| {
                    settings.sampler = ImageSampler::Descriptor(descriptor.clone());
                    settings.is_srgb = color_space.is_srgb();
                })
                .load_untyped_value_from_reader(path.clone(), &mut reader)
                .await
                .map_err(|error| {
                    ModelError::from_source(ImageLoadError { load: error })
                        .with_context(PreparationContext::Operation(
                            PreparationOperation::ImageDecode,
                        ))
                        .with_context(PreparationContext::Asset(path.clone()))
                })?;
            let image = loaded.take::<Image>().ok_or_else(|| {
                ModelError::new("resource resolved to a non-image asset")
                    .with_context(PreparationContext::Operation(
                        PreparationOperation::ImageDecode,
                    ))
                    .with_context(PreparationContext::Asset(path.clone()))
            })?;
            let handle = context.add_labeled_asset(format!("Image{}", self.variants.len()), image);
            self.variants.insert(key, handle.clone());
            return Ok(Some(handle));
        }
        Ok(None)
    }
}
fn address(mode: AddressMode) -> ImageAddressMode {
    match mode {
        AddressMode::Repeat => ImageAddressMode::Repeat,
        AddressMode::Mirror => ImageAddressMode::MirrorRepeat,
        _ => ImageAddressMode::ClampToEdge,
    }
}
