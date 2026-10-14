//! Node-local skin authoring and functional rig classifications.
use super::{
    Correction, EnvelopeFalloff, JointAxis, JointMotion, NodePose, Poses, RigLimb, RigPlant,
    RigRole, RigSide, RigSource,
};
use crate::model::semantic::{Error, Link, NodeId, ValidateLinks};

/// Influence envelope described by inner and outer endpoint pairs.
#[derive(Clone, Debug, Default)]
pub struct Envelope {
    /// Inner endpoint at the start of the influence envelope.
    pub inner_start: f32,
    /// Inner endpoint at the end of the influence envelope.
    pub inner_end: f32,
    /// Outer endpoint at the start of the influence envelope.
    pub outer_start: f32,
    /// Outer endpoint at the end of the influence envelope.
    pub outer_end: f32,
    /// Shape of the influence falloff between the boundaries.
    pub falloff: EnvelopeFalloff,
}

/// Motion limits used by skinning authoring tools.
#[derive(Clone, Debug, Default)]
pub struct Joint {
    /// Bending, twisting, spinning, or sliding motion.
    pub motion: JointMotion,
    /// Axis along which the motion is constrained.
    pub axis: JointAxis,
    /// Interprets axes in the parent's frame instead of the model frame.
    pub parent_space: bool,
    /// Lower limit, in degrees for angular motion or model units for sliding.
    pub low: f32,
    /// Upper limit, using the same units as the lower limit.
    pub high: f32,
}

/// Editable skin setup attached to its actual node.
#[derive(Clone, Debug, Default)]
pub struct SkinSetup {
    /// Protects the recorded setup from automatic authoring adjustments.
    pub locked: bool,
    /// Optional influence envelope.
    pub envelope: Option<Envelope>,
    /// Optional joint motion constraint.
    pub joint: Option<Joint>,
    /// Node paired with this node for mirrored editing.
    pub mirror: Option<NodeId>,
    /// Corrections associated with actual saved pose identities.
    pub corrections: Vec<Correction>,
}

/// Functional classification and motion-following controls of one rig node.
#[derive(Clone, Debug, Default)]
pub struct Rig {
    /// Functional role in the authoring rig.
    pub role: RigRole,
    /// Body side to which this node belongs.
    pub side: RigSide,
    /// Limb category used by joint authoring controls.
    pub limb: RigLimb,
    /// Additional motion source outside the parent chain.
    pub rides_with: Option<NodeId>,
    /// Position-holding policy for limb ends.
    pub plant: RigPlant,
    /// Fraction of a limb end's twist assigned to this node.
    pub twist_share: f32,
    /// Origin of the functional classification.
    pub source: RigSource,
    /// Origin of the additional motion-source assignment.
    pub rides_source: RigSource,
}

/// Saved transforms and authoring setup belonging to one node.
#[derive(Clone, Debug, Default)]
pub struct NodeSetup {
    /// Saved transforms keyed by rig-owned pose identities.
    pub poses: Vec<NodePose>,
    /// Optional skin-editing setup.
    pub skin: Option<SkinSetup>,
    /// Optional functional rig classification.
    pub rig: Option<Rig>,
}
impl NodeSetup {
    /// Checks the contiguous saved-value vocabularies required by a native editable model.
    ///
    /// General graphs may use sparse pose identities. Native pose arrays independently
    /// contain prefixes of the component, matrix, and correction conventions.
    pub(crate) fn validate_native(&self, poses: &Poses<'_>) -> Result<(), Error> {
        self.validate(poses)?;
        let mut component_gap = false;
        let mut matrix_gap = false;
        let mut correction_gap = false;
        let mut origin_presence = None;
        for (id, _) in poses.iter() {
            let value = self.poses.iter().find(|value| value.pose == id);
            let components = value.is_some_and(|value| value.components.is_some());
            let matrix = value.is_some_and(|value| value.matrix.is_some());
            if components && component_gap {
                return Err(Error::new(
                    "wem.node.poses",
                    "component values require a contiguous prefix of saved conventions",
                ));
            }
            if matrix && matrix_gap {
                return Err(Error::new(
                    "wem.node.poses",
                    "matrix values require a contiguous prefix of saved conventions",
                ));
            }
            component_gap |= !components;
            matrix_gap |= !matrix;
            let correction = self
                .skin
                .as_ref()
                .and_then(|skin| skin.corrections.iter().find(|value| value.pose == id));
            if let Some(correction) = correction {
                if correction_gap {
                    return Err(Error::new(
                        "wem.node.skin.corrections",
                        "corrections require a contiguous prefix of saved conventions",
                    ));
                }
                let present = correction.origin.is_some();
                if origin_presence.is_some_and(|previous| previous != present) {
                    return Err(Error::new(
                        "wem.node.skin.corrections",
                        "correction origins must be declared for every correction or for none",
                    ));
                }
                origin_presence = Some(present);
            } else {
                correction_gap = true;
            }
        }
        Ok(())
    }
    /// Validates saved identities and all scalar authoring controls.
    pub fn validate(&self, poses: &Poses<'_>) -> Result<(), Error> {
        let corrections = self
            .skin
            .as_ref()
            .map_or(&[][..], |skin| skin.corrections.as_slice());
        poses.validate_node(&self.poses, corrections)?;
        if let Some(skin) = &self.skin {
            if skin.envelope.as_ref().is_some_and(|envelope| {
                [
                    envelope.inner_start,
                    envelope.inner_end,
                    envelope.outer_start,
                    envelope.outer_end,
                ]
                .iter()
                .any(|value| !value.is_finite())
            }) {
                return Err(Error::new(
                    "wem.node.skin.envelope",
                    "nonfinite influence envelope",
                ));
            }
            if skin
                .joint
                .as_ref()
                .is_some_and(|joint| !joint.low.is_finite() || !joint.high.is_finite())
            {
                return Err(Error::new("wem.node.skin.joint", "nonfinite joint limit"));
            }
        }
        if self
            .rig
            .as_ref()
            .is_some_and(|rig| !rig.twist_share.is_finite())
        {
            return Err(Error::new(
                "wem.node.rig.twist_share",
                "nonfinite twist share",
            ));
        }
        Ok(())
    }
}
impl ValidateLinks for NodeSetup {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Some(skin) = &self.skin {
            skin.mirror.visit_links(visitor);
        }
        if let Some(rig) = &self.rig {
            rig.rides_with.visit_links(visitor);
        }
    }
}
