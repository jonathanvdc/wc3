use super::{compose_atlas, AtlasLayer, PixelRect, RgbaImage, Source, TextureReplacement};
use crate::compiler::compile_with_geometry;
use crate::mesh_interner::MeshInterner;
use crate::textures::ImageKey;
use crate::{
    texture_requests, CompileOptions, CompiledModel, ModelError, TextureColorSpace, TextureRequest,
};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::{Assets, Handle, Image};
use bevy::render::render_resource::TextureFormat;
use std::collections::{HashMap, HashSet};
use wc3::model::{AddressMode, ImageResource, Model, Sampler};
pub(super) struct ImageCompiler<'a, F> {
    pub(super) images: &'a mut Assets<Image>,
    pub(super) resolve: &'a mut F,
    pub(super) decoded: &'a mut HashMap<ImageResource<'static>, RgbaImage>,
    pub(super) handles: Vec<(ImageKey<TextureReplacement>, Handle<Image>)>,
}
impl<F: FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>> ImageCompiler<'_, F> {
    fn decode(&mut self, resource: &ImageResource<'static>) -> Result<RgbaImage, ModelError> {
        if !self.decoded.contains_key(resource) {
            self.decoded
                .insert(resource.clone(), (self.resolve)(resource)?);
        }
        Ok(self.decoded[resource].clone())
    }
    fn replacement(&mut self, replacement: &TextureReplacement) -> Result<RgbaImage, ModelError> {
        match replacement {
            TextureReplacement::Image(resource) => self.decode(resource),
            TextureReplacement::Atlas { atlas, layers } => {
                let sources: Vec<_> = layers
                    .iter()
                    .map(|layer| self.decode(&layer.resource))
                    .collect::<Result<_, _>>()?;
                let layers: Vec<_> = layers
                    .iter()
                    .zip(&sources)
                    .map(|(layer, image)| AtlasLayer {
                        image,
                        source: layer.source.unwrap_or(PixelRect {
                            x: 0,
                            y: 0,
                            width: image.width(),
                            height: image.height(),
                        }),
                        destination: layer.destination,
                        opacity: layer.opacity,
                        blend: layer.blend,
                    })
                    .collect();
                compose_atlas(*atlas, &layers).map_err(|error| ModelError(error.to_string()))
            }
        }
    }
    pub(super) fn compile(
        &mut self,
        model: &Model<'_>,
        source: &Source,
        geometry: &mut MeshInterner,
    ) -> Result<CompiledModel, ModelError> {
        let scoped: HashSet<_> = texture_requests(model)
            .into_iter()
            .filter(|request| request.input.is_some())
            .map(|request| request.texture)
            .collect();
        let mut failure = None;
        let compiled = compile_with_geometry(
            model,
            CompileOptions {
                material: source.material,
                draws: source.draws.clone(),
                ..Default::default()
            },
            |request: TextureRequest<'_, '_>| {
                if request.input.is_none() && scoped.contains(&request.texture) {
                    return None;
                }
                let selection = request
                    .input
                    .and_then(|input| source.inputs.get(&input))
                    .or_else(|| source.images.get(&request.texture.resource))
                    .cloned()
                    .unwrap_or_else(|| {
                        TextureReplacement::Image(request.resource.clone().into_owned())
                    });
                let key = ImageKey {
                    source: selection.clone(),
                    color_space: request.color_space,
                    sampler: request.sampler,
                };
                if let Some((_, handle)) =
                    self.handles.iter().find(|(existing, _)| *existing == key)
                {
                    return Some(handle.clone());
                }
                let pixels = self.replacement(&selection);
                let pixels = match pixels {
                    Ok(pixels) => pixels,
                    Err(error) => {
                        if failure.is_none() {
                            failure = Some(error);
                        }
                        return None;
                    }
                };
                let mut image = pixels.into_image();
                if request.color_space == TextureColorSpace::Linear {
                    image.texture_descriptor.format = TextureFormat::Rgba8Unorm;
                }
                image.sampler = ImageSampler::Descriptor(sampler(request.sampler));
                let handle = self.images.add(image);
                self.handles.push((key, handle.clone()));
                Some(handle)
            },
            geometry,
        )?;
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(compiled)
    }
}
fn sampler(sampler: Sampler) -> ImageSamplerDescriptor {
    let address = |mode| match mode {
        AddressMode::Repeat => ImageAddressMode::Repeat,
        AddressMode::Mirror => ImageAddressMode::MirrorRepeat,
        AddressMode::Clamp => ImageAddressMode::ClampToEdge,
    };
    ImageSamplerDescriptor {
        address_mode_u: address(sampler.u),
        address_mode_v: address(sampler.v),
        ..Default::default()
    }
}
