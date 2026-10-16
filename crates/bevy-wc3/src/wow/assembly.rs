use super::{
    compose_atlas, AtlasLayer, CharacterCatalog, CharacterDefinition, EquipmentBinding,
    GeosetSelection, ModelPart as EquipmentPart, PixelRect, RgbaImage, TextureReplacement,
};
use crate::mesh_attributes::{EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT};
use crate::{
    compile_model_with_options, CompileOptions, CompiledModel, DrawSelection, JointMap, ModelError,
    ModelPart, PartBinding, TextureColorSpace, TextureImageKey, TextureRequest,
};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::{Assets, Handle, Image, Mat4, Mesh};
use bevy::render::render_resource::TextureFormat;
use std::collections::{BTreeMap, HashMap, HashSet};
use wc3::model::{AddressMode, Attachment, Model, NodeFeature, NodeId, Resource, Sampler};

/// Source models resolved from one client build before character compilation.
/// Models keep their semantic resource and node identities; no native merge occurs.
pub type CharacterModels<'data> = HashMap<Resource<'static>, Model<'data>>;

// One collection model can serve several slots. Compile compatible selections
// together so shared-skeleton composition sees its scoped graph only once.
fn coalesce_parts(parts: &[EquipmentPart]) -> Result<Vec<EquipmentPart>, ModelError> {
    let mut output: Vec<EquipmentPart> = Vec::with_capacity(parts.len());
    let mut shared = HashMap::new();
    for part in parts {
        if !matches!(part.binding, EquipmentBinding::SharedSkeleton) {
            output.push(part.clone());
            continue;
        }
        if let Some(&index) = shared.get(&part.model) {
            let existing: &mut EquipmentPart = &mut output[index];
            let conflict = if existing.textures != part.textures {
                Some("replacement textures")
            } else if existing.clips != part.clips {
                Some("clip mappings")
            } else if existing.joints != part.joints {
                Some("joint mappings")
            } else if existing.retain_unmatched_joints != part.retain_unmatched_joints {
                Some("unmatched-joint policy")
            } else {
                None
            };
            if let Some(conflict) = conflict {
                return Err(ModelError(format!("shared-skeleton collection {:?} has incompatible {conflict} between {:?} and {:?}", part.model, existing.slot, part.slot)));
            }
            match (&mut existing.geosets, &part.geosets) {
                (Some(existing), Some(incoming)) => {
                    existing.visible.extend(incoming.visible.iter().copied())
                }
                (existing, None) => *existing = None,
                (None, Some(_)) => {}
            }
        } else {
            shared.insert(part.model.clone(), output.len());
            output.push(part.clone());
        }
    }
    Ok(output)
}

