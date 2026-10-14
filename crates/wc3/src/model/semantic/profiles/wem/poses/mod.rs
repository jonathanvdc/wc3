//! Saved poses are authored transforms with explicit coordinate and inversion conventions.
mod rig_values;
pub use rig_values::*;
mod rig;
use crate::model::semantic::Scope;
use crate::model::semantic::{ClipId, Error, Id, Link, Quaternion, Time, ValidateLinks};
pub use rig::{Envelope, Joint, NodeSetup, Rig, SkinSetup};
use std::borrow::Cow;
/// Marker of a saved pose convention owned by one rig.
pub enum PoseObject {}
/// Stable saved pose identity, independent of physical array positions.
pub type PoseId = Id<PoseObject>;
/// Coordinate frame of a saved transform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PoseSpace {
    /// Coordinates relative to the model origin.
    #[default]
    Model,
    /// Coordinates relative to the node's parent.
    Parent,
    /// Unrecognized authored coordinate convention.
    Unknown(u8),
}
/// Preferred authored representation of a saved pose.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PoseRepresentation {
    /// Independent translation, rotation, and scale components.
    #[default]
    Components,
    /// Full affine matrix.
    Matrix,
    /// Unrecognized representation convention.
    Unknown(u8),
}
/// Translation, rotation, and scale without an additional pivot authority.
#[derive(Clone, Copy, Debug)]
pub struct Components {
    /// Translation in the pose's declared frame.
    pub translation: [f32; 3],
    /// Authored rotation.
    pub rotation: Quaternion,
    /// Axis scale factors.
    pub scale: [f32; 3],
}
/// A named saved transform convention shared by node pose values.
#[derive(Clone, Debug)]
pub struct Pose<'a> {
    /// Display name.
    pub name: Cow<'a, str>,
    /// Frame of the saved values.
    pub space: PoseSpace,
    /// Values represent the inverse of the named pose.
    pub inverse: bool,
    /// Selected authored representation; both supplied forms remain independently inspectable.
    pub representation: PoseRepresentation,
}
/// Values belonging to one actual node and one rig-owned saved pose identity.
#[derive(Clone, Debug)]
pub struct NodePose {
    /// Owning saved convention.
    pub pose: PoseId,
    /// Authored component form, when present.
    pub components: Option<Components>,
    /// Authored matrix form, when present.
    pub matrix: Option<[[f32; 4]; 4]>,
}
/// Origin of a saved skinning correction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CorrectionOrigin {
    /// No origin declared.
    #[default]
    Unspecified,
    /// Copied from an authoring donor.
    Donor,
    /// Recorded from an evaluated pose.
    Recorded,
    /// Derived from clip motion.
    Clips,
    /// Derived from the bind frame.
    BindFrame,
    /// Edited manually.
    Manual,
    /// Unrecognized provenance declaration.
    Unknown(u8),
}
/// Rotation and translation correction associated with a saved pose.
#[derive(Clone, Debug)]
pub struct Correction {
    /// Saved convention being corrected.
    pub pose: PoseId,
    /// Rotation delta.
    pub rotation: Quaternion,
    /// Translation delta.
    pub translation: [f32; 3],
    /// Authoring origin, when one is declared.
    pub origin: Option<CorrectionOrigin>,
}
/// A named clip frame used to test skinning setup.
#[derive(Clone, Debug)]
pub struct TestPose<'a> {
    /// Display name.
    pub name: Cow<'a, str>,
    /// Playback program providing the base; absence selects rest pose.
    pub clip: Option<ClipId>,
    /// Signed time within that program.
    pub time: Time,
}
/// Rig-owned saved conventions and explicitly selected reference pose.
#[derive(Clone, Debug, Default)]
pub struct Poses<'a> {
    scope: Scope,
    values: Vec<(PoseId, Pose<'a>)>,
    reference: Option<PoseId>,
}
impl<'a> Poses<'a> {
    /// Adds a named saved convention.
    pub fn add(&mut self, pose: Pose<'a>) -> PoseId {
        let id = self.scope.allocate();
        self.values.push((id, pose));
        id
    }
    /// Borrows a saved convention only when this rig owns the identity.
    pub fn get(&self, id: PoseId) -> Option<&Pose<'a>> {
        self.values
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Iterates saved conventions in authored order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (PoseId, &Pose<'a>)> {
        self.values.iter().map(|(id, value)| (*id, value))
    }
    /// Selects the hierarchy's actual reference pose after checking ownership.
    pub fn set_reference(&mut self, id: Option<PoseId>) -> Result<(), Error> {
        if id.is_some_and(|id| self.get(id).is_none()) {
            return Err(Error::new(
                "wem.poses.reference",
                "pose belongs to another rig",
            ));
        }
        self.reference = id;
        Ok(())
    }
    /// Selected reference convention.
    pub fn reference(&self) -> Option<PoseId> {
        self.reference
    }

