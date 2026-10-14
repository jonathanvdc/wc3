//! A shared model-table vocabulary preserves optional authoring declarations.
use crate::model::conversion::wem::actors::common::Field;
use crate::model::conversion::wem::hierarchy::poses::put;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{HierarchyConvention, System};
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::Model;
use crate::model::wem::{self, records, InlineList, RawDocument, Ref, Target};

pub(super) struct EncodedModel {
    pub name: Ref<records::targets::Character>,
    pub meshes: Ref<records::targets::Mesh>,
    pub nodes: Ref<records::targets::Node>,
    pub poses: Ref<records::targets::PoseSchema>,
    pub reference_pose: u32,
    pub convention: Option<records::RigConvention>,
    pub slots: InlineList<Ref<records::targets::Character>>,
    pub channels: Ref<records::targets::AnimChannel>,
    pub animation_set: u32,
    pub materials: Ref<records::targets::ProfileMaterialSet>,
    pub bounds: records::Extent,
    pub tests: Ref<records::targets::TestPose>,
    pub lod: Option<records::LodExport>,
    pub selected_test: Option<u32>,
    pub selections: Ref<records::targets::TrackSet>,
    pub stages: Ref<records::targets::PoseStage>,
    pub physics: Option<records::PhysicsSet<records::V9>>,
}
trait TreeField: Sized {
    fn supports(convention: Option<records::RigConvention>) -> bool;
    fn write(value: &EncodedModel) -> Result<Self, Error>;
}
impl<D: records::NodeTreeDialect> TreeField for records::NodeTree<D>
where
    D::Rig: Field<records::RigConvention>,
{
    fn supports(convention: Option<records::RigConvention>) -> bool {
        D::Rig::write(convention).is_ok()
    }
    fn write(value: &EncodedModel) -> Result<Self, Error> {
        Ok(Self {
            nodes: value.nodes,
            pose_schema: value.poses,
            authoritative_pose: value.reference_pose,
            rig: D::Rig::write(value.convention)?,
        })
    }
}
trait PhysicsField: Sized {
    fn supports(present: bool, soft: bool) -> bool;
    fn write(value: Option<&records::PhysicsSet<records::V9>>) -> Result<Self, Error>;
}
impl PhysicsField for () {
    fn supports(present: bool, _: bool) -> bool {
        !present
    }
    fn write(value: Option<&records::PhysicsSet<records::V9>>) -> Result<Self, Error> {
        if value.is_none() {
            Ok(())
        } else {
            Err(Error::new(
                "wem.physics",
                "selected model vocabulary omits simulation declarations",
            ))
        }
    }
}
impl PhysicsField for records::PhysicsSet<records::V9> {
    fn supports(present: bool, _: bool) -> bool {
        present
    }
    fn write(value: Option<&Self>) -> Result<Self, Error> {
        value.cloned().ok_or_else(|| {
            Error::new(
                "wem.physics",
                "selected model vocabulary requires a simulation declaration",
            )
        })
    }
}
impl PhysicsField for records::PhysicsSet<records::V8> {
    fn supports(present: bool, soft: bool) -> bool {
        present && !soft
    }
    fn write(value: Option<&records::PhysicsSet<records::V9>>) -> Result<Self, Error> {
        let value = value.ok_or_else(|| {
            Error::new(
                "wem.physics",
                "selected model vocabulary requires a simulation declaration",
            )
        })?;
        if value.soft.entries != 0 {
            return Err(Error::new(
                "wem.physics.soft",
                "selected model vocabulary omits soft bodies",
            ));
        }
        Ok(Self {
            bodies: value.bodies,
            joints: value.joints,
            colliders: value.colliders,
            cloths: value.cloths,
            rigs: value.rigs,
            next_id: value.next_id,
            soft: (),
        })
    }
}
fn reference<T: Target, F: Field<Ref<T>>>(value: Ref<T>) -> Result<F, Error> {
    F::write(Some(value)).or_else(|error| {
        if value.entries == 0 {
            F::write(None)
        } else {
            Err(error)
        }
    })
}
fn selected<F: Field<u32>>(value: Option<u32>) -> Result<F, Error> {
    F::write(Some(value.unwrap_or(u32::MAX))).or_else(|error| {
        if value.is_none() {
            F::write(None)
        } else {
            Err(error)
        }
    })
}
trait ModelFields: records::ModelRecordDialect {
    fn supports(value: &EncodedModel) -> bool;
    fn supports_model(value: &Model<'_, Wem>) -> bool;
    fn row(value: &EncodedModel) -> Result<records::ModelRecord<Self>, Error>;
}
impl<D: records::ModelRecordDialect> ModelFields for D
where
    D::Nodes: TreeField,
    D::TestPoses: Field<Ref<records::targets::TestPose>>,
    D::LodExport: Field<records::LodExport>,
    D::TPose: Field<u32>,
    D::TrackSets: Field<Ref<records::targets::TrackSet>>,
    D::PoseStages: Field<Ref<records::targets::PoseStage>>,
    D::Physics: PhysicsField,
{
    fn supports(value: &EncodedModel) -> bool {
        D::Nodes::supports(value.convention)
            && D::LodExport::write(value.lod.clone()).is_ok()
            && reference::<_, D::TestPoses>(value.tests).is_ok()
            && selected::<D::TPose>(value.selected_test).is_ok()
            && reference::<_, D::TrackSets>(value.selections).is_ok()
            && reference::<_, D::PoseStages>(value.stages).is_ok()
            && D::Physics::supports(
                value.physics.is_some(),
                value
                    .physics
                    .as_ref()
                    .is_some_and(|value| value.soft.entries != 0),
            )
    }
    fn supports_model(value: &Model<'_, Wem>) -> bool {
        let controls = value.controls();
        let convention = controls.hierarchy_convention.map(|value| match value {
            HierarchyConvention::PivotRelative => records::RigConvention::PivotRelative,
            HierarchyConvention::ExplicitBind => records::RigConvention::ExplicitBind,
            HierarchyConvention::Unknown(value) => records::RigConvention::Unknown(value),
        });
        let has_physics = value.systems().len() != 0;
        let soft=value.systems().any(|(_,system)|matches!(system,System::Physics(physics) if physics.soft_bodies().len()!=0));
        D::Nodes::supports(convention)
            && D::LodExport::write(
                controls
                    .lod_preparation
                    .as_ref()
                    .map(|_| records::LodExport::default()),
            )
            .is_ok()
            && (controls.test_poses().len() == 0
                || D::TestPoses::write(Some(Ref::default())).is_ok())
            && (controls.selected_test().is_none() || D::TPose::write(Some(0)).is_ok())
            && (controls.channel_selections.iter().len() == 0
                || D::TrackSets::write(Some(Ref::default())).is_ok())
            && (controls.pose_pipeline.stages().len() == 0
                || D::PoseStages::write(Some(Ref::default())).is_ok())
            && D::Physics::supports(has_physics, soft)
    }
    fn row(value: &EncodedModel) -> Result<records::ModelRecord<D>, Error> {
        Ok(records::ModelRecord {
            name: value.name,
            meshes: value.meshes,
            nodes: D::Nodes::write(value)?,
            material_slots: value.slots.clone(),
            anim_channels: records::AnimChannelTable {
                channels: value.channels,
            },
            anim_set: value.animation_set,
            profile_sets: value.materials,
            bounds: value.bounds.clone(),
            test_poses: reference(value.tests)?,
            lod_export: D::LodExport::write(value.lod.clone())?,
            t_pose: selected(value.selected_test)?,
            track_sets: reference(value.selections)?,
            pose_stages: reference(value.stages)?,
            physics: D::Physics::write(value.physics.as_ref())?,
        })
    }
}
fn rows<D: ModelFields>(
    document: &mut RawDocument<'_>,
    values: &[EncodedModel],
) -> Result<Ref<records::targets::Model>, Error>
where
    records::ModelRecord<D>: wem::Write,
{
    let values = values.iter().map(D::row).collect::<Result<Vec<_>, _>>()?;
    put(document, &values)
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    values: &[EncodedModel],
) -> Result<Ref<records::targets::Model>, Error> {
    macro_rules! candidates {($($dialect:ident),+)=>{$(if values.iter().all(records::$dialect::supports){return rows::<records::$dialect>(document,values);})+};}
    candidates!(V9, V8, V7, V6, V5, V4, V3, V2, V1);
    Err(Error::new(
        "wem.models.capabilities",
        "no shared model vocabulary preserves hierarchy, LOD, and simulation declarations",
    ))
}
pub(in crate::model::conversion::wem) fn supports(models: &[Model<'_, Wem>]) -> bool {
    macro_rules! candidates {($($dialect:ident),+)=>{false $(|| models.iter().all(records::$dialect::supports_model))+};}
    candidates!(V9, V8, V7, V6, V5, V4, V3, V2, V1)
}

#[cfg(test)]
#[path = "model_records_tests.rs"]
mod tests;