/// Compiles a resolved, fully equipped character into one shared animated rig.
///
/// The image callback supplies decoded sRGB RGBA pixels. Complete resources are
/// decoded once per call; role, sampler, and replacement interpretations receive
/// separate Bevy images. Database-selected geosets filter geometry before GPU
/// preparation. Shared parts use validated native bone matches or explicit maps.
/// Missing models, images, attachments and matching joints are errors; no partial
/// compiled character is returned. Image assets already inserted remain reusable.
pub fn compile_character(
    definition: &CharacterDefinition,
    models: &CharacterModels<'_>,
    images: &mut Assets<Image>,
    mut resolve_image: impl FnMut(&Resource<'static>) -> Result<RgbaImage, ModelError>,
) -> Result<CompiledModel, ModelError> {
    definition
        .validate()
        .map_err(|error| ModelError(error.to_string()))?;
    let parts = coalesce_parts(&definition.parts)?;
    // Validate each requested selection before unions involving authored/all
    // geometry can subsume it. Unknown requested geosets remain errors.
    for part in &definition.parts {
        if let Some(selection) = &part.geosets {
            let source = models
                .get(&part.model)
                .ok_or_else(|| ModelError(format!("missing equipment model {:?}", part.model)))?;
            selection
                .validate(&CharacterCatalog::inspect(source))
                .map_err(|error| ModelError(error.to_string()))?;
        }
    }
    let body = models
        .get(&definition.model)
        .ok_or_else(|| ModelError(format!("missing character model {:?}", definition.model)))?;
    let body_catalog = CharacterCatalog::inspect(body);
    let mut decoded = HashMap::new();
    let mut compiler = ImageCompiler {
        images,
        resolve: &mut resolve_image,
        decoded: &mut decoded,
    };
    let compiled_body = compiler.compile(body, Some(&definition.geosets), &definition.textures)?;
    let mut compiled_parts = Vec::with_capacity(parts.len());
    let mut bindings = Vec::with_capacity(parts.len());
    for part in &parts {
        let model = models
            .get(&part.model)
            .ok_or_else(|| ModelError(format!("missing equipment model {:?}", part.model)))?;
        let catalog = CharacterCatalog::inspect(model);
        let compiled_part = compiler.compile(model, part.geosets.as_ref(), &part.textures)?;
        let binding = match &part.binding {
            EquipmentBinding::Attachment(identity) => {
                let points: Vec<_> = body
                    .features()
                    .filter_map(|(id, feature)| {
                        let point = match &feature.payload {
                            NodeFeature::Attachment(Attachment::Point(point)) => Some(point),
                            NodeFeature::Attachment(Attachment::External(external)) => {
                                external.point.as_ref()
                            }
                            _ => None,
                        }?;
                        (point.identity == *identity).then_some((id, feature.node))
                    })
                    .collect();
                let (point, parent) = match points.as_slice() {
                    [point] => *point,
                    [] => {
                        return Err(ModelError(format!(
                            "character attachment {identity:?} is absent"
                        )))
                    }
                    _ => {
                        return Err(ModelError(format!(
                            "character attachment {identity:?} is ambiguous"
                        )))
                    }
                };
                // A rigid mount uses the pivot-local attachment frame; semantic
                // bone matrices themselves map authored model coordinates.
                let pivot = body
                    .node(parent)
                    .ok_or_else(|| ModelError("attachment node is absent".into()))?
                    .transform
                    .pivot;
                PartBinding::Attachment {
                    point,
                    transform: Mat4::from_translation(pivot.into()),
                }
            }
            EquipmentBinding::SharedSkeleton => {
                let mut joints: Vec<_> = part
                    .joints
                    .iter()
                    .map(|(source, target)| (*source, *target))
                    .collect();
                let explicit: HashSet<_> = part.joints.keys().copied().collect();
                let required = weighted_joints(&compiled_part);
                let mut required: Vec<_> = required
                    .into_iter()
                    .filter(|node| !explicit.contains(node))
                    .collect();
                required.sort();
                if part.retain_unmatched_joints {
                    for node in required {
                        match body_catalog.match_joints(&catalog, &[node]) {
                            Ok(matches) => joints.extend(matches),
                            Err(super::JointMatchError::Unmatched(_)) => {}
                            Err(error) => return Err(ModelError(error.to_string())),
                        }
                    }
                } else {
                    joints.extend(
                        body_catalog
                            .match_joints(&catalog, &required)
                            .map_err(|error| ModelError(error.to_string()))?,
                    );
                }
                // Bind other unambiguous native bones opportunistically so a
                // full-skeleton armor file does not instantiate duplicate rig
                // nodes merely because a particular geoset leaves them unused.
                let mut mapped_source: HashSet<_> =
                    joints.iter().map(|(source, _)| *source).collect();
                let mut mapped_host: HashSet<_> = joints.iter().map(|(_, host)| *host).collect();
                for bone in &catalog.bones {
                    if mapped_source.contains(&bone.node) {
                        continue;
                    }
                    if let Ok(matches) = body_catalog.match_joints(&catalog, &[bone.node]) {
                        let (source, host) = matches[0];
                        if mapped_host.insert(host) {
                            mapped_source.insert(source);
                            joints.push((source, host));
                        }
                    }
                }
                PartBinding::Skinned {
                    joints: JointMap::new(joints)?,
                    transform: Mat4::IDENTITY,
                }
            }
        };
        compiled_parts.push(compiled_part);
        bindings.push(binding);
    }
    compiled_body.compose(compiled_parts.iter().zip(bindings).zip(&parts).map(
        |((model, binding), part)| {
            ModelPart {
                model,
                binding,
                clips: part
                    .clips
                    .iter()
                    .map(|(host, child)| (*host, *child))
                    .collect(),
            }
        },
    ))
}

fn weighted_joints(model: &CompiledModel) -> HashSet<NodeId> {
    let mut joints = HashSet::new();
    for draw in &model.draws {
        let Some(skin) = draw.skin.map(|index| &model.skins[index]) else {
            continue;
        };
        for (indices, weights) in [
            (Mesh::ATTRIBUTE_JOINT_INDEX, Mesh::ATTRIBUTE_JOINT_WEIGHT),
            (EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT),
        ] {
            if let (
                Some(VertexAttributeValues::Uint16x4(indices)),
                Some(VertexAttributeValues::Float32x4(weights)),
            ) = (draw.mesh.attribute(indices), draw.mesh.attribute(weights))
            {
                for (indices, weights) in indices.iter().zip(weights) {
                    for (&joint, &weight) in indices.iter().zip(weights) {
                        if weight > 0. {
                            joints.insert(model.nodes[skin.joints[usize::from(joint)]].id);
                        }
                    }
                }
            }
        }
    }
    joints
}

struct ImageCompiler<'a, F> {
    images: &'a mut Assets<Image>,
    resolve: &'a mut F,
    decoded: &'a mut HashMap<Resource<'static>, RgbaImage>,
}
impl<F: FnMut(&Resource<'static>) -> Result<RgbaImage, ModelError>> ImageCompiler<'_, F> {
    fn decode(&mut self, resource: &Resource<'static>) -> Result<RgbaImage, ModelError> {
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
    fn compile(
        &mut self,
        model: &Model<'_>,
        geosets: Option<&GeosetSelection>,
        replacements: &BTreeMap<u32, TextureReplacement>,
    ) -> Result<CompiledModel, ModelError> {
        let draws = match geosets {
            Some(selection) => Some(
                selection
                    .draws(&CharacterCatalog::inspect(model))
                    .map_err(|error| ModelError(error.to_string()))?
                    .into_iter()
                    .map(|draw| draw.draw)
                    .collect(),
            ),
            None => None,
        };
        let mut prepared_replacements = HashMap::new();
        let mut handles: HashMap<TextureImageKey, Handle<Image>> = HashMap::new();
        let mut failure = None;
        let compiled = compile_model_with_options(
            model,
            CompileOptions {
                draws: DrawSelection {
                    draws,
                    ..Default::default()
                },
                ..Default::default()
            },
            |request: TextureRequest<'_, '_>| {
                if let Some(handle) = handles.get(&request.key()) {
                    return Some(handle.clone());
                }
                let pixels = match &request.texture.resource {
                    Resource::Replaceable(category) if replacements.contains_key(category) => {
                        if let Some(image) = prepared_replacements.get(category) {
                            Ok(RgbaImage::clone(image))
                        } else {
                            self.replacement(&replacements[category]).inspect(|image| {
                                prepared_replacements.insert(*category, image.clone());
                            })
                        }
                    }
                    resource => self.decode(&resource.clone().into_owned()),
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

#[cfg(test)]
#[path = "assembly_tests.rs"]
mod tests;
