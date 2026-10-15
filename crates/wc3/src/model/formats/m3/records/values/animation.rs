//! Native animation selectors and bit masks.
use crate::model::formats::m3;
use bitfield::bitfield;

/// Native Interpolation selector.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
    m3::Value,
)]
#[m3(value = u16)]
pub enum Interpolation {
    /// Selects Constant.
    #[m3(value = 0)]
    Constant,
    /// Selects Linear.
    #[m3(value = 1)]
    Linear,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u16),
}
impl Default for Interpolation {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// Native AnimationFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct AnimationFlags(u16);
    /// Returns the complete stored bit mask.
    pub bits, _: 15, 0;
}
impl AnimationFlags {
    /// Aggregate header word used for valid keyed animation references.
    pub const VALID: Self = Self(6);
}
impl From<u16> for AnimationFlags {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<AnimationFlags> for u16 {
    fn from(value: AnimationFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native SequenceFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct SequenceFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the not looping control.
    pub not_looping, set_not_looping: 0;
    /// Returns or changes the always global control.
    pub always_global, set_always_global: 1;
    /// Returns or changes the global in previewer control.
    pub global_in_previewer, set_global_in_previewer: 3;
}
impl From<u32> for SequenceFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<SequenceFlags> for u32 {
    fn from(value: SequenceFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native BoneFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct BoneFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the inherit translation control.
    pub inherit_translation, set_inherit_translation: 0;
    /// Returns or changes the inherit scale control.
    pub inherit_scale, set_inherit_scale: 1;
    /// Returns or changes the inherit rotation control.
    pub inherit_rotation, set_inherit_rotation: 2;
    /// Returns or changes the billboard1 control.
    pub billboard1, set_billboard1: 4;
    /// Returns or changes the billboard2 control.
    pub billboard2, set_billboard2: 6;
    /// Returns or changes the project 2d control.
    pub project_2d, set_project_2d: 8;
    /// Returns or changes the animated control.
    pub animated, set_animated: 9;
    /// Returns or changes the ik control.
    pub ik, set_ik: 10;
    /// Returns or changes the skinned control.
    pub skinned, set_skinned: 11;
    /// Returns or changes the real control.
    pub real, set_real: 13;
    /// Returns or changes the batch1 control.
    pub batch1, set_batch1: 14;
    /// Returns or changes the batch2 control.
    pub batch2, set_batch2: 15;
}
impl From<u32> for BoneFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<BoneFlags> for u32 {
    fn from(value: BoneFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native BoneSetFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct BoneSetFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
}
impl From<u32> for BoneSetFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<BoneSetFlags> for u32 {
    fn from(value: BoneSetFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native AnimationBlockFlags control mask; every unknown bit is preserved.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct AnimationBlockFlags(u32);
    /// Returns the complete encoded mask.
    pub bits, _: 31, 0;
}
impl From<u32> for AnimationBlockFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<AnimationBlockFlags> for u32 {
    fn from(value: AnimationBlockFlags) -> Self {
        value.bits()
    }
}
