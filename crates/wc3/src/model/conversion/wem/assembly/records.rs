//! One source inventory separates independent section revisions before graph assembly.
use super::super::{actors::common::Field, hierarchy::poses::rows};
use crate::model::conversion::Error;
use crate::model::wem::{self, records, InlineList, RawDocument, RecordLayout, Ref};

pub(super) struct Tree {
    pub nodes: Ref<records::targets::Node>,
    pub poses: Ref<records::targets::PoseSchema>,
    pub reference: u32,
    pub convention: Option<records::RigConvention>,
}
trait TreeSource {
    fn inventory(&self) -> Tree;
}
impl Field<records::RigConvention> for records::RigConvention {
    fn read(&self) -> Option<records::RigConvention> {
        Some(*self)
    }
    fn write(value: Option<records::RigConvention>) -> Result<Self, Error> {
        value.ok_or_else(|| {
            Error::new(
                "wem.hierarchy.convention",
                "selected layout requires a hierarchy convention",
            )
        })
    }
}
impl<D: records::NodeTreeDialect> TreeSource for records::NodeTree<D>
where
    D::Rig: Field<records::RigConvention>,
{
    fn inventory(&self) -> Tree {
        Tree {
            nodes: self.nodes,
            poses: self.pose_schema,
            reference: self.authoritative_pose,
            convention: self.rig.read(),
        }
    }
}
impl Field<records::LodExport> for records::LodExport {
    fn read(&self) -> Option<records::LodExport> {
        Some(self.clone())
    }
    fn write(value: Option<records::LodExport>) -> Result<Self, Error> {
        value.ok_or_else(|| {
            Error::new(
                "wem.lod",
                "selected layout requires an LOD preparation declaration",
            )
        })
    }
}
impl Field<u32> for u32 {
    fn read(&self) -> Option<u32> {
        Some(*self)
    }
    fn write(value: Option<u32>) -> Result<Self, Error> {
        value.ok_or_else(|| {
            Error::new(
                "wem.selection",
                "selected layout requires a selection field",
            )
        })
    }
}
pub(super) enum Physics {
    Absent,
    Legacy(records::PhysicsSet<records::V8>),
    Current(records::PhysicsSet<records::V9>),
}
trait PhysicsSource {
    fn inventory(&self) -> Physics;
}
impl PhysicsSource for () {
    fn inventory(&self) -> Physics {
        Physics::Absent
    }
}
impl PhysicsSource for records::PhysicsSet<records::V8> {
    fn inventory(&self) -> Physics {
        Physics::Legacy(self.clone())
    }
}
impl PhysicsSource for records::PhysicsSet<records::V9> {
    fn inventory(&self) -> Physics {
        Physics::Current(self.clone())
    }
}
pub(super) struct Model {
    pub name: Ref<records::targets::Character>,
    pub meshes: Ref<records::targets::Mesh>,
    pub tree: Tree,
    pub slots: InlineList<Ref<records::targets::Character>>,
    pub channels: Ref<records::targets::AnimChannel>,
    pub animation_set: u32,
    pub materials: Ref<records::targets::ProfileMaterialSet>,
    pub bounds: records::Extent,
    pub tests: Option<Ref<records::targets::TestPose>>,
    pub lod: Option<records::LodExport>,
    pub selected_test: Option<u32>,
    pub selections: Option<Ref<records::targets::TrackSet>>,
    pub stages: Option<Ref<records::targets::PoseStage>>,
    pub physics: Physics,
}
fn models<D: records::ModelRecordDialect>(
    document: &RawDocument<'_>,
    reference: Ref<records::targets::Model>,
) -> Result<Vec<Model>, Error>
where
    records::ModelRecord<D>: RecordLayout + for<'record> wem::Read<'record>,
    D::Nodes: TreeSource,
    D::TestPoses: Field<Ref<records::targets::TestPose>>,
    D::LodExport: Field<records::LodExport>,
    D::TPose: Field<u32>,
    D::TrackSets: Field<Ref<records::targets::TrackSet>>,
    D::PoseStages: Field<Ref<records::targets::PoseStage>>,
    D::Physics: PhysicsSource,
{
    rows::<records::ModelRecord<D>, _>(document, reference)?
        .into_iter()
        .map(|value| {
            Ok(Model {
                name: value.name,
                meshes: value.meshes,
                tree: value.nodes.inventory(),
                slots: value.material_slots,
                channels: value.anim_channels.channels,
                animation_set: value.anim_set,
                materials: value.profile_sets,
                bounds: value.bounds,
                tests: value.test_poses.read(),
                lod: value.lod_export.read(),
                selected_test: value.t_pose.read(),
                selections: value.track_sets.read(),
                stages: value.pose_stages.read(),
                physics: value.physics.inventory(),
            })
        })
        .collect()
}
pub(super) fn read(
    document: &RawDocument<'_>,
    reference: Ref<records::targets::Model>,
) -> Result<Vec<Model>, Error> {
    let Some(section) = document
        .referenced_section(reference)
        .map_err(|error| Error::new("wem.models", error.to_string()))?
    else {
        return Ok(Vec::new());
    };
    match section.revision() {
        1 => models::<records::V1>(document, reference),
        2 => models::<records::V2>(document, reference),
        3 => models::<records::V3>(document, reference),
        4 => models::<records::V4>(document, reference),
        5 => models::<records::V5>(document, reference),
        6 => models::<records::V6>(document, reference),
        7 => models::<records::V7>(document, reference),
        8 => models::<records::V8>(document, reference),
        9 => models::<records::V9>(document, reference),
        _ => Err(Error::new(
            "wem.models",
            "unsupported model record revision",
        )),
    }
}