    /// Checks a node's saved values against the conventions owned by this rig.
    ///
    /// A node can carry both transform representations, but the representation selected
    /// by its pose convention must actually be supplied. Each convention occurs once.
    pub fn validate_node(
        &self,
        values: &[NodePose],
        corrections: &[Correction],
    ) -> Result<(), Error> {
        for (index, value) in values.iter().enumerate() {
            let convention = self
                .get(value.pose)
                .ok_or_else(|| Error::new("wem.node.poses", "pose belongs to another rig"))?;
            if values[..index].iter().any(|other| other.pose == value.pose) {
                return Err(Error::new("wem.node.poses", "duplicate saved convention"));
            }
            value.validate()?;
            let supplied = match convention.representation {
                PoseRepresentation::Components => value.components.is_some(),
                PoseRepresentation::Matrix => value.matrix.is_some(),
                PoseRepresentation::Unknown(_) => {
                    value.components.is_some() || value.matrix.is_some()
                }
            };
            if !supplied {
                return Err(Error::new(
                    "wem.node.poses",
                    "selected pose representation is absent",
                ));
            }
        }
        for (index, correction) in corrections.iter().enumerate() {
            if self.get(correction.pose).is_none() {
                return Err(Error::new(
                    "wem.node.corrections",
                    "pose belongs to another rig",
                ));
            }
            if corrections[..index]
                .iter()
                .any(|other| other.pose == correction.pose)
            {
                return Err(Error::new(
                    "wem.node.corrections",
                    "duplicate pose correction",
                ));
            }
            correction.validate()?;
        }
        Ok(())
    }
}
impl ValidateLinks for Poses<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for TestPose<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.clip.visit_links(visitor);
    }
}

impl Default for Components {
    fn default() -> Self {
        Self {
            translation: [0.; 3],
            rotation: Quaternion::IDENTITY,
            scale: [1.; 3],
        }
    }
}
impl Components {
    /// Checks finite component values and a nonzero authored rotation.
    pub fn validate(&self) -> Result<(), Error> {
        if self
            .translation
            .iter()
            .chain(&self.scale)
            .any(|value| !value.is_finite())
            || !self.rotation.is_rotation()
        {
            return Err(Error::new("wem.pose.components", "invalid saved transform"));
        }
        Ok(())
    }
}
impl NodePose {
    /// Checks supplied pose values; rig ownership is checked where the node is published.
    pub fn validate(&self) -> Result<(), Error> {
        if let Some(value) = &self.components {
            value.validate()?;
        }
        if self
            .matrix
            .as_ref()
            .is_some_and(|matrix| matrix.iter().flatten().any(|value| !value.is_finite()))
        {
            return Err(Error::new("wem.pose.matrix", "nonfinite saved transform"));
        }
        Ok(())
    }
}
impl Correction {
    /// Checks finite correction data and a nonzero rotation delta.
    pub fn validate(&self) -> Result<(), Error> {
        if !self.rotation.is_rotation() || self.translation.iter().any(|value| !value.is_finite()) {
            return Err(Error::new(
                "wem.pose.correction",
                "invalid saved pose correction",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convention(representation: PoseRepresentation) -> Pose<'static> {
        Pose {
            name: "bind".into(),
            space: PoseSpace::Model,
            inverse: false,
            representation,
        }
    }

    fn components() -> Components {
        Components {
            translation: [0.0; 3],
            rotation: Quaternion::default(),
            scale: [1.0; 3],
        }
    }

    #[test]
    fn node_poses_require_the_owning_rig_and_selected_representation() {
        let mut rig = Poses::default();
        let own = rig.add(convention(PoseRepresentation::Matrix));
        let mut other = Poses::default();
        let foreign = other.add(convention(PoseRepresentation::Matrix));
        let mut value = NodePose {
            pose: foreign,
            components: Some(components()),
            matrix: None,
        };
        assert!(rig.validate_node(&[value.clone()], &[]).is_err());
        value.pose = own;
        assert!(rig.validate_node(&[value.clone()], &[]).is_err());
        value.matrix = Some([[0.0; 4]; 4]);
        rig.validate_node(&[value.clone()], &[]).unwrap();
        assert!(rig.validate_node(&[value.clone(), value], &[]).is_err());
    }

    #[test]
    fn pose_corrections_reject_invalid_rotations_and_foreign_conventions() {
        let mut rig = Poses::default();
        let own = rig.add(convention(PoseRepresentation::Components));
        let mut correction = Correction {
            pose: own,
            rotation: Quaternion([0.0; 4]),
            translation: [0.0; 3],
            origin: None,
        };
        assert!(rig.validate_node(&[], &[correction.clone()]).is_err());
        correction.rotation = Quaternion::default();
        rig.validate_node(&[], &[correction.clone()]).unwrap();
        let foreign = Poses::default().add(convention(PoseRepresentation::Components));
        correction.pose = foreign;
        assert!(rig.validate_node(&[], &[correction]).is_err());
    }
}
