//! File-backed model and texture loading through Bevy's asset server.
use bevy::asset::RenderAssetUsages;
use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::path::{Component, Path};

use wc3::blp::BlpRef;
use wc3::model::materials::Texture;
use wc3::model::materials::TextureFlags;

use crate::model::{ModelError, Wc3Model};
use crate::model_resources::Wc3ModelResources;

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
    model_path: &Path,
    texture: &Texture,
) -> ResolvedTexture {
    let path = texture.path.text();
    let mut resolved = ResolvedTexture {
        replaceable_id: texture.replaceable_id,
        ..default()
    };
    if !path.is_empty() {
        for candidate in texture_paths(model_path, &path) {
            if context.read_asset_bytes(candidate.clone()).await.is_ok() {
                let sampler = texture_sampler(texture.flags);
                resolved.default = Some(
                    context
                        .load_builder()
                        .with_settings::<ImageLoaderSettings>(move |settings| {
                            settings.sampler = sampler.clone()
                        })
                        .load(candidate),
                );
                break;
            }
        }
    }
    resolved
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
        let model_path = context.path().path().to_owned();
        let mut textures = ResolvedModelTextures::default();
        for texture in source.model.textures() {
            let binding = load_texture(context, &model_path, &texture).await;
            if binding.default.is_none() && !texture.path.text().is_empty() {
                warn!(
                    "Could not resolve WC3 texture {} for {}",
                    texture.path.text(),
                    model_path.display()
                );
            }
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
            attachments.push(load_model(context, &model_path, &attachment.path.text()).await);
        }
        let mut particles = Vec::new();
        for emitter in source.model.particle_emitters() {
            particles.push(if emitter.flags().emitter_uses_mdl() {
                load_model(context, &model_path, &emitter.path.text()).await
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

/// Resolve beside the model first, then from the Bevy asset root.
fn texture_paths(model_path: &Path, name: &str) -> Vec<String> {
    let normalized = name.replace('\\', "/");
    if normalized.is_empty() {
        return Vec::new();
    }
    let path = Path::new(&normalized);
    if path.is_absolute()
        || path
            .components()
            .any(|part| matches!(part, Component::ParentDir))
    {
        return Vec::new();
    }
    let local = model_path.parent().unwrap_or(Path::new("")).join(path);
    let local = local.to_string_lossy().replace('\\', "/");
    if local == normalized {
        vec![normalized]
    } else {
        vec![local, normalized]
    }
}

/// Model lookup uses the same locations as textures. Try a real MDL before
/// falling back to the MDX commonly shipped for a Warcraft `.mdl` reference.
fn model_paths(model_path: &Path, name: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    for path in texture_paths(model_path, name) {
        let extension = Path::new(&path)
            .extension()
            .and_then(|value| value.to_str());
        if extension.is_some_and(|value| value.eq_ignore_ascii_case("mdl")) {
            let mdx = Path::new(&path)
                .with_extension("mdx")
                .to_string_lossy()
                .into_owned();
            candidates.push(path);
            candidates.push(mdx);
        } else if extension.is_some_and(|value| value.eq_ignore_ascii_case("mdx")) {
            candidates.push(path);
        }
    }
    candidates
}

async fn load_model(
    context: &mut LoadContext<'_>,
    model_path: &Path,
    name: &str,
) -> Option<Handle<Wc3ModelAsset>> {
    if name.is_empty() {
        return None;
    }
    for candidate in model_paths(model_path, name) {
        if context.read_asset_bytes(candidate.clone()).await.is_ok() {
            return Some(context.load(candidate));
        }
    }
    warn!(
        "Could not resolve WC3 model {name} for {}",
        model_path.display()
    );
    None
}

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

#[cfg(test)]
#[path = "asset_tests.rs"]
mod tests;
