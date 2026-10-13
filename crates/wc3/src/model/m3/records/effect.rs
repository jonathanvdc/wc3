//! Native effect record revisions.
use super::dialect::*;
use super::values::{
    EmitterShape, LocalForces, ParticleAdditionalFlags, ParticleFlags, ParticleInstance,
    ParticleRotationFlags, RibbonAdditionalFlags, RibbonFlags, RibbonKind, Smoothing, Variation,
    VelocityKind, WorldForces,
};
use super::{targets, AnimationValue};
use crate::model::m3::{self, Record, Ref};
use core::fmt::Debug;
/// Native field selections for independent `ParticleEmitter` section revisions.
pub trait ParticleEmitterDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `additional_flags`; `()` omits the field.
    type AdditionalFlags: NativeField;
    /// Field representation for `legacy_speed_randomize`; `()` omits the field.
    type LegacySpeedRandomize: NativeField;
    /// Field representation for `legacy_lifespan_randomize`; `()` omits the field.
    type LegacyLifespanRandomize: NativeField;
    /// Field representation for `legacy_size_animation`; `()` omits the field.
    type LegacySizeAnimation: NativeField;
    /// Field representation for `size_mid_time`; `()` omits the field.
    type SizeMidTime: NativeField;
    /// Field representation for `legacy_size_mid_time`; `()` omits the field.
    type LegacySizeMidTime: NativeField;
    /// Field representation for `color_mid_time`; `()` omits the field.
    type ColorMidTime: NativeField;
    /// Field representation for `legacy_rotation_animation`; `()` omits the field.
    type LegacyRotationAnimation: NativeField;
    /// Field representation for `alpha_mid_time`; `()` omits the field.
    type AlphaMidTime: NativeField;
    /// Field representation for `legacy_rotation_mid_time`; `()` omits the field.
    type LegacyRotationMidTime: NativeField;
    /// Field representation for `rotation_mid_time`; `()` omits the field.
    type RotationMidTime: NativeField;
    /// Field representation for `legacy_color_start`; `()` omits the field.
    type LegacyColorStart: NativeField;
    /// Field representation for `size_mid_hold_time`; `()` omits the field.
    type SizeMidHoldTime: NativeField;
    /// Field representation for `legacy_color_mid`; `()` omits the field.
    type LegacyColorMid: NativeField;
    /// Field representation for `color_mid_hold_time`; `()` omits the field.
    type ColorMidHoldTime: NativeField;
    /// Field representation for `legacy_color_end`; `()` omits the field.
    type LegacyColorEnd: NativeField;
    /// Field representation for `alpha_mid_hold_time`; `()` omits the field.
    type AlphaMidHoldTime: NativeField;
    /// Field representation for `legacy_color_mid_time`; `()` omits the field.
    type LegacyColorMidTime: NativeField;
    /// Field representation for `rotation_mid_hold_time`; `()` omits the field.
    type RotationMidHoldTime: NativeField;
    /// Field representation for `legacy_alpha_mid_time`; `()` omits the field.
    type LegacyAlphaMidTime: NativeField;
    /// Field representation for `size_animation`; `()` omits the field.
    type SizeAnimation: NativeField;
    /// Field representation for `rotation_animation`; `()` omits the field.
    type RotationAnimation: NativeField;
    /// Field representation for `color_start`; `()` omits the field.
    type ColorStart: NativeField;
    /// Field representation for `color_mid`; `()` omits the field.
    type ColorMid: NativeField;
    /// Field representation for `color_end`; `()` omits the field.
    type ColorEnd: NativeField;
    /// Field representation for `legacy_mass_randomize`; `()` omits the field.
    type LegacyMassRandomize: NativeField;
    /// Field representation for `mass_size_multiplier`; `()` omits the field.
    type MassSizeMultiplier: NativeField;
    /// Field representation for `legacy_world_space`; `()` omits the field.
    type LegacyWorldSpace: NativeField;
    /// Field representation for `world_forces_mass_multiplier`; `()` omits the field.
    type WorldForcesMassMultiplier: NativeField;
    /// Field representation for `deprecated_noise_smoothness`; `()` omits the field.
    type DeprecatedNoiseSmoothness: NativeField;
    /// Field representation for `index_plus_length`; `()` omits the field.
    type IndexPlusLength: NativeField;
    /// Field representation for `shape_regions`; `()` omits the field.
    type ShapeRegions: NativeField;
    /// Field representation for `flipbook_column_fraction`; `()` omits the field.
    type FlipbookColumnFraction: NativeField;
    /// Field representation for `flipbook_row_fraction`; `()` omits the field.
    type FlipbookRowFraction: NativeField;
    /// Field representation for `instance_distance`; `()` omits the field.
    type InstanceDistance: NativeField;
    /// Field representation for `phase_shift`; `()` omits the field.
    type PhaseShift: NativeField;
    /// Field representation for `rotation_flags`; `()` omits the field.
    type RotationFlags: NativeField;
    /// Field representation for `color_smoothing`; `()` omits the field.
    type ColorSmoothing: NativeField;
    /// Field representation for `size_smoothing`; `()` omits the field.
    type SizeSmoothing: NativeField;
    /// Field representation for `rotation_smoothing`; `()` omits the field.
    type RotationSmoothing: NativeField;
    /// Field representation for `alpha_threshold`; `()` omits the field.
    type AlphaThreshold: NativeField;
    /// Field representation for `uv_offset`; `()` omits the field.
    type UvOffset: NativeField;
    /// Field representation for `uv_angle`; `()` omits the field.
    type UvAngle: NativeField;
    /// Field representation for `uv_tiling`; `()` omits the field.
    type UvTiling: NativeField;
    /// Field representation for `spawn_ribbon_on_bounce_chance`; `()` omits the field.
    type SpawnRibbonOnBounceChance: NativeField;
    /// Field representation for `ribbon_link_index`; `()` omits the field.
    type RibbonLinkIndex: NativeField;
}

