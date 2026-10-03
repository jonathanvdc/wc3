//! Source-aware model and texture loading through Bevy's asset server.
use super::paths::{model_paths, texture_paths};
use bevy::asset::{
    io::{AssetReaderError, Reader},
    AssetLoader, AssetPath, LoadContext, ReadAssetBytesError,
};
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use std::path::Path;

use wc3::model::materials::{Texture, TextureFlags};

use crate::assets::model::{ModelError, Wc3Model};
use crate::assets::resources::Wc3ModelResources;

/// An MDX or MDL file with its resolved image and child model dependencies.
#[derive(Asset, TypePath)]
pub struct Wc3ModelAsset {
    pub(crate) source: Wc3Model,
    pub(crate) textures: ResolvedModelTextures,
    pub(crate) models: Wc3ModelResources,
}

#[derive(Clone, Default)]
pub(crate) struct ResolvedTexture {
    pub(crate) default: Option<Handle<Image>>,
    pub(crate) replaceable_id: u32,
    pub(crate) bitmap_id: Option<usize>,
}

#[derive(Clone, Default)]
pub(crate) struct ResolvedModelTextures {
    pub(crate) bitmaps: Vec<ResolvedTexture>,
    pub(crate) particles: Vec<ResolvedTexture>,
}

pub(crate) fn resolve_texture(
    texture: &Texture,
    resolve: &mut impl FnMut(&str) -> Option<Handle<Image>>,
) -> ResolvedTexture {
    let path = texture.path.text();
    ResolvedTexture {
        default: if path.is_empty() {
            None
        } else {
            resolve(&path)
        },
        replaceable_id: texture.replaceable_id,
        bitmap_id: None,
    }
}

async fn load_texture(
    context: &mut LoadContext<'_>,
    model_path: &AssetPath<'_>,
    texture: &Texture,
) -> Result<ResolvedTexture, ModelError> {
    let path = texture.path.text();
    let mut resolved = ResolvedTexture {
        replaceable_id: texture.replaceable_id,
        ..default()
    };
    if !path.is_empty() {
        let candidates = texture_paths(model_path.path(), &path);
        for candidate in &candidates {
            let candidate = AssetPath::from(Path::new(candidate).to_owned())
                .with_source(model_path.source().clone_owned());
            if candidate_exists(context, &candidate).await? {
                debug!(
                    "Resolved WC3 texture {path} for {} to {candidate}",
                    model_path
                );
                let sampler = texture_sampler(texture.flags);
                resolved.default = Some(
                    context
                        .load_builder()
                        .with_settings::<ImageLoaderSettings>(move |settings| {
                            settings.sampler = sampler.clone()
                        })
                        .load(candidate.clone()),
                );
                break;
            }
        }
        if resolved.default.is_none() {
            warn!(
                "Could not resolve WC3 texture {path} for {}; tried {candidates:?}",
                model_path
            );
        }
    }
    Ok(resolved)
}

impl Wc3ModelAsset {
    /// Resolved attachment and Classic particle model dependencies.
    pub fn model_resources(&self) -> &Wc3ModelResources {
        &self.models
    }

    /// The decoded and normalized source model.
    pub fn source(&self) -> &Wc3Model {
        &self.source
    }
}

#[derive(Default, TypePath)]
pub struct Wc3ModelLoader;

impl AssetLoader for Wc3ModelLoader {
    type Asset = Wc3ModelAsset;
    type Settings = ();
    type Error = ModelError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        context: &mut LoadContext<'_>,
    ) -> Result<Wc3ModelAsset, ModelError> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|error| ModelError(error.to_string()))?;
        let source = Wc3Model::decode(&bytes)?;
        let model_path = context.path().clone_owned();
        let mut textures = ResolvedModelTextures::default();
        for texture in source.model.textures() {
            let binding = load_texture(context, &model_path, &texture).await?;
            textures.bitmaps.push(binding);
        }
        for emitter in source.model.particle_emitters2() {
            textures.particles.push(ResolvedTexture {
                replaceable_id: emitter.replaceable_id,
                bitmap_id: (emitter.replaceable_id == 0).then_some(emitter.texture_id as usize),
                ..default()
            });
        }
        let mut attachments = Vec::new();
        for attachment in source.model.attachments() {
            attachments.push(load_model(context, &model_path, &attachment.path.text()).await?);
        }
        let mut particles = Vec::new();
        for emitter in source.model.particle_emitters() {
            particles.push(if emitter.flags().emitter_uses_mdl() {
                load_model(context, &model_path, &emitter.path.text()).await?
            } else {
                None
            });
        }
        let models = Wc3ModelResources::from_handles(attachments, particles);
        Ok(Wc3ModelAsset {
            source,
            textures,
            models,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["mdx", "mdl"]
    }
}

fn texture_sampler(flags: TextureFlags) -> ImageSampler {
    let mut descriptor = ImageSamplerDescriptor::linear();
    descriptor.address_mode_u = if flags.wrap_width() {
        ImageAddressMode::Repeat
    } else {
        ImageAddressMode::ClampToEdge
    };
    descriptor.address_mode_v = if flags.wrap_height() {
        ImageAddressMode::Repeat
    } else {
        ImageAddressMode::ClampToEdge
    };
    ImageSampler::Descriptor(descriptor)
}

async fn load_model(
    context: &mut LoadContext<'_>,
    model_path: &AssetPath<'_>,
    name: &str,
) -> Result<Option<Handle<Wc3ModelAsset>>, ModelError> {
    if name.is_empty() {
        return Ok(None);
    }
    for candidate in model_paths(model_path.path(), name) {
        let candidate = AssetPath::from(Path::new(&candidate).to_owned())
            .with_source(model_path.source().clone_owned());
        if candidate_exists(context, &candidate).await? {
            return Ok(Some(context.load(candidate)));
        }
    }
    warn!("Could not resolve WC3 model {name} for {}", model_path);
    Ok(None)
}

async fn candidate_exists(
    context: &mut LoadContext<'_>,
    path: &AssetPath<'static>,
) -> Result<bool, ModelError> {
    match context.read_asset_bytes(path.clone()).await {
        Ok(_) => Ok(true),
        Err(ReadAssetBytesError::AssetReaderError(AssetReaderError::NotFound(_))) => Ok(false),
        Err(error) => Err(ModelError(format!("Could not read {path}: {error}"))),
    }
}

#[cfg(test)]
#[path = "loader_tests.rs"]
mod tests;
