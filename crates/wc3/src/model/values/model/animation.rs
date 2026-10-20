//! Native animation wire codes and flags.
use bitfield::bitfield;

mod ids;
pub use ids::AnimationId;

/// Native interpolation operation; unknown values retain their exact wire representation.
///
/// Cubic Hermite uses wire code 2; cubic Bezier uses wire code 3.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Interpolation {
    #[default]
    /// Keeps the preceding key value until the next timestamp.
    Step,
    /// Interpolates between neighboring keys along a straight segment.
    Linear,
    /// Uses incoming and outgoing Bezier control points.
    Bezier,
    /// Uses incoming and outgoing Hermite tangents.
    Hermite,
    /// Unrecognized native code.
    Unknown(u16),
}

bitfield! {
    /// Sequence storage and alias flags, including unrecognized bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct SequenceFlags(u32);
    /// Exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the alias bit.
    pub alias, set_alias: 6;
    /// Returns or changes the low-priority sequence-storage bit.
    pub load_low_priority, set_load_low_priority: 4;
    /// Returns or changes the primary bone-sequence storage bit.
    pub primary_bone_sequence, set_primary_bone_sequence: 5;
    /// Returns or changes the stored-in-model bit.
    pub stored_in_model, set_stored_in_model: 8;
}
impl SequenceFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for SequenceFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<SequenceFlags> for u32 {
    fn from(flags: SequenceFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native BoneFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct BoneFlags(u32);
    /// Exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the dont inherit translation bit.
    pub dont_inherit_translation, set_dont_inherit_translation: 0;
    /// Returns or changes the dont inherit scale bit.
    pub dont_inherit_scale, set_dont_inherit_scale: 1;
    /// Returns or changes the dont inherit rotation bit.
    pub dont_inherit_rotation, set_dont_inherit_rotation: 2;
    /// Returns or changes the billboard bit.
    pub billboard, set_billboard: 3;
    /// Returns or changes the billboard lock x bit.
    pub billboard_lock_x, set_billboard_lock_x: 4;
    /// Returns or changes the billboard lock y bit.
    pub billboard_lock_y, set_billboard_lock_y: 5;
    /// Returns or changes the billboard lock z bit.
    pub billboard_lock_z, set_billboard_lock_z: 6;
    /// Returns or changes the model space bit.
    pub model_space, set_model_space: 24;
    /// Returns or changes the transformed bit.
    pub transformed, set_transformed: 9;
    /// Returns or changes the kinematic bit.
    pub kinematic, set_kinematic: 10;
    /// Excludes the bone from body-particle generator candidates.
    pub no_body_spawn, set_no_body_spawn: 11;
    /// Returns or changes the helmet animation scaled bit.
    pub helmet_animation_scaled, set_helmet_animation_scaled: 12;
}
impl BoneFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for BoneFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<BoneFlags> for u32 {
    fn from(flags: BoneFlags) -> Self {
        flags.bits()
    }
}

impl SequenceFlags {
    /// Whether timeline data is stored in an external animation file.
    pub fn external_animation(self) -> bool {
        !(self.load_low_priority() || self.primary_bone_sequence() || self.stored_in_model())
    }
}
impl Interpolation {
    /// Whether values contain incoming and outgoing spline controls.
    pub const fn spline(self) -> bool {
        matches!(self, Self::Bezier | Self::Hermite)
    }
}

impl Interpolation {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Step,
            1 => Self::Linear,
            3 => Self::Bezier,
            2 => Self::Hermite,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Step => 0,
            Self::Linear => 1,
            Self::Bezier => 3,
            Self::Hermite => 2,
            Self::Unknown(value) => value,
        }
    }
}

bitfield! {
    /// Bone-family behavior beyond common inheritance and billboarding.
    #[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
    pub struct BoneBehaviorFlags(u32);
    /// Behavior flags, retaining unrecognized bits.
    pub bits,_:31,0;
    /// Apply animated bone transforms.
    pub transformed,set_transformed:9;
    /// Kinematic simulation participation.
    pub kinematic,set_kinematic:10;
    /// Excludes the bone from body-particle generator candidates.
    pub no_body_spawn,set_no_body_spawn:11;
    /// Scale helmet animation coordinates.
    pub helmet_animation_scaled,set_helmet_animation_scaled:12;
    /// Evaluate transforms in model coordinates.
    pub model_space,set_model_space:24;
}
impl BoneBehaviorFlags {
    /// Retains behavior and unknown bits; common hierarchy bits belong to Node.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits & !0x7f)
    }
}