impl ParticleEmitterDialect for V10 {
    const VERSION: u32 = 10;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = AnimationValue<[f32; 3]>;
    type SizeMidTime = ();
    type LegacySizeMidTime = f32;
    type ColorMidTime = ();
    type LegacyRotationAnimation = AnimationValue<[f32; 3]>;
    type AlphaMidTime = ();
    type LegacyRotationMidTime = f32;
    type RotationMidTime = ();
    type LegacyColorStart = AnimationValue<FixedBytes<4>>;
    type SizeMidHoldTime = ();
    type LegacyColorMid = AnimationValue<FixedBytes<4>>;
    type ColorMidHoldTime = ();
    type LegacyColorEnd = AnimationValue<FixedBytes<4>>;
    type AlphaMidHoldTime = ();
    type LegacyColorMidTime = f32;
    type RotationMidHoldTime = ();
    type LegacyAlphaMidTime = f32;
    type SizeAnimation = ();
    type RotationAnimation = ();
    type ColorStart = ();
    type ColorMid = ();
    type ColorEnd = ();
    type LegacyMassRandomize = u32;
    type MassSizeMultiplier = ();
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = f32;
    type IndexPlusLength = ();
    type ShapeRegions = ();
    type FlipbookColumnFraction = ();
    type FlipbookRowFraction = ();
    type InstanceDistance = ();
    type PhaseShift = ();
    type RotationFlags = ();
    type ColorSmoothing = ();
    type SizeSmoothing = ();
    type RotationSmoothing = ();
    type AlphaThreshold = ();
    type UvOffset = ();
    type UvAngle = ();
    type UvTiling = ();
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V11 {
    const VERSION: u32 = 11;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = AnimationValue<[f32; 3]>;
    type SizeMidTime = ();
    type LegacySizeMidTime = f32;
    type ColorMidTime = ();
    type LegacyRotationAnimation = AnimationValue<[f32; 3]>;
    type AlphaMidTime = ();
    type LegacyRotationMidTime = f32;
    type RotationMidTime = ();
    type LegacyColorStart = AnimationValue<FixedBytes<4>>;
    type SizeMidHoldTime = ();
    type LegacyColorMid = AnimationValue<FixedBytes<4>>;
    type ColorMidHoldTime = ();
    type LegacyColorEnd = AnimationValue<FixedBytes<4>>;
    type AlphaMidHoldTime = ();
    type LegacyColorMidTime = f32;
    type RotationMidHoldTime = ();
    type LegacyAlphaMidTime = f32;
    type SizeAnimation = ();
    type RotationAnimation = ();
    type ColorStart = ();
    type ColorMid = ();
    type ColorEnd = ();
    type LegacyMassRandomize = u32;
    type MassSizeMultiplier = ();
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = f32;
    type IndexPlusLength = u32;
    type ShapeRegions = ();
    type FlipbookColumnFraction = ();
    type FlipbookRowFraction = ();
    type InstanceDistance = ();
    type PhaseShift = ();
    type RotationFlags = ();
    type ColorSmoothing = ();
    type SizeSmoothing = ();
    type RotationSmoothing = ();
    type AlphaThreshold = ();
    type UvOffset = ();
    type UvAngle = ();
    type UvTiling = ();
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V12 {
    const VERSION: u32 = 12;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = ();
    type LegacyColorMid = ();
    type ColorMidHoldTime = ();
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = ();
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = ();
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = u32;
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = ();
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = ();
    type PhaseShift = ();
    type RotationFlags = ();
    type ColorSmoothing = ();
    type SizeSmoothing = ();
    type RotationSmoothing = ();
    type AlphaThreshold = ();
    type UvOffset = ();
    type UvAngle = ();
    type UvTiling = ();
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V13 {
    const VERSION: u32 = 13;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = u32;
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = ();
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = ();
    type PhaseShift = ();
    type RotationFlags = ();
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = ();
    type UvOffset = ();
    type UvAngle = ();
    type UvTiling = ();
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V14 {
    const VERSION: u32 = 14;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = u32;
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = ();
    type PhaseShift = ();
    type RotationFlags = ();
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = ();
    type UvOffset = ();
    type UvAngle = ();
    type UvTiling = ();
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V15 {
    const VERSION: u32 = 15;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = u32;
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = ();
    type PhaseShift = ();
    type RotationFlags = ();
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V16 {
    const VERSION: u32 = 16;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = u32;
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = f32;
    type PhaseShift = ();
    type RotationFlags = ();
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V17 {
    const VERSION: u32 = 17;
    type AdditionalFlags = ParticleAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = f32;
    type PhaseShift = ();
    type RotationFlags = ();
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V18 {
    const VERSION: u32 = 18;
    type AdditionalFlags = ParticleAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = f32;
    type PhaseShift = ();
    type RotationFlags = ParticleRotationFlags;
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V19 {
    const VERSION: u32 = 19;
    type AdditionalFlags = ParticleAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = f32;
    type PhaseShift = ();
    type RotationFlags = ParticleRotationFlags;
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V20 {
    const VERSION: u32 = 20;
    type AdditionalFlags = ParticleAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = f32;
    type PhaseShift = ();
    type RotationFlags = ParticleRotationFlags;
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V21 {
    const VERSION: u32 = 21;
    type AdditionalFlags = ParticleAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = f32;
    type PhaseShift = ();
    type RotationFlags = ParticleRotationFlags;
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V22 {
    const VERSION: u32 = 22;
    type AdditionalFlags = ParticleAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = f32;
    type PhaseShift = AnimationValue<f32>;
    type RotationFlags = ParticleRotationFlags;
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = ();
    type RibbonLinkIndex = ();
}

impl ParticleEmitterDialect for V23 {
    const VERSION: u32 = 23;
    type AdditionalFlags = ParticleAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = ();
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = f32;
    type PhaseShift = AnimationValue<f32>;
    type RotationFlags = ParticleRotationFlags;
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = f32;
    type RibbonLinkIndex = i32;
}

impl ParticleEmitterDialect for V24 {
    const VERSION: u32 = 24;
    type AdditionalFlags = ParticleAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type MassSizeMultiplier = f32;
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = f32;
    type DeprecatedNoiseSmoothness = ();
    type IndexPlusLength = u32;
    type ShapeRegions = Ref<targets::U32>;
    type FlipbookColumnFraction = f32;
    type FlipbookRowFraction = f32;
    type InstanceDistance = f32;
    type PhaseShift = AnimationValue<f32>;
    type RotationFlags = ParticleRotationFlags;
    type ColorSmoothing = Smoothing;
    type SizeSmoothing = Smoothing;
    type RotationSmoothing = Smoothing;
    type AlphaThreshold = AnimationValue<f32>;
    type UvOffset = AnimationValue<[f32; 2]>;
    type UvAngle = AnimationValue<[f32; 3]>;
    type UvTiling = AnimationValue<[f32; 2]>;
    type SpawnRibbonOnBounceChance = f32;
    type RibbonLinkIndex = i32;
}

/// Fields of the native `PAR_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ParticleEmitter<D: ParticleEmitterDialect> {
    /// Selects the BONE entry to which this emitter is attached.
    pub bone_index: u32,
    /// Selects the MATM entry supplying this effect’s material.
    pub material_index: u32,
    /// The meaning of the additional flags storage has not been established; its encoded bits are preserved.
    pub additional_flags: D::AdditionalFlags,
    /// Sets initial particle speed.
    pub initial_speed: AnimationValue<f32>,
    /// Sets random speed variation.
    pub initial_speed_random: AnimationValue<f32>,
    /// Controls speed randomization with a separate legacy boolean rather than a later flag bit.
    pub legacy_speed_randomize: D::LegacySpeedRandomize,
    /// Sets initial yaw angle.
    pub initial_yaw: AnimationValue<f32>,
    /// Sets initial pitch angle.
    pub initial_pitch: AnimationValue<f32>,
    /// Sets initial horizontal spread.
    pub initial_horizontal: AnimationValue<f32>,
    /// Sets initial vertical spread.
    pub initial_vertical: AnimationValue<f32>,
    /// Sets base particle lifetime.
    pub lifetime: AnimationValue<f32>,
    /// Sets random lifetime variation.
    pub lifetime_random: AnimationValue<f32>,
    /// Controls lifespan randomization with a separate legacy boolean rather than a later flag bit.
    pub legacy_lifespan_randomize: D::LegacyLifespanRandomize,
    /// Sets the radius beyond which particles are removed.
    pub kill_radius: f32,
    /// Stores the X component of gravity; reference data normally uses zero.
    pub gravity_x: u32,
    /// Stores the Y component of gravity; reference data normally uses zero.
    pub gravity_y: u32,
    /// Sets gravity Z component.
    pub gravity: f32,
    /// Supplies size values at the beginning, middle, and end of the lifetime.
    pub legacy_size_animation: D::LegacySizeAnimation,
    /// Places the size curve’s middle key at a normalized lifetime fraction from zero to one.
    pub size_mid_time: D::SizeMidTime,
    /// Places the size curve’s middle key at a normalized lifetime fraction from zero to one.
    pub legacy_size_mid_time: D::LegacySizeMidTime,
    /// Places the color curve’s middle key at a normalized lifetime fraction from zero to one.
    pub color_mid_time: D::ColorMidTime,
    /// Supplies rotation values at the beginning, middle, and end of the lifetime.
    pub legacy_rotation_animation: D::LegacyRotationAnimation,
    /// Places the alpha curve’s middle key at a normalized lifetime fraction from zero to one.
    pub alpha_mid_time: D::AlphaMidTime,
    /// Places the rotation curve’s middle key at a normalized lifetime fraction from zero to one.
    pub legacy_rotation_mid_time: D::LegacyRotationMidTime,
    /// Places the rotation curve’s middle key at a normalized lifetime fraction from zero to one.
    pub rotation_mid_time: D::RotationMidTime,
    /// Supplies the color when an instance is created.
    pub legacy_color_start: D::LegacyColorStart,
    /// Sets how long the size curve retains its midpoint value. This field is present in revision 14 onward.
    pub size_mid_hold_time: D::SizeMidHoldTime,
    /// Supplies the color at the lifetime curve’s middle key.
    pub legacy_color_mid: D::LegacyColorMid,
    /// Sets how long the color curve retains its midpoint value. This field is present in revision 14 onward.
    pub color_mid_hold_time: D::ColorMidHoldTime,
    /// Supplies the color at the end of the lifetime.
    pub legacy_color_end: D::LegacyColorEnd,
    /// Sets how long the alpha curve retains its midpoint value. This field is present in revision 14 onward.
    pub alpha_mid_hold_time: D::AlphaMidHoldTime,
    /// Places the color curve’s middle key at a normalized lifetime fraction from zero to one.
    pub legacy_color_mid_time: D::LegacyColorMidTime,
    /// Sets how long the rotation curve retains its midpoint value. This field is present in revision 14 onward.
    pub rotation_mid_hold_time: D::RotationMidHoldTime,
    /// Places the alpha curve’s middle key at a normalized lifetime fraction from zero to one.
    pub legacy_alpha_mid_time: D::LegacyAlphaMidTime,
    /// Supplies size values at the beginning, middle, and end of the lifetime.
    pub size_animation: D::SizeAnimation,
    /// Supplies rotation values at the beginning, middle, and end of the lifetime.
    pub rotation_animation: D::RotationAnimation,
    /// Supplies the color when an instance is created.
    pub color_start: D::ColorStart,
    /// Supplies the color at the lifetime curve’s middle key.
    pub color_mid: D::ColorMid,
    /// Supplies the color at the end of the lifetime.
    pub color_end: D::ColorEnd,
    /// Sets air drag coefficient.
    pub drag: f32,
    /// Controls mass randomization with a separate legacy boolean rather than a later flag bit.
    pub legacy_mass_randomize: D::LegacyMassRandomize,
    /// Sets particle mass.
    pub mass: f32,
    /// Sets random mass variation multiplier.
    pub mass_random: f32,
    /// Sets mass–size coupling. This field is present in revision 12 onward.
    pub mass_size_multiplier: D::MassSizeMultiplier,
    /// Controls world space with a separate legacy boolean rather than a later flag bit.
    pub legacy_world_space: D::LegacyWorldSpace,
    /// Selects local force channel through individual bits.
    pub local_forces: LocalForces,
    /// Selects world force channel through individual bits.
    pub world_forces: WorldForces,
    /// Sets fallback local force channels.
    pub local_forces_fallback: LocalForces,
    /// Sets fallback world force channels.
    pub world_forces_fallback: WorldForces,
    /// Sets world force mass multiplier. This field is present in revision 24 onward.
    pub world_forces_mass_multiplier: D::WorldForcesMassMultiplier,
    /// Sets noise displacement amplitude.
    pub noise_amplitude: f32,
    /// Sets noise spatial frequency.
    pub noise_frequency: f32,
    /// Sets noise temporal coherence.
    pub noise_coherence: f32,
    /// Sets noise edge sharpness.
    pub noise_edge: f32,
    /// Adjusts noise smoothing in legacy layouts through revision 11.
    pub deprecated_noise_smoothness: D::DeprecatedNoiseSmoothness,
    /// Stores a combined index-and-length value from revision 11; its component encoding is unresolved.
    pub index_plus_length: D::IndexPlusLength,
    /// Sets maximum live particle count.
    pub max_particles: u32,
    /// Controls emission rate (particles/sec) through its animation value.
    pub emission_rate: AnimationValue<f32>,
    /// Sets emission shape.
    pub emitter_shape: EmitterShape,
    /// Controls outer shape dimensions through its animation value.
    pub shape_outer: AnimationValue<[f32; 3]>,
    /// Controls inner shape dimensions through its animation value.
    pub shape_inner: AnimationValue<[f32; 3]>,
    /// Controls outer radius through its animation value.
    pub outer_radius: AnimationValue<f32>,
    /// Controls inner radius through its animation value.
    pub inner_radius: AnimationValue<f32>,
    /// Links to U32_ region indices selecting emission geometry from a division, starting at revision 14.
    pub shape_regions: D::ShapeRegions,
    /// Selects the behavior used for velocity randomization type.
    pub velocity_type: VelocityKind,
    /// Enables random variation of size.
    pub size_random_enable: u32,
    /// Sets random size curve.
    pub size_random_animation: AnimationValue<[f32; 3]>,
    /// Enables random variation of rotation.
    pub rotation_random_enable: u32,
    /// Sets random rotation curve.
    pub rotation_random_animation: AnimationValue<[f32; 3]>,
    /// Enables random variation of color.
    pub color_random_enable: u32,
    /// Supplies the randomized color endpoint used at birth.
    pub color_start_random: AnimationValue<[u8; 4]>,
    /// Supplies the randomized color endpoint used at midpoint.
    pub color_mid_random: AnimationValue<[u8; 4]>,
    /// Supplies the randomized color endpoint used at death.
    pub color_end_random: AnimationValue<[u8; 4]>,
    /// Enables random variation of alpha.
    pub alpha_random_enable: u32,
    /// Controls squirt burst count through its animation value.
    pub squirt_amount: AnimationValue<u16>,
    /// Sets flipbook start initial frame index.
    pub flipbook_start_init_index: u8,
    /// Sets flipbook start stop frame index.
    pub flipbook_start_stop_index: u8,
    /// Sets flipbook end initial frame index.
    pub flipbook_end_init_index: u8,
    /// Sets flipbook end stop frame index.
    pub flipbook_end_stop_index: u8,
    /// Locates the flipbook curve’s midpoint within the lifetime, normalized from zero to one.
    pub flipbook_mid_time: f32,
    /// Sets flipbook grid columns.
    pub flipbook_columns: u16,
    /// Sets flipbook grid rows.
    pub flipbook_rows: u16,
    /// Sets column fraction. This field is present in revision 12 onward.
    pub flipbook_column_fraction: D::FlipbookColumnFraction,
    /// Sets row fraction. This field is present in revision 12 onward.
    pub flipbook_row_fraction: D::FlipbookRowFraction,
    /// Sets bounce coefficient.
    pub bounce: f32,
    /// Sets friction coefficient.
    pub friction: f32,
    /// Selects emitter index to spawn on collision; all bits set indicates no linked entry.
    pub collision_spawn_index: i32,
    /// Sets minimum spawn count on collision.
    pub collision_spawn_min: u32,
    /// Sets maximum spawn count on collision.
    pub collision_spawn_max: u32,
    /// Sets spawn probability on collision.
    pub collision_spawn_chance: f32,
    /// Sets spawn energy transfer.
    pub collision_spawn_energy: f32,
    /// Sets die after N bounces.
    pub collision_die_bounce: u32,
    /// Selects the particle representation passed to the shader as b_iInstanceType.
    pub instance_type: ParticleInstance,
    /// Sets tail length for Tail/Trail types.
    pub tail_length: f32,
    /// Sets instance orientation angles.
    pub instance_angle: [f32; 3],
    /// Sets instance distance. This field is present in revision 17 onward.
    pub instance_distance: D::InstanceDistance,
    /// Selects the function used to vary pitch.
    pub pitch_type: Variation,
    /// Sets the magnitude of changes applied to pitch.
    pub pitch_amplitude: AnimationValue<f32>,
    /// Sets how frequently pitch varies.
    pub pitch_frequency: AnimationValue<f32>,
    /// Selects the function used to vary yaw.
    pub yaw_type: Variation,
    /// Sets the magnitude of changes applied to yaw.
    pub yaw_amplitude: AnimationValue<f32>,
    /// Sets how frequently yaw varies.
    pub yaw_frequency: AnimationValue<f32>,
    /// Selects the function used to vary speed.
    pub speed_type: Variation,
    /// Sets the magnitude of changes applied to speed.
    pub speed_amplitude: AnimationValue<f32>,
    /// Sets how frequently speed varies.
    pub speed_frequency: AnimationValue<f32>,
    /// Selects the function used to vary size.
    pub size_type: Variation,
    /// Sets the magnitude of changes applied to size.
    pub size_amplitude: AnimationValue<f32>,
    /// Sets how frequently size varies.
    pub size_frequency: AnimationValue<f32>,
    /// Selects the function used to vary alpha.
    pub alpha_type: Variation,
    /// Sets the magnitude of changes applied to alpha.
    pub alpha_amplitude: AnimationValue<f32>,
    /// Sets how frequently alpha varies.
    pub alpha_frequency: AnimationValue<f32>,
    /// Selects the function used to vary color.
    pub color_type: Variation,
    /// Sets the magnitude of changes applied to color.
    pub color_amplitude: AnimationValue<f32>,
    /// Sets how frequently color varies.
    pub color_frequency: AnimationValue<f32>,
    /// Selects the function used to vary rotation.
    pub rotation_type: Variation,
    /// Sets the magnitude of changes applied to rotation.
    pub rotation_amplitude: AnimationValue<f32>,
    /// Sets how frequently rotation varies.
    pub rotation_frequency: AnimationValue<f32>,
    /// Selects the function used to vary horizontal.
    pub horizontal_type: Variation,
    /// Sets the magnitude of changes applied to horizontal.
    pub horizontal_amplitude: AnimationValue<f32>,
    /// Sets how frequently horizontal varies.
    pub horizontal_frequency: AnimationValue<f32>,
    /// Selects the function used to vary vertical.
    pub vertical_type: Variation,
    /// Sets the magnitude of changes applied to vertical.
    pub vertical_amplitude: AnimationValue<f32>,
    /// Sets how frequently the vertical emission spread varies.
    pub vertical_frequency: AnimationValue<f32>,
    /// Controls parent velocity influence through its animation value.
    pub particle_velocity: AnimationValue<f32>,
    /// Controls phase shift through its animation value. This field is present in revision 22 onward.
    pub phase_shift: D::PhaseShift,
    /// Stores the bit mask governing main particle.
    pub flags: ParticleFlags,
    /// Stores the bit mask governing rotation. This field is present in revision 18 onward.
    pub rotation_flags: D::RotationFlags,
    /// The meaning of the color smoothing storage has not been established; its encoded bits are preserved.
    pub color_smoothing: D::ColorSmoothing,
    /// The meaning of the size smoothing storage has not been established; its encoded bits are preserved.
    pub size_smoothing: D::SizeSmoothing,
    /// The meaning of the rotation smoothing storage has not been established; its encoded bits are preserved.
    pub rotation_smoothing: D::RotationSmoothing,
    /// Controls alpha threshold through its animation value.
    pub alpha_threshold: D::AlphaThreshold,
    /// Controls UV offset through its animation value.
    pub uv_offset: D::UvOffset,
    /// Controls UV rotation angles through its animation value.
    pub uv_angle: D::UvAngle,
    /// Controls UV tiling through its animation value.
    pub uv_tiling: D::UvTiling,
    /// Sets spline control points (SVC3).
    pub spline_line_data: Ref<targets::AnimationValueVector3>,
    /// Sets wind influence multiplier.
    pub wind_multiplier: f32,
    /// Sets LOD reduction level.
    pub lod_reduce: u32,
    /// Sets LOD cut-off level.
    pub lod_cut: u32,
    /// Controls lower bound through its animation value.
    pub lower_bound: AnimationValue<f32>,
    /// Controls upper bound through its animation value.
    pub upper_bound: AnimationValue<f32>,
    /// The meaning of the trail link index storage has not been established; its encoded bits are preserved.
    pub trail_link_index: i32,
    /// Sets trail spawn probability.
    pub trail_chance: f32,
    /// Controls trail emission rate through its animation value.
    pub trail_emission_rate: AnimationValue<f32>,
    /// Selects linked projector index; all bits set indicates no linked entry.
    pub splat_projection_index: i32,
    /// Sets splat spawn probability.
    pub splat_chance: f32,
    /// Sets model particle paths (SCHR).
    pub model_paths: Ref<targets::StringReference>,
    /// Sets emitter copy indices (U32_).
    pub copy_indices: Ref<targets::U32>,
    /// Sets ribbon spawn probability on bounce. This field is present in revision 23 onward.
    pub spawn_ribbon_on_bounce_chance: D::SpawnRibbonOnBounceChance,
    /// Selects index into rib_ array; all bits set indicates no linked entry.
    pub ribbon_link_index: D::RibbonLinkIndex,
}

impl<D: ParticleEmitterDialect> Record for ParticleEmitter<D> {
    const TAG: [u8; 4] = *b"PAR_";
    const VERSION: u32 = D::VERSION;
}

/// Fields of the native `PARC` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ParticleEmitterCopy {
    /// Sets overridden emission rate.
    pub emission_rate: AnimationValue<f32>,
    /// Sets overridden squirt burst count.
    pub squirt_amount: AnimationValue<u16>,
    /// Selects the BONE entry to which this emitter is attached.
    pub bone_index: u32,
}

impl Record for ParticleEmitterCopy {
    const TAG: [u8; 4] = *b"PARC";
    const VERSION: u32 = 0;
}

/// Fields of the native `SRIB` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct SplineRibbon {
    /// Sets emission point offset from bone.
    pub emission_offset: [f32; 3],
    /// Sets emission direction vector.
    pub emission_vector: [f32; 3],
    /// Controls base velocity through its animation value.
    pub velocity: AnimationValue<f32>,
    /// Reserved storage; reference samples use zero.
    pub reserved: u32,
    /// Selects the BONE entry to which this emitter is attached.
    pub bone_index: u32,
    /// Controls base velocity factor through its animation value.
    pub velocity_base_factor: AnimationValue<f32>,
    /// Controls end velocity factor through its animation value.
    pub velocity_end_factor: AnimationValue<f32>,
    /// Selects the function used to vary yaw.
    pub yaw_type: Variation,
    /// Sets the magnitude of changes applied to yaw.
    pub yaw_amplitude: AnimationValue<f32>,
    /// Sets how frequently yaw varies.
    pub yaw_frequency: AnimationValue<f32>,
    /// Selects the function used to vary pitch.
    pub pitch_type: Variation,
    /// Sets the magnitude of changes applied to pitch.
    pub pitch_amplitude: AnimationValue<f32>,
    /// Sets how frequently pitch varies.
    pub pitch_frequency: AnimationValue<f32>,
    /// Selects the function used to vary velocity.
    pub velocity_type: Variation,
    /// Sets the magnitude of changes applied to velocity.
    pub velocity_amplitude: AnimationValue<f32>,
    /// Sets how frequently velocity varies.
    pub velocity_frequency: AnimationValue<f32>,
    /// Controls yaw angle through its animation value.
    pub yaw: AnimationValue<f32>,
    /// Controls pitch angle through its animation value.
    pub pitch: AnimationValue<f32>,
    /// Stores a cached normalization factor approximately equal to 0.01 divided by the emission vector’s magnitude.
    pub emission_vector_norm_factor: f32,
    /// Stores a cached normalization factor approximately equal to 0.01 divided by the initial velocity.
    pub velocity_norm_factor: f32,
}

impl Record for SplineRibbon {
    const TAG: [u8; 4] = *b"SRIB";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `RibbonEmitter` section revisions.
pub trait RibbonEmitterDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `additional_flags`; `()` omits the field.
    type AdditionalFlags: NativeField;
    /// Field representation for `legacy_speed_randomize`; `()` omits the field.
    type LegacySpeedRandomize: NativeField;
    /// Field representation for `legacy_initial_pitch`; `()` omits the field.
    type LegacyInitialPitch: NativeField;
    /// Field representation for `initial_yaw`; `()` omits the field.
    type InitialYaw: NativeField;
    /// Field representation for `legacy_initial_yaw`; `()` omits the field.
    type LegacyInitialYaw: NativeField;
    /// Field representation for `initial_pitch`; `()` omits the field.
    type InitialPitch: NativeField;
    /// Field representation for `legacy_lifespan_randomize`; `()` omits the field.
    type LegacyLifespanRandomize: NativeField;
    /// Field representation for `legacy_size_animation`; `()` omits the field.
    type LegacySizeAnimation: NativeField;
    /// Field representation for `size_mid_time`; `()` omits the field.
    type SizeMidTime: NativeField;
    /// Field representation for `legacy_size_mid_time`; `()` omits the field.
    type LegacySizeMidTime: NativeField;
    /// Field representation for `color_mid_time`; `()` omits the field.
    type ColorMidTime: NativeField;
    /// Field representation for `legacy_rotation_animation`; `()` omits the field.
    type LegacyRotationAnimation: NativeField;
    /// Field representation for `alpha_mid_time`; `()` omits the field.
    type AlphaMidTime: NativeField;
    /// Field representation for `legacy_rotation_mid_time`; `()` omits the field.
    type LegacyRotationMidTime: NativeField;
    /// Field representation for `rotation_mid_time`; `()` omits the field.
    type RotationMidTime: NativeField;
    /// Field representation for `legacy_color_start`; `()` omits the field.
    type LegacyColorStart: NativeField;
    /// Field representation for `size_mid_hold_time`; `()` omits the field.
    type SizeMidHoldTime: NativeField;
    /// Field representation for `legacy_color_mid`; `()` omits the field.
    type LegacyColorMid: NativeField;
    /// Field representation for `color_mid_hold_time`; `()` omits the field.
    type ColorMidHoldTime: NativeField;
    /// Field representation for `legacy_color_end`; `()` omits the field.
    type LegacyColorEnd: NativeField;
    /// Field representation for `alpha_mid_hold_time`; `()` omits the field.
    type AlphaMidHoldTime: NativeField;
    /// Field representation for `legacy_color_mid_time`; `()` omits the field.
    type LegacyColorMidTime: NativeField;
    /// Field representation for `rotation_mid_hold_time`; `()` omits the field.
    type RotationMidHoldTime: NativeField;
    /// Field representation for `legacy_alpha_mid_time`; `()` omits the field.
    type LegacyAlphaMidTime: NativeField;
    /// Field representation for `size_animation`; `()` omits the field.
    type SizeAnimation: NativeField;
    /// Field representation for `rotation_animation`; `()` omits the field.
    type RotationAnimation: NativeField;
    /// Field representation for `color_start`; `()` omits the field.
    type ColorStart: NativeField;
    /// Field representation for `color_mid`; `()` omits the field.
    type ColorMid: NativeField;
    /// Field representation for `color_end`; `()` omits the field.
    type ColorEnd: NativeField;
    /// Field representation for `legacy_mass_randomize`; `()` omits the field.
    type LegacyMassRandomize: NativeField;
    /// Field representation for `legacy_world_space`; `()` omits the field.
    type LegacyWorldSpace: NativeField;
    /// Field representation for `world_forces_mass_multiplier`; `()` omits the field.
    type WorldForcesMassMultiplier: NativeField;
    /// Field representation for `index_plus_length`; `()` omits the field.
    type IndexPlusLength: NativeField;
    /// Field representation for `deprecated_unknown3fbae7d6`; `()` omits the field.
    type DeprecatedUnknown3fbae7d6: NativeField;
    /// Field representation for `size_smoothing`; `()` omits the field.
    type SizeSmoothing: NativeField;
    /// Field representation for `color_smoothing`; `()` omits the field.
    type ColorSmoothing: NativeField;
    /// Field representation for `legacy_pitch_type`; `()` omits the field.
    type LegacyPitchType: NativeField;
    /// Field representation for `yaw_type`; `()` omits the field.
    type YawType: NativeField;
    /// Field representation for `legacy_pitch_amplitude`; `()` omits the field.
    type LegacyPitchAmplitude: NativeField;
    /// Field representation for `yaw_amplitude`; `()` omits the field.
    type YawAmplitude: NativeField;
    /// Field representation for `legacy_pitch_frequency`; `()` omits the field.
    type LegacyPitchFrequency: NativeField;
    /// Field representation for `yaw_frequency`; `()` omits the field.
    type YawFrequency: NativeField;
    /// Field representation for `legacy_yaw_type`; `()` omits the field.
    type LegacyYawType: NativeField;
    /// Field representation for `pitch_type`; `()` omits the field.
    type PitchType: NativeField;
    /// Field representation for `legacy_yaw_amplitude`; `()` omits the field.
    type LegacyYawAmplitude: NativeField;
    /// Field representation for `pitch_amplitude`; `()` omits the field.
    type PitchAmplitude: NativeField;
    /// Field representation for `legacy_yaw_frequency`; `()` omits the field.
    type LegacyYawFrequency: NativeField;
    /// Field representation for `pitch_frequency`; `()` omits the field.
    type PitchFrequency: NativeField;
}

impl RibbonEmitterDialect for V4 {
    const VERSION: u32 = 4;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyInitialPitch = AnimationValue<f32>;
    type InitialYaw = ();
    type LegacyInitialYaw = AnimationValue<f32>;
    type InitialPitch = ();
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = AnimationValue<[f32; 3]>;
    type SizeMidTime = ();
    type LegacySizeMidTime = f32;
    type ColorMidTime = ();
    type LegacyRotationAnimation = AnimationValue<[f32; 3]>;
    type AlphaMidTime = ();
    type LegacyRotationMidTime = f32;
    type RotationMidTime = ();
    type LegacyColorStart = AnimationValue<FixedBytes<4>>;
    type SizeMidHoldTime = ();
    type LegacyColorMid = AnimationValue<FixedBytes<4>>;
    type ColorMidHoldTime = ();
    type LegacyColorEnd = AnimationValue<FixedBytes<4>>;
    type AlphaMidHoldTime = ();
    type LegacyColorMidTime = f32;
    type RotationMidHoldTime = ();
    type LegacyAlphaMidTime = f32;
    type SizeAnimation = ();
    type RotationAnimation = ();
    type ColorStart = ();
    type ColorMid = ();
    type ColorEnd = ();
    type LegacyMassRandomize = u32;
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type IndexPlusLength = ();
    type DeprecatedUnknown3fbae7d6 = i32;
    type SizeSmoothing = ();
    type ColorSmoothing = ();
    type LegacyPitchType = Variation;
    type YawType = ();
    type LegacyPitchAmplitude = AnimationValue<f32>;
    type YawAmplitude = ();
    type LegacyPitchFrequency = AnimationValue<f32>;
    type YawFrequency = ();
    type LegacyYawType = Variation;
    type PitchType = ();
    type LegacyYawAmplitude = AnimationValue<f32>;
    type PitchAmplitude = ();
    type LegacyYawFrequency = AnimationValue<f32>;
    type PitchFrequency = ();
}

impl RibbonEmitterDialect for V5 {
    const VERSION: u32 = 5;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyInitialPitch = AnimationValue<f32>;
    type InitialYaw = ();
    type LegacyInitialYaw = AnimationValue<f32>;
    type InitialPitch = ();
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = AnimationValue<[f32; 3]>;
    type SizeMidTime = ();
    type LegacySizeMidTime = f32;
    type ColorMidTime = ();
    type LegacyRotationAnimation = AnimationValue<[f32; 3]>;
    type AlphaMidTime = ();
    type LegacyRotationMidTime = f32;
    type RotationMidTime = ();
    type LegacyColorStart = AnimationValue<FixedBytes<4>>;
    type SizeMidHoldTime = ();
    type LegacyColorMid = AnimationValue<FixedBytes<4>>;
    type ColorMidHoldTime = ();
    type LegacyColorEnd = AnimationValue<FixedBytes<4>>;
    type AlphaMidHoldTime = ();
    type LegacyColorMidTime = f32;
    type RotationMidHoldTime = ();
    type LegacyAlphaMidTime = f32;
    type SizeAnimation = ();
    type RotationAnimation = ();
    type ColorStart = ();
    type ColorMid = ();
    type ColorEnd = ();
    type LegacyMassRandomize = u32;
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type IndexPlusLength = u32;
    type DeprecatedUnknown3fbae7d6 = i32;
    type SizeSmoothing = ();
    type ColorSmoothing = ();
    type LegacyPitchType = Variation;
    type YawType = ();
    type LegacyPitchAmplitude = AnimationValue<f32>;
    type YawAmplitude = ();
    type LegacyPitchFrequency = AnimationValue<f32>;
    type YawFrequency = ();
    type LegacyYawType = Variation;
    type PitchType = ();
    type LegacyYawAmplitude = AnimationValue<f32>;
    type PitchAmplitude = ();
    type LegacyYawFrequency = AnimationValue<f32>;
    type PitchFrequency = ();
}

impl RibbonEmitterDialect for V6 {
    const VERSION: u32 = 6;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyInitialPitch = AnimationValue<f32>;
    type InitialYaw = ();
    type LegacyInitialYaw = AnimationValue<f32>;
    type InitialPitch = ();
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = ();
    type LegacyColorMid = ();
    type ColorMidHoldTime = ();
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = ();
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = ();
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = u32;
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type IndexPlusLength = u32;
    type DeprecatedUnknown3fbae7d6 = i32;
    type SizeSmoothing = ();
    type ColorSmoothing = ();
    type LegacyPitchType = Variation;
    type YawType = ();
    type LegacyPitchAmplitude = AnimationValue<f32>;
    type YawAmplitude = ();
    type LegacyPitchFrequency = AnimationValue<f32>;
    type YawFrequency = ();
    type LegacyYawType = Variation;
    type PitchType = ();
    type LegacyYawAmplitude = AnimationValue<f32>;
    type PitchAmplitude = ();
    type LegacyYawFrequency = AnimationValue<f32>;
    type PitchFrequency = ();
}

impl RibbonEmitterDialect for V7 {
    const VERSION: u32 = 7;
    type AdditionalFlags = ();
    type LegacySpeedRandomize = u32;
    type LegacyInitialPitch = ();
    type InitialYaw = AnimationValue<f32>;
    type LegacyInitialYaw = ();
    type InitialPitch = AnimationValue<f32>;
    type LegacyLifespanRandomize = u32;
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = u32;
    type LegacyWorldSpace = u32;
    type WorldForcesMassMultiplier = ();
    type IndexPlusLength = u32;
    type DeprecatedUnknown3fbae7d6 = ();
    type SizeSmoothing = Smoothing;
    type ColorSmoothing = Smoothing;
    type LegacyPitchType = ();
    type YawType = Variation;
    type LegacyPitchAmplitude = ();
    type YawAmplitude = AnimationValue<f32>;
    type LegacyPitchFrequency = ();
    type YawFrequency = AnimationValue<f32>;
    type LegacyYawType = ();
    type PitchType = Variation;
    type LegacyYawAmplitude = ();
    type PitchAmplitude = AnimationValue<f32>;
    type LegacyYawFrequency = ();
    type PitchFrequency = AnimationValue<f32>;
}

impl RibbonEmitterDialect for V8 {
    const VERSION: u32 = 8;
    type AdditionalFlags = RibbonAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyInitialPitch = ();
    type InitialYaw = AnimationValue<f32>;
    type LegacyInitialYaw = ();
    type InitialPitch = AnimationValue<f32>;
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = ();
    type IndexPlusLength = u32;
    type DeprecatedUnknown3fbae7d6 = ();
    type SizeSmoothing = Smoothing;
    type ColorSmoothing = Smoothing;
    type LegacyPitchType = ();
    type YawType = Variation;
    type LegacyPitchAmplitude = ();
    type YawAmplitude = AnimationValue<f32>;
    type LegacyPitchFrequency = ();
    type YawFrequency = AnimationValue<f32>;
    type LegacyYawType = ();
    type PitchType = Variation;
    type LegacyYawAmplitude = ();
    type PitchAmplitude = AnimationValue<f32>;
    type LegacyYawFrequency = ();
    type PitchFrequency = AnimationValue<f32>;
}

impl RibbonEmitterDialect for V9 {
    const VERSION: u32 = 9;
    type AdditionalFlags = RibbonAdditionalFlags;
    type LegacySpeedRandomize = ();
    type LegacyInitialPitch = ();
    type InitialYaw = AnimationValue<f32>;
    type LegacyInitialYaw = ();
    type InitialPitch = AnimationValue<f32>;
    type LegacyLifespanRandomize = ();
    type LegacySizeAnimation = ();
    type SizeMidTime = f32;
    type LegacySizeMidTime = ();
    type ColorMidTime = f32;
    type LegacyRotationAnimation = ();
    type AlphaMidTime = f32;
    type LegacyRotationMidTime = ();
    type RotationMidTime = f32;
    type LegacyColorStart = ();
    type SizeMidHoldTime = f32;
    type LegacyColorMid = ();
    type ColorMidHoldTime = f32;
    type LegacyColorEnd = ();
    type AlphaMidHoldTime = f32;
    type LegacyColorMidTime = ();
    type RotationMidHoldTime = f32;
    type LegacyAlphaMidTime = ();
    type SizeAnimation = AnimationValue<[f32; 3]>;
    type RotationAnimation = AnimationValue<[f32; 3]>;
    type ColorStart = AnimationValue<FixedBytes<4>>;
    type ColorMid = AnimationValue<FixedBytes<4>>;
    type ColorEnd = AnimationValue<FixedBytes<4>>;
    type LegacyMassRandomize = ();
    type LegacyWorldSpace = ();
    type WorldForcesMassMultiplier = f32;
    type IndexPlusLength = u32;
    type DeprecatedUnknown3fbae7d6 = ();
    type SizeSmoothing = Smoothing;
    type ColorSmoothing = Smoothing;
    type LegacyPitchType = ();
    type YawType = Variation;
    type LegacyPitchAmplitude = ();
    type YawAmplitude = AnimationValue<f32>;
    type LegacyPitchFrequency = ();
    type YawFrequency = AnimationValue<f32>;
    type LegacyYawType = ();
    type PitchType = Variation;
    type LegacyYawAmplitude = ();
    type PitchAmplitude = AnimationValue<f32>;
    type LegacyYawFrequency = ();
    type PitchFrequency = AnimationValue<f32>;
}

/// Fields of the native `RIB_` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct RibbonEmitter<D: RibbonEmitterDialect> {
    /// Sets primary bone index.
    pub bone_index: u16,
    /// Sets fallback bone index.
    pub bone_index_fallback: u16,
    /// Selects the MATM entry supplying this effect’s material.
    pub material_index: u32,
    /// Stores the bit mask governing additional. This field is present in revision 8 onward.
    pub additional_flags: D::AdditionalFlags,
    /// Sets initial ribbon segment speed.
    pub initial_speed: AnimationValue<f32>,
    /// Sets random speed variation.
    pub initial_speed_random: AnimationValue<f32>,
    /// Controls speed randomization with a separate legacy boolean rather than a later flag bit.
    pub legacy_speed_randomize: D::LegacySpeedRandomize,
    /// Sets initial pitch angle.
    pub legacy_initial_pitch: D::LegacyInitialPitch,
    /// Sets initial yaw angle.
    pub initial_yaw: D::InitialYaw,
    /// Sets initial yaw angle.
    pub legacy_initial_yaw: D::LegacyInitialYaw,
    /// Sets initial pitch angle.
    pub initial_pitch: D::InitialPitch,
    /// Sets initial horizontal spread.
    pub initial_horizontal: AnimationValue<f32>,
    /// Sets initial vertical spread.
    pub initial_vertical: AnimationValue<f32>,
    /// Sets base segment lifetime.
    pub lifetime: AnimationValue<f32>,
    /// Sets random lifetime variation.
    pub lifetime_random: AnimationValue<f32>,
    /// Controls lifespan randomization with a separate legacy boolean rather than a later flag bit.
    pub legacy_lifespan_randomize: D::LegacyLifespanRandomize,
    /// Sets the radius beyond which particles are removed.
    pub kill_radius: u32,
    /// Sets gravity X component.
    pub gravity_x: f32,
    /// Sets gravity Y component.
    pub gravity_y: f32,
    /// Sets gravity Z component.
    pub gravity: f32,
    /// Supplies size values at the beginning, middle, and end of the lifetime.
    pub legacy_size_animation: D::LegacySizeAnimation,
    /// Locates the size curve’s midpoint within the lifetime, normalized from zero to one.
    pub size_mid_time: D::SizeMidTime,
    /// Locates the size curve’s midpoint within the lifetime, normalized from zero to one.
    pub legacy_size_mid_time: D::LegacySizeMidTime,
    /// Locates the color curve’s midpoint within the lifetime, normalized from zero to one.
    pub color_mid_time: D::ColorMidTime,
    /// Supplies rotation values at the beginning, middle, and end of the lifetime.
    pub legacy_rotation_animation: D::LegacyRotationAnimation,
    /// Locates the alpha curve’s midpoint within the lifetime, normalized from zero to one.
    pub alpha_mid_time: D::AlphaMidTime,
    /// Locates the rotation curve’s midpoint within the lifetime, normalized from zero to one.
    pub legacy_rotation_mid_time: D::LegacyRotationMidTime,
    /// Locates the rotation curve’s midpoint within the lifetime, normalized from zero to one.
    pub rotation_mid_time: D::RotationMidTime,
    /// Supplies the color when an instance is created.
    pub legacy_color_start: D::LegacyColorStart,
    /// Sets how long the size curve retains its midpoint value.
    pub size_mid_hold_time: D::SizeMidHoldTime,
    /// Supplies the color at the lifetime curve’s middle key.
    pub legacy_color_mid: D::LegacyColorMid,
    /// Sets how long the color curve retains its midpoint value.
    pub color_mid_hold_time: D::ColorMidHoldTime,
    /// Supplies the color at the end of the lifetime.
    pub legacy_color_end: D::LegacyColorEnd,
    /// Sets how long the alpha curve retains its midpoint value.
    pub alpha_mid_hold_time: D::AlphaMidHoldTime,
    /// Locates the color curve’s midpoint within the lifetime, normalized from zero to one.
    pub legacy_color_mid_time: D::LegacyColorMidTime,
    /// Sets how long the rotation curve retains its midpoint value.
    pub rotation_mid_hold_time: D::RotationMidHoldTime,
    /// Locates the alpha curve’s midpoint within the lifetime, normalized from zero to one.
    pub legacy_alpha_mid_time: D::LegacyAlphaMidTime,
    /// Supplies size values at the beginning, middle, and end of the lifetime.
    pub size_animation: D::SizeAnimation,
    /// Supplies rotation values at the beginning, middle, and end of the lifetime.
    pub rotation_animation: D::RotationAnimation,
    /// Supplies the color when an instance is created.
    pub color_start: D::ColorStart,
    /// Supplies the color at the lifetime curve’s middle key.
    pub color_mid: D::ColorMid,
    /// Supplies the color at the end of the lifetime.
    pub color_end: D::ColorEnd,
    /// Sets air drag coefficient.
    pub drag: f32,
    /// Controls mass randomization with a separate legacy boolean rather than a later flag bit.
    pub legacy_mass_randomize: D::LegacyMassRandomize,
    /// Sets segment mass.
    pub mass: f32,
    /// Sets random mass variation.
    pub mass_random: f32,
    /// Sets mass–size coupling.
    pub mass_size_multiplier: f32,
    /// Controls world space with a separate legacy boolean rather than a later flag bit.
    pub legacy_world_space: D::LegacyWorldSpace,
    /// Selects local force channel through individual bits.
    pub local_forces: LocalForces,
    /// Selects world force channel through individual bits.
    pub world_forces: WorldForces,
    /// Sets fallback local force channels.
    pub local_forces_fallback: LocalForces,
    /// Sets fallback world force channels.
    pub world_forces_fallback: WorldForces,
    /// Sets world force mass multiplier.
    pub world_forces_mass_multiplier: D::WorldForcesMassMultiplier,
    /// Sets noise displacement amplitude.
    pub noise_amplitude: f32,
    /// Sets noise spatial frequency.
    pub noise_frequency: f32,
    /// Sets noise temporal coherence.
    pub noise_coherence: f32,
    /// Sets noise edge sharpness.
    pub noise_edge: f32,
    /// Stores a combined index-and-length value whose component encoding is unresolved.
    pub index_plus_length: D::IndexPlusLength,
    /// Selects the behavior used for emitter shape type.
    pub emitter_shape: EmitterShape,
    /// Selects the behavior used for ribbon cross-section type.
    pub ribbon_type: RibbonKind,
    /// Sets the number of ribbon divisions.
    pub divisions: f32,
    /// Sets the number of cross-section edges.
    pub edges: u32,
    /// Sets inner radius.
    pub inner_radius: f32,
    /// Controls maximum ribbon length through its animation value.
    pub max_length: AnimationValue<f32>,
    /// Unknown-purpose legacy storage present through revision 6.
    pub deprecated_unknown3fbae7d6: D::DeprecatedUnknown3fbae7d6,
    /// Sets spline ribbon segments (SRIB).
    pub spline_ribbons: Ref<targets::SplineRibbon>,
    /// Controls active state through its animation value.
    pub active: AnimationValue<u32>,
    /// Stores the bit mask governing ribbon emitter.
    pub flags: RibbonFlags,
    /// Selects the behavior used for size interpolation mode.
    pub size_smoothing: D::SizeSmoothing,
    /// Selects the behavior used for color interpolation mode.
    pub color_smoothing: D::ColorSmoothing,
    /// Sets friction coefficient.
    pub friction: f32,
    /// Sets bounce coefficient.
    pub bounce: f32,
    /// Sets LOD reduction level.
    pub lod_reduce: u32,
    /// Sets LOD cut-off level.
    pub lod_cut: u32,
    /// Selects the function used to vary pitch.
    pub legacy_pitch_type: D::LegacyPitchType,
    /// Selects the function used to vary yaw.
    pub yaw_type: D::YawType,
    /// Sets the magnitude of changes applied to pitch.
    pub legacy_pitch_amplitude: D::LegacyPitchAmplitude,
    /// Sets the magnitude of changes applied to yaw.
    pub yaw_amplitude: D::YawAmplitude,
    /// Sets how frequently pitch varies.
    pub legacy_pitch_frequency: D::LegacyPitchFrequency,
    /// Sets how frequently yaw varies.
    pub yaw_frequency: D::YawFrequency,
    /// Selects the function used to vary yaw.
    pub legacy_yaw_type: D::LegacyYawType,
    /// Selects the function used to vary pitch.
    pub pitch_type: D::PitchType,
    /// Sets the magnitude of changes applied to yaw.
    pub legacy_yaw_amplitude: D::LegacyYawAmplitude,
    /// Sets the magnitude of changes applied to pitch.
    pub pitch_amplitude: D::PitchAmplitude,
    /// Sets how frequently yaw varies.
    pub legacy_yaw_frequency: D::LegacyYawFrequency,
    /// Sets how frequently pitch varies.
    pub pitch_frequency: D::PitchFrequency,
    /// Selects the function used to vary speed.
    pub speed_type: Variation,
    /// Sets the magnitude of changes applied to speed.
    pub speed_amplitude: AnimationValue<f32>,
    /// Sets how frequently speed varies.
    pub speed_frequency: AnimationValue<f32>,
    /// Selects the function used to vary size.
    pub size_type: Variation,
    /// Sets the magnitude of changes applied to size.
    pub size_amplitude: AnimationValue<f32>,
    /// Sets how frequently size varies.
    pub size_frequency: AnimationValue<f32>,
    /// Selects the function used to vary alpha.
    pub alpha_type: Variation,
    /// Sets the magnitude of changes applied to alpha.
    pub alpha_amplitude: AnimationValue<f32>,
    /// Sets how frequently alpha varies.
    pub alpha_frequency: AnimationValue<f32>,
    /// Controls parent velocity influence through its animation value.
    pub particle_velocity: AnimationValue<f32>,
    /// Controls overlay effect through its animation value.
    pub overlay: AnimationValue<f32>,
}

impl<D: RibbonEmitterDialect> Record for RibbonEmitter<D> {
    const TAG: [u8; 4] = *b"RIB_";
    const VERSION: u32 = D::VERSION;
}
