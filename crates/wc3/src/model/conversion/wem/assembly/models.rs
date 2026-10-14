//! Model-local objects resolve forward references against reserved scene identities.
use super::super::{actors, appearance, geometry, hierarchy, physics, stages};
use super::{prepare::Prepared, records};
use crate::model::conversion::{EncodingHint, Error};
use crate::model::semantic::profiles::wem::{
    HierarchyConvention, LodPreparation, ModelControls, SectionId, System,
};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{
    Bounds, ClipId, MaterialSlotId, Mesh as LogicalMesh, MeshId, NodeId, Resource, SystemId,
    Texture,
};
use crate::model::wem::{
    records::{self as wire, Mesh, V1, V2},
    RawDocument, Ref,
};
use std::collections::BTreeMap;

pub(super) struct Decoded<'view> {
    pub nodes: Vec<NodeId>,
    pub meshes: Vec<MeshId>,
    pub sections: Vec<Vec<SectionId>>,
    pub appearance: appearance::Decoded<'view>,
    pub physics: Option<(SystemId, physics::Decoded<'view>)>,
    pub stages: stages::Decoded<'view>,
}
fn error(value: impl ToString) -> Error {
    Error::new("wem.model", value.to_string())
}
fn meshes<'view>(
    document: &'view RawDocument<'_>,
    reference: Ref<wire::targets::Mesh>,
    nodes: &[NodeId],
    slots: &[MaterialSlotId],
) -> Result<Vec<LogicalMesh<'view, Wem>>, Error> {
    let Some(section) = document.referenced_section(reference).map_err(error)? else {
        return Ok(Vec::new());
    };
    macro_rules! read {
        ($dialect:ty) => {
            hierarchy::poses::rows::<Mesh<$dialect>, _>(document, reference)?
                .iter()
                .map(|source| geometry::read(document, source, nodes, slots))
                .collect()
        };
    }
    match section.revision() {
        1 => read!(V1),
        2 => read!(V2),
        _ => Err(error("unsupported mesh revision")),
    }
}
pub(super) fn read<'view>(
    document: &'view RawDocument<'_>,
    prepared: &mut Prepared<'view>,
    clips: &[ClipId],
    hints: &mut Vec<EncodingHint<'view>>,
) -> Result<Vec<Decoded<'view>>, Error> {
    let mut decoded = Vec::new();
    for (index, source) in prepared.models.iter().enumerate() {
        let sources = hierarchy::read(document, source.tree.nodes)?;
        let poses = hierarchy::poses::read(document, source.tree.poses, source.tree.reference)?;
        let builder = prepared
            .builder
            .model_mut(index)
            .ok_or_else(|| error("model draft is absent"))?;
        builder.set_bounds(Some(Bounds {
            minimum: source.bounds.minimum,
            maximum: source.bounds.maximum,
            sphere_radius: source.bounds.sphere_radius,
        }));
        let nodes: Vec<_> = sources.iter().map(|_| builder.reserve_node()).collect();
        let features: BTreeMap<_, _> = sources
            .iter()
            .enumerate()
            .filter(|(_, source)| source.has_feature())
            .map(|(index, _)| (index as u32, builder.reserve_feature(nodes[index])))
            .collect();
        for (source, id) in sources.iter().zip(&nodes) {
            builder.set_node(*id, source.read_base(document, &nodes, &poses)?)?;
        }
        let (builder, resources) = prepared
            .builder
            .model_and_resources_mut(index)
            .ok_or_else(|| error("model draft is absent"))?;
        let appearance = appearance::read(
            document,
            &source.slots,
            source.materials,
            &prepared.textures,
            &mut |path| {
                let existing=resources.textures().find_map(|(id,value)|matches!(&value.resource,Resource::Path(bytes) if bytes.as_ref()==path.as_ref()).then_some(id));
                Ok(existing.unwrap_or_else(|| {
                    resources.add_texture(Texture {
                        resource: Resource::Path(path),
                        color_space: Default::default(),
                        sampler: Default::default(),
                        controls: Default::default(),
                    })
                }))
            },
            builder,
            hints,
        )?;
        builder.set_appearance(appearance.appearance.clone());
        let mut mesh_ids = Vec::new();
        let mut sections: Vec<Vec<SectionId>> = Vec::new();
        for mesh in meshes(document, source.meshes, &nodes, &appearance.slots)? {
            sections.push(mesh.geometry.sections().map(|(id, _)| id).collect());
            mesh_ids.push(builder.add_mesh(mesh)?);
        }
        let empty = Vec::new();
        let context = hierarchy::payloads::ReadContext {
            document,
            nodes: &nodes,
            meshes: &mesh_ids,
            models: &prepared.model_ids,
            features: &features,
            slots: &appearance.slots,
            textures: &prepared.textures,
            sections: sections.first().unwrap_or(&empty),
        };
        for (source_index, source_node) in sources.iter().enumerate() {
            let value = source_node.read_payload(&context)?;
            if let Some(bone) = value.bone {
                let mut node = builder
                    .model()
                    .node(nodes[source_index])
                    .ok_or_else(|| error("reserved node is absent"))?
                    .clone();
                node.controls.bone = Some(bone);
                builder.set_node(nodes[source_index], node)?;
            }
            if let Some(value) = value.feature {
                builder.set_feature(
                    *features
                        .get(&(source_index as u32))
                        .ok_or_else(|| error("attached behavior has no reserved identity"))?,
                    value,
                )?;
            }
        }
        for channel in document
            .resolve::<wire::AnimChannel>(source.channels.cast())
            .map_err(error)?
            .iter()
        {
            let channel = channel.map_err(error)?;
            if channel.target.kind != wire::TrackTargetKind::Node
                || channel.target.channel != wire::Channel::EmitterProperty
            {
                continue;
            }
            let Some(id) = features.get(&channel.target.node).copied() else {
                continue;
            };
            let mut actor = builder
                .model()
                .feature(id)
                .ok_or_else(|| error("emitter field owner is absent"))?
                .payload
                .clone();
            if actors::bind_property(&mut actor, id, channel.target.sub)?.is_some() {
                builder.set_feature(id, actor)?;
            }
        }
        let native_physics = match &source.physics {
            records::Physics::Absent => None,
            records::Physics::Legacy(value) => Some(physics::read(
                document,
                value,
                &nodes,
                &mesh_ids,
                &sections,
                &appearance.slots,
            )?),
            records::Physics::Current(value) => Some(physics::read(
                document,
                value,
                &nodes,
                &mesh_ids,
                &sections,
                &appearance.slots,
            )?),
        };
        let physics = native_physics
            .map(|value| {
                builder
                    .add_system(System::Physics(value.physics.clone()))
                    .map(|id| (id, value))
            })
            .transpose()?;
        let empty_map = BTreeMap::new();
        let rig_ids = physics
            .as_ref()
            .map(|(_, value)| &value.rigs)
            .unwrap_or(&empty_map);
        let empty_map = BTreeMap::new();
        let cloth_ids = physics
            .as_ref()
            .map(|(_, value)| &value.cloths)
            .unwrap_or(&empty_map);
        let stages = stages::read(
            document,
            source.stages.unwrap_or_default(),
            &nodes,
            physics.as_ref().map(|(id, _)| *id),
            rig_ids,
            cloth_ids,
        )?;
        let mut controls = ModelControls::default();
        controls.poses = poses;
        controls.pose_pipeline = stages.value.clone();
        controls.hierarchy_convention = source.tree.convention.map(|value| match value {
            wire::RigConvention::PivotRelative => HierarchyConvention::PivotRelative,
            wire::RigConvention::ExplicitBind => HierarchyConvention::ExplicitBind,
            wire::RigConvention::Unknown(value) => HierarchyConvention::Unknown(value),
        });
        controls.lod_preparation = source.lod.as_ref().map(|value| LodPreparation {
            generate: value.generate.get(),
            triangle_ratios: [value.ratio1, value.ratio2, value.ratio3],
            lock_borders: value.lock_borders.get(),
            source_had_levels: value.source_had_levels.get(),
        });

        let mut tests = Vec::new();
        if let Some(reference) = source.tests {
            for value in hierarchy::poses::rows::<wire::TestPose, _>(document, reference)? {
                tests.push(
                    controls.add_test_pose(hierarchy::poses::read_test(document, &value, clips)?),
                );
            }
        }
        let selected = source
            .selected_test
            .filter(|value| *value != u32::MAX)
            .map(|index| {
                tests
                    .get(index as usize)
                    .copied()
                    .ok_or_else(|| error("selected test pose is outside the model test table"))
            })
            .transpose()?;
        controls.set_selected_test(selected)?;
        builder.set_controls(controls);
        decoded.push(Decoded {
            nodes,
            meshes: mesh_ids,
            sections,
            appearance,
            physics,
            stages,
        });
    }
    Ok(decoded)
}
