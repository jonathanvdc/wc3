use super::{
    CharacterCatalog, CharacterDefinition, EquipmentBinding, GeosetSelection, JointMatchError,
};
use crate::assembly::{
    compile_with_bindings, Models, PendingAssembly, PendingPart, RgbaImage, Source,
    TextureReplacement,
};
use crate::mesh_attributes::{EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT};
use crate::{CompiledModel, DrawSelection, JointMap, ModelError, PartBinding};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::{Assets, Image, Mat4, Mesh};
use std::collections::{BTreeMap, HashSet};
use wc3::model::{ImageResource, Model, NodeFeature, NodeId, ResourceLocator};
use wc3::model::{ModelMount, MountTarget, Transform as ModelTransform};

#[derive(Clone, PartialEq, Eq)]
struct NativeBinding {
    binding: EquipmentBinding,
    joints: BTreeMap<NodeId, NodeId>,
    retain_unmatched_joints: bool,
}

/// Compiles a resolved WoW appearance through the generic model assembly pipeline.
/// Native geoset IDs and replacement categories are translated to semantic IDs;
/// attachment roles and native bone identities provide the equipment bindings.
pub fn compile_character(
    definition: &CharacterDefinition,
    models: &Models<'_>,
    images: &mut Assets<Image>,
    resolve_image: impl FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>,
) -> Result<CompiledModel, ModelError> {
    definition
        .validate()
        .map_err(|error| ModelError(error.to_string()))?;
    let assembly = PendingAssembly {
        root: source(
            &definition.model,
            Some(&definition.geosets),
            &definition.textures,
            models,
        )?,
        parts: definition
            .parts
            .iter()
            .map(|part| {
                Ok(PendingPart {
                    source: source(&part.model, part.geosets.as_ref(), &part.textures, models)?,
                    binding: NativeBinding {
                        binding: part.binding.clone(),
                        joints: part.joints.clone(),
                        retain_unmatched_joints: part.retain_unmatched_joints,
                    },
                    clips: part.clips.clone(),
                })
            })
            .collect::<Result<Vec<_>, ModelError>>()?,
    };
    let body_catalog = CharacterCatalog::inspect(&models[&definition.model]);
    compile_with_bindings(
        &assembly,
        models,
        images,
        resolve_image,
        |binding| matches!(binding.binding, EquipmentBinding::SharedSkeleton),
        |body, source, compiled, binding| bind_part(body, source, compiled, binding, &body_catalog),
    )
}

fn source(
    resource: &ResourceLocator<'static>,
    geosets: Option<&GeosetSelection>,
    replacements: &BTreeMap<u32, TextureReplacement>,
    models: &Models<'_>,
) -> Result<Source, ModelError> {
    let model = models
        .get(resource)
        .ok_or_else(|| ModelError(format!("missing assembly model {resource:?}")))?;
    let catalog = CharacterCatalog::inspect(model);
    let draws = geosets
        .map(|selection| {
            selection
                .draws(&catalog)
                .map(|draws| draws.into_iter().map(|draw| draw.draw).collect())
                .map_err(|error| ModelError(error.to_string()))
        })
        .transpose()?;
    let textures = model
        .textures()
        .filter_map(|(id, texture)| {
            match model
                .image_resource(texture.resource)
                .expect("validated image resource")
            {
                ImageResource::Replaceable(category) => replacements
                    .get(category)
                    .map(|replacement| (id, replacement.clone())),
                _ => None,
            }
        })
        .collect();
    Ok(Source {
        model: resource.clone(),
        draws: DrawSelection {
            draws,
            ..Default::default()
        },
        textures,
    })
}

fn bind_part(
    body: &Model<'_>,
    model: &Model<'_>,
    compiled_part: &CompiledModel,
    part: &NativeBinding,
    body_catalog: &CharacterCatalog,
) -> Result<PartBinding, ModelError> {
    let catalog = CharacterCatalog::inspect(model);
    Ok(match &part.binding {
        EquipmentBinding::Attachment(identity) => {
            let points: Vec<_> = body
                .features()
                .filter_map(|(id, feature)| {
                    let point = match &feature.payload {
                        NodeFeature::Attachment(value) => value.socket(),
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
            PartBinding::Mounted(ModelMount {
                target: MountTarget::Socket(point),
                transform: ModelTransform {
                    translation: pivot,
                    ..Default::default()
                },
                ..ModelMount::new(MountTarget::Socket(point))
            })
        }
        EquipmentBinding::SharedSkeleton => {
            let mut joints: Vec<_> = part
                .joints
                .iter()
                .map(|(source, target)| (*source, *target))
                .collect();
            let explicit: HashSet<_> = part.joints.keys().copied().collect();
            let required = weighted_joints(compiled_part);
            let mut required: Vec<_> = required
                .into_iter()
                .filter(|node| !explicit.contains(node))
                .collect();
            required.sort();
            if part.retain_unmatched_joints {
                for node in required {
                    match body_catalog.match_joints(&catalog, &[node]) {
                        Ok(matches) => joints.extend(matches),
                        Err(JointMatchError::Unmatched(_)) => {}
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
            let mut mapped_source: HashSet<_> = joints.iter().map(|(source, _)| *source).collect();
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
    })
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

#[cfg(test)]
#[path = "assembly_tests.rs"]
mod tests;
