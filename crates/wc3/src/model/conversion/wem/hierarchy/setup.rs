//! Skin-editing constraints and rig classifications attach to their actual nodes.
use super::super::actors::common::Field;
use super::poses::{put, rows};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::poses::{
    self, Correction, CorrectionOrigin, PoseId, Rig, SkinSetup,
};
use crate::model::semantic::{NodeId, Quaternion};
use crate::model::wem::{records, InlineOption, RawDocument, Ref};
use std::collections::BTreeMap;

macro_rules! choices {
    ($read:ident,$write:ident,$type:ident,$($variant:ident),+ $(,)?)=>{
        fn $read(value:records::$type)->poses::$type{match value{$(records::$type::$variant=>poses::$type::$variant,)+records::$type::Unknown(value)=>poses::$type::Unknown(value)}}
        fn $write(value:poses::$type)->records::$type{match value{$(poses::$type::$variant=>records::$type::$variant,)+poses::$type::Unknown(value)=>records::$type::Unknown(value)}}
    }
}
choices!(
    read_role, write_role, RigRole, None, Body, Spine, Neck, Head, Clavicle, Upper, Lower, Hock,
    End, Toe, Twist, Pad, Prop
);
choices!(read_side, write_side, RigSide, Centre, Left, Right);
choices!(read_limb, write_limb, RigLimb, Other, Arm, Leg);
choices!(
    read_plant,
    write_plant,
    RigPlant,
    ByRole,
    OnGround,
    Always,
    Free
);
choices!(
    read_source,
    write_source,
    RigSource,
    None,
    File,
    Name,
    Attachment,
    Shape,
    You
);
choices!(
    read_motion,
    write_motion,
    JointMotion,
    Bend,
    Twist,
    Spin,
    Slide
);
choices!(read_axis, write_axis, JointAxis, X, Y, Z, Along);
choices!(
    read_falloff,
    write_falloff,
    EnvelopeFalloff,
    Hard,
    Linear,
    Smooth,
    Gaussian
);
fn node(value: u32, nodes: &[NodeId]) -> Result<Option<NodeId>, Error> {
    if value == u32::MAX {
        Ok(None)
    } else {
        nodes
            .get(value as usize)
            .copied()
            .map(Some)
            .ok_or_else(|| Error::new("wem.node.setup", "node is outside the model hierarchy"))
    }
}
fn index(value: Option<NodeId>, nodes: &BTreeMap<NodeId, u32>) -> Result<u32, Error> {
    value
        .map(|id| {
            nodes
                .get(&id)
                .copied()
                .ok_or_else(|| Error::new("wem.node.setup", "node belongs to another hierarchy"))
        })
        .transpose()
        .map(|value| value.unwrap_or(u32::MAX))
}
pub(in crate::model::conversion::wem) trait RigField {
    fn supports(value: Option<&Rig>) -> bool;
    fn read_setup(&self, nodes: &[NodeId]) -> Result<Option<Rig>, Error>;
    fn write_setup(value: Option<&Rig>, nodes: &BTreeMap<NodeId, u32>) -> Result<Self, Error>
    where
        Self: Sized;
}
impl RigField for () {
    fn supports(value: Option<&Rig>) -> bool {
        value.is_none()
    }
    fn read_setup(&self, _nodes: &[NodeId]) -> Result<Option<Rig>, Error> {
        Ok(None)
    }
    fn write_setup(value: Option<&Rig>, _nodes: &BTreeMap<NodeId, u32>) -> Result<Self, Error> {
        if value.is_some() {
            Err(Error::new(
                "wem.node.rig",
                "selected node layout omits rig classification",
            ))
        } else {
            Ok(())
        }
    }
}
impl RigField for records::NodeRig {
    fn supports(value: Option<&Rig>) -> bool {
        value.is_some()
    }
    fn read_setup(&self, nodes: &[NodeId]) -> Result<Option<Rig>, Error> {
        Ok(Some(Rig {
            role: read_role(self.role),
            side: read_side(self.side),
            limb: read_limb(self.limb),
            rides_with: node(self.rides_with, nodes)?,
            plant: read_plant(self.plant),
            twist_share: self.twist_share,
            source: read_source(self.source),
            rides_source: read_source(self.rides_source),
        }))
    }
    fn write_setup(value: Option<&Rig>, nodes: &BTreeMap<NodeId, u32>) -> Result<Self, Error> {
        let value = value.ok_or_else(|| {
            Error::new(
                "wem.node.rig",
                "selected layout requires rig classification",
            )
        })?;
        Ok(Self {
            role: write_role(value.role),
            side: write_side(value.side),
            limb: write_limb(value.limb),
            rides_with: index(value.rides_with, nodes)?,
            plant: write_plant(value.plant),
            twist_share: value.twist_share,
            source: write_source(value.source),
            rides_source: write_source(value.rides_source),
        })
    }
}
pub(in crate::model::conversion::wem) trait SkinField {
    fn supports(value: Option<&SkinSetup>) -> bool;
    fn read_setup(
        &self,
        document: &RawDocument<'_>,
        nodes: &[NodeId],
        poses: &[PoseId],
    ) -> Result<Option<SkinSetup>, Error>;
    fn write_setup(
        document: &mut RawDocument<'_>,
        value: Option<&SkinSetup>,
        nodes: &BTreeMap<NodeId, u32>,
        poses: &[PoseId],
    ) -> Result<Self, Error>
    where
        Self: Sized;
}
impl SkinField for () {
    fn supports(value: Option<&SkinSetup>) -> bool {
        value.is_none()
    }
    fn read_setup(
        &self,
        _document: &RawDocument<'_>,
        _nodes: &[NodeId],
        _poses: &[PoseId],
    ) -> Result<Option<SkinSetup>, Error> {
        Ok(None)
    }
    fn write_setup(
        _document: &mut RawDocument<'_>,
        value: Option<&SkinSetup>,
        _nodes: &BTreeMap<NodeId, u32>,
        _poses: &[PoseId],
    ) -> Result<Self, Error> {
        if value.is_some() {
            Err(Error::new(
                "wem.node.skin",
                "selected layout omits skin editing setup",
            ))
        } else {
            Ok(())
        }
    }
}
fn read_origin(value: records::TPoseSource) -> CorrectionOrigin {
    match value {
        records::TPoseSource::None => CorrectionOrigin::Unspecified,
        records::TPoseSource::Donor => CorrectionOrigin::Donor,
        records::TPoseSource::Record => CorrectionOrigin::Recorded,
        records::TPoseSource::Clips => CorrectionOrigin::Clips,
        records::TPoseSource::BindFrame => CorrectionOrigin::BindFrame,
        records::TPoseSource::You => CorrectionOrigin::Manual,
        records::TPoseSource::Unknown(value) => CorrectionOrigin::Unknown(value),
    }
}
fn write_origin(value: CorrectionOrigin) -> records::TPoseSource {
    match value {
        CorrectionOrigin::Unspecified => records::TPoseSource::None,
        CorrectionOrigin::Donor => records::TPoseSource::Donor,
        CorrectionOrigin::Recorded => records::TPoseSource::Record,
        CorrectionOrigin::Clips => records::TPoseSource::Clips,
        CorrectionOrigin::BindFrame => records::TPoseSource::BindFrame,
        CorrectionOrigin::Manual => records::TPoseSource::You,
        CorrectionOrigin::Unknown(value) => records::TPoseSource::Unknown(value),
    }
}
impl<D: records::NodeSkinSetupDialect> SkinField for records::NodeSkinSetup<D>
where
    D::PoseSources: Field<Ref<records::targets::TPoseSourceRecord>>,
{
    fn supports(value: Option<&SkinSetup>) -> bool {
        value.is_some_and(|value| {
            !value.corrections.iter().any(|value| value.origin.is_some())
                || D::PoseSources::write(Some(Ref::default())).is_ok()
        })
    }
    fn read_setup(
        &self,
        document: &RawDocument<'_>,
        nodes: &[NodeId],
        poses: &[PoseId],
    ) -> Result<Option<SkinSetup>, Error> {
        let deltas = rows::<records::PoseDelta, _>(document, self.pose_deltas)?;
        let origins = self
            .pose_sources
            .read()
            .map(|reference| rows::<records::TPoseSourceRecord, _>(document, reference))
            .transpose()?;
        if deltas.len() > poses.len()
            || origins
                .as_ref()
                .is_some_and(|origins| !origins.is_empty() && origins.len() != deltas.len())
        {
            return Err(Error::new(
                "wem.node.skin.corrections",
                "saved correction tables have inconsistent cardinality",
            ));
        }
        Ok(Some(SkinSetup {
            locked: self.locked.get(),
            envelope: self.envelope.value.as_ref().map(|value| poses::Envelope {
                inner_start: value.inner_start,
                inner_end: value.inner_end,
                outer_start: value.outer_start,
                outer_end: value.outer_end,
                falloff: read_falloff(value.falloff),
            }),
            joint: self.joint.value.as_ref().map(|value| poses::Joint {
                motion: read_motion(value.motion),
                axis: read_axis(value.axis),
                parent_space: value.parent_space.get(),
                low: value.low,
                high: value.high,
            }),
            mirror: node(self.mirror, nodes)?,
            corrections: deltas
                .into_iter()
                .enumerate()
                .map(|(index, value)| Correction {
                    pose: poses[index],
                    rotation: Quaternion(value.rotation),
                    translation: value.translation,
                    origin: origins
                        .as_ref()
                        .and_then(|values| values.get(index))
                        .map(|value| read_origin(value.value)),
                })
                .collect(),
        }))
    }
    fn write_setup(
        document: &mut RawDocument<'_>,
        value: Option<&SkinSetup>,
        nodes: &BTreeMap<NodeId, u32>,
        poses: &[PoseId],
    ) -> Result<Self, Error> {
        let value = value.ok_or_else(|| {
            Error::new(
                "wem.node.skin",
                "selected layout requires skin editing setup",
            )
        })?;
        if value
            .corrections
            .iter()
            .any(|value| !poses.contains(&value.pose))
        {
            return Err(Error::new(
                "wem.node.skin.corrections",
                "correction belongs to another saved convention table",
            ));
        }
        let mut deltas = Vec::new();
        let mut origins = Vec::new();
        let mut gap = false;
        let mut any_origin = false;
        let mut absent_origin = false;
        for pose in poses {
            match value.corrections.iter().find(|value| value.pose == *pose) {
                Some(value) if !gap => {
                    deltas.push(records::PoseDelta {
                        rotation: value.rotation.components(),
                        translation: value.translation,
                    });
                    if let Some(origin) = value.origin {
                        any_origin = true;
                        origins.push(records::TPoseSourceRecord {
                            value: write_origin(origin),
                        });
                    } else {
                        absent_origin = true;
                    }
                }
                Some(_) => {
                    return Err(Error::new(
                        "wem.node.skin.corrections",
                        "saved corrections cannot contain absent interior slots",
                    ));
                }
                None => gap = true,
            }
        }
        if any_origin && absent_origin {
            return Err(Error::new(
                "wem.node.skin.corrections",
                "correction origin table cannot mix declared and absent origins",
            ));
        }
        let pose_sources = if any_origin {
            D::PoseSources::write(Some(put(document, &origins)?))?
        } else {
            D::PoseSources::write(Some(Ref::default())).or_else(|_| D::PoseSources::write(None))?
        };
        Ok(Self {
            locked: value.locked.into(),
            envelope: InlineOption::from(value.envelope.as_ref().map(|value| records::Envelope {
                inner_start: value.inner_start,
                inner_end: value.inner_end,
                outer_start: value.outer_start,
                outer_end: value.outer_end,
                falloff: write_falloff(value.falloff),
            })),
            joint: InlineOption::from(value.joint.as_ref().map(|value| records::Joint {
                motion: write_motion(value.motion),
                axis: write_axis(value.axis),
                parent_space: value.parent_space.into(),
                low: value.low,
                high: value.high,
            })),
            mirror: index(value.mirror, nodes)?,
            pose_deltas: put(document, &deltas)?,
            pose_sources,
        })
    }
}
