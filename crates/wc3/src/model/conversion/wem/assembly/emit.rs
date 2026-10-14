//! Model payloads share one destination dependency table and resolve only current logical identities.
use super::super::{appearance, geometry, hierarchy, physics, stages, text};
use super::{maps, model_records::EncodedModel, resources::Encoded as TextureIndices};
use crate::model::conversion::{Asset, EncodingHint, Error};
use crate::model::semantic::profiles::wem::{AnimationSetId, HierarchyConvention, System};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{ClipId, ModelId, SystemId};
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::{BTreeMap, HashMap};

pub(super) struct Encoded {
    pub maps: maps::Indices,
    pub appearance: appearance::Encoded,
    pub physics: Option<(SystemId, physics::Encoded)>,
    pub stages: stages::Encoded,
    pub record: EncodedModel,
}
fn error(message: impl ToString) -> Error {
    Error::new("wem.model", message.to_string())
}
pub(super) fn write<'view, 'data: 'view>(
    document: &mut RawDocument<'view>,
    asset: &'view Asset<'data, Wem>,
    textures: &TextureIndices,
    clips: &BTreeMap<ClipId, u32>,
    animation_sets: &BTreeMap<AnimationSetId, u32>,
    hints: &[EncodingHint<'_>],
) -> Result<Vec<Encoded>, Error> {
    let model_indices = asset
        .models
        .iter()
        .enumerate()
        .map(|(index, model)| {
            u32::try_from(index)
                .map(|index| (model.id(), index))
                .map_err(error)
        })
        .collect::<Result<BTreeMap<ModelId, u32>, _>>()?;
    let mut models = Vec::new();
    for model in asset.models.iter() {
        let maps = maps::collect(model)?;
        let appearance = appearance::write(document, model, &textures.indices, hints, &|id| {
            textures
                .filenames
                .get(&id)
                .copied()
                .ok_or_else(|| error("StarCraft image requires an authored filename"))
        })?;
        let node_hash: HashMap<_, _> = maps.nodes.iter().map(|(id, index)| (*id, *index)).collect();
        let slot_hash: HashMap<_, _> = appearance
            .slot_indices
            .iter()
            .map(|(id, index)| (*id, *index))
            .collect();
        let meshes = model
            .meshes()
            .map(|(_, mesh)| geometry::write(document, mesh, &node_hash, &slot_hash))
            .collect::<Result<Vec<_>, _>>()?;
        let mesh_reference = hierarchy::poses::put(document, &meshes)?;
        let nodes = hierarchy::write(
            document,
            model,
            &hierarchy::WriteContext {
                nodes: &maps.nodes,
                meshes: &maps.meshes,
                models: &model_indices,
                features: &maps.features,
                textures: &textures.indices,
                slots: &appearance.slot_indices,
                sections: &maps.first_sections,
            },
        )?;
        let (poses, reference_pose, _) =
            hierarchy::poses::write(document, &model.controls().poses)?;
        let mut systems = model.systems();
        let physics = systems
            .next()
            .map(|(id, value)| match value {
                System::Physics(value) => physics::write_packed(
                    document,
                    value,
                    &maps.nodes,
                    &maps.meshes,
                    &maps.section_indices,
                    &appearance.slot_indices,
                    &maps::packed(model),
                )
                .map(|value| (id, value)),
            })
            .transpose()?;
        if systems.next().is_some() {
            return Err(error(
                "one model has more simulation systems than its native inline declaration can represent",
            ));
        }
        let empty_rigs = BTreeMap::new();
        let empty_cloths = BTreeMap::new();
        let rig_ids = physics
            .as_ref()
            .map(|(_, value)| &value.rigs)
            .unwrap_or(&empty_rigs);
        let cloth_ids = physics
            .as_ref()
            .map(|(_, value)| &value.cloths)
            .unwrap_or(&empty_cloths);
        let stages = stages::write(
            document,
            &model.controls().pose_pipeline,
            &maps.nodes,
            physics.as_ref().map(|(id, _)| *id),
            rig_ids,
            cloth_ids,
        )?;
        let tests = model
            .controls()
            .test_poses()
            .map(|(_, value)| hierarchy::poses::write_test(document, value, clips))
            .collect::<Result<Vec<_>, _>>()?;
        let test_indices = model
            .controls()
            .test_poses()
            .enumerate()
            .map(|(index, (id, _))| u32::try_from(index).map(|index| (id, index)).map_err(error))
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let tests = hierarchy::poses::put(document, &tests)?;
        let controls = model.controls();
        let selected_test = controls
            .selected_test()
            .map(|id| {
                test_indices
                    .get(&id)
                    .copied()
                    .ok_or_else(|| error("selected test pose belongs to another model"))
            })
            .transpose()?;
        let bounds = model
            .bounds()
            .ok_or_else(|| error("native model requires authored bounds"))?;
        let record = EncodedModel {
            name: text::write_bytes(document, model.name().as_bytes())?,
            meshes: mesh_reference,
            nodes,
            poses,
            reference_pose,
            convention: controls.hierarchy_convention.map(|value| match value {
                HierarchyConvention::PivotRelative => records::RigConvention::PivotRelative,
                HierarchyConvention::ExplicitBind => records::RigConvention::ExplicitBind,
                HierarchyConvention::Unknown(value) => records::RigConvention::Unknown(value),
            }),
            slots: appearance.slots.clone(),
            channels: Ref::default(),
            animation_set: controls
                .animation_set
                .map(|id| {
                    animation_sets
                        .get(&id)
                        .copied()
                        .ok_or_else(|| error("default animation set belongs to another document"))
                })
                .transpose()?
                .unwrap_or(u32::MAX),
            materials: appearance.sets,
            bounds: records::Extent {
                minimum: bounds.minimum,
                maximum: bounds.maximum,
                sphere_radius: bounds.sphere_radius,
            },
            tests,
            lod: controls
                .lod_preparation
                .as_ref()
                .map(|value| records::LodExport {
                    generate: value.generate.into(),
                    ratio1: value.triangle_ratios[0],
                    ratio2: value.triangle_ratios[1],
                    ratio3: value.triangle_ratios[2],
                    lock_borders: value.lock_borders.into(),
                    source_had_levels: value.source_had_levels.into(),
                }),
            selected_test,
            selections: Ref::default(),
            stages: stages.value,
            physics: physics.as_ref().map(|(_, value)| value.value.clone()),
        };
        models.push(Encoded {
            maps,
            appearance,
            physics,
            stages,
            record,
        });
    }
    Ok(models)
}
