use super::{
    CharacterCatalog, CharacterDefinition, EquipmentBinding, GeosetSelection, JointMatchError,
};
use crate::assembly::{
    compile_with_bindings, ImageSink, Models, PendingAssembly, PendingPart, RgbaImage, Source,
    TextureReplacement,
};
use crate::bounds::rest_bounds;
use crate::mesh_attributes::{EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT};
use crate::{CompiledModel, DrawSelection, JointMap, ModelError, PartBinding};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::{Mat4, Mesh};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use wc3::model::systems::{model::System as M2System, System};
use wc3::model::PoseLayout;
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
/// Images are emitted to the sink during preparation; a staging sink can defer
/// publication until the entire character has compiled successfully.
pub fn compile_character(
    definition: &CharacterDefinition,
    models: &Models<'_>,
    images: &mut dyn ImageSink,
    mut resolve_image: impl FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>,
) -> Result<CompiledModel, ModelError> {
    compile_character_impl(definition, models, images, &mut resolve_image)
}

// Keep preparation code generation in this crate; only image I/O crosses the callback boundary.
#[inline(never)]
fn compile_character_impl(
    definition: &CharacterDefinition,
    models: &Models<'_>,
    images: &mut dyn ImageSink,
    resolve_image: &mut dyn FnMut(&ImageResource<'static>) -> Result<RgbaImage, ModelError>,
) -> Result<CompiledModel, ModelError> {
    definition
        .validate()
        .map_err(|error| ModelError(error.to_string()))?;
    let root_node_count = models
        .get(&definition.model)
        .map_or(0, |model| model.nodes().len());
    let corrections = bone_corrections(
        models
            .get(&definition.model)
            .ok_or_else(|| ModelError("missing character model".into()))?,
        &definition.bone_sets,
    )?;
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
    let mut compiled = compile_with_bindings(
        &assembly,
        models,
        images,
        resolve_image,
        |binding| matches!(binding.binding, EquipmentBinding::SharedSkeleton),
        |body, source, compiled, binding| bind_part(body, source, compiled, binding, &body_catalog),
    )?;
    if !corrections.is_empty() {
        apply_root_corrections(&mut compiled, root_node_count, &corrections)?;
    }
    Ok(compiled)
}

fn apply_root_corrections(
    compiled: &mut CompiledModel,
    root_node_count: usize,
    corrections: &HashMap<NodeId, Mat4>,
) -> Result<(), ModelError> {
    // Image rebinding must retain the same selected face as the visible model.
    if let Some(template) = &mut compiled.image_template {
        apply_root_corrections(Arc::make_mut(template), root_node_count, corrections)?;
    }
    // Composition retains the root's node prefix; source-local identities in
    // appended parts are not root customization targets.
    for node in Arc::make_mut(&mut compiled.nodes)
        .iter_mut()
        .take(root_node_count)
    {
        node.correction = corrections.get(&node.id).copied();
    }
    // Validate every descendant, including composed equipment, before publishing a model.
    PoseLayout::new(compiled.nodes.to_vec()).map_err(|error| ModelError(error.to_string()))?;
    compiled.rest_bounds = rest_bounds(compiled, None);
    Ok(())
}

fn bone_corrections(
    model: &Model<'_>,
    selected: &HashSet<ResourceLocator<'static>>,
) -> Result<HashMap<NodeId, Mat4>, ModelError> {
    let mut remaining = selected.clone();
    let mut corrections = HashMap::new();
    for (_, system) in model.systems() {
        let System::M2(M2System::BoneOverrides(sets)) = system else {
            continue;
        };
        for set in sets {
            let Some(resource) = set.resource.and_then(|id| model.bone_override_resource(id))
            else {
                continue;
            };
            if !selected.contains(resource) {
                continue;
            }
            if !remaining.remove(resource) {
                return Err(ModelError(format!(
                    "ambiguous native bone correction resource {resource:?}"
                )));
            }
            for (index, node) in set.bones.iter().enumerate() {
                let matrix = set
                    .transforms
                    .get(index)
                    .ok_or_else(|| ModelError("native bone correction matrix is missing".into()))?;
                if corrections.insert(*node, matrix).is_some() {
                    return Err(ModelError(format!(
                        "selected native bone sets overlap at {node:?}"
                    )));
                }
            }
        }
    }
    if let Some(resource) = remaining.iter().next() {
        return Err(ModelError(format!(
            "selected native bone correction resource is absent: {resource:?}"
        )));
    }
    Ok(corrections)
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
    let images = model
        .dependencies()
        .images()
        .iter()
        .filter_map(|(id, resource)| match resource {
            ImageResource::Replaceable { id: category, .. } => replacements
                .get(category)
                .map(|replacement| (id, replacement.clone())),
            _ => None,
        })
        .collect();
    Ok(Source {
        model: resource.clone(),
        material: Default::default(),
        draws: DrawSelection {
            draws,
            ..Default::default()
        },
        images,
        inputs: BTreeMap::new(),
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
                .iter()
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
