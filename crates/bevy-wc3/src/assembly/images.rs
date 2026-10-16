use super::{compose_atlas, AtlasLayer, PixelRect, RgbaImage, TextureReplacement};
use crate::{
    compile_model_with_options, CompileOptions, CompiledModel, DrawSelection, ModelError,
    TextureColorSpace, TextureImageKey, TextureRequest,
};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::{Assets, Handle, Image};
use bevy::render::render_resource::TextureFormat;
use std::collections::{BTreeMap, HashMap};
use wc3::model::{AddressMode, ImageResource, Model, Sampler, TextureId};
pub(super) struct ImageCompiler<'a, F> {
    pub(super) images: &'a mut Assets<Image>,
    pub(super) resolve: &'a mut F,
    pub(super) decoded: &'a mut HashMap<ImageResource<'static>, RgbaImage>,
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
                    })
                    .collect();
                compose_atlas(*atlas, &layers).map_err(|error| ModelError(error.to_string()))
            }
        }
    }
    pub(super) fn compile(
        &mut self,
        model: &Model<'_>,
        draws: DrawSelection,
        replacements: &BTreeMap<TextureId, TextureReplacement>,
    ) -> Result<CompiledModel, ModelError> {
        let mut prepared_replacements = HashMap::new();
        let mut handles: HashMap<TextureImageKey, Handle<Image>> = HashMap::new();
        let mut failure = None;
        let compiled = compile_model_with_options(
            model,
            CompileOptions {
                draws,
                ..Default::default()
            },
            |request: TextureRequest<'_, '_>| {
                if let Some(handle) = handles.get(&request.key()) {
                    return Some(handle.clone());
                }
                let pixels = match replacements.get(&request.id) {
                    Some(replacement) => {
                        if let Some(image) = prepared_replacements.get(&request.id) {
                            Ok(RgbaImage::clone(image))
                        } else {
                            self.replacement(replacement).inspect(|image| {
                                prepared_replacements.insert(request.id, image.clone());
                            })
                        }
                    }
                    None => self.decode(&request.resource.clone().into_owned()),
                };
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
                handles.insert(request.key(), handle.clone());
                Some(handle)
            },
        )?;
        if let Some(error) = failure {
            return Err(error);
        }
        Ok(compiled)
    }
}
fn sampler(sampler: Sampler) -> ImageSamplerDescriptor {
    let address = |mode| match mode {
        AddressMode::Repeat | AddressMode::Unknown(_) => ImageAddressMode::Repeat,
        AddressMode::Mirror => ImageAddressMode::MirrorRepeat,
        AddressMode::Clamp => ImageAddressMode::ClampToEdge,
    };
    ImageSamplerDescriptor {
        address_mode_u: address(sampler.u),
        address_mode_v: address(sampler.v),
        ..Default::default()
    }
}
