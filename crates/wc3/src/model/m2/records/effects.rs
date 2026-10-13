//! Scene effects with field types selected by native version families.
//!
//! Cameras retain spline triplets, while emitter tracks use the owning model's
//! timeline. Particle appearance has a separate normalized-age clock. Packed
//! integers and fixed-point values remain in their encoded representation; these
//! records describe file data without evaluating or rendering the effects.
use super::{
    ArrayVisitor, ParticleTrack, RibbonLayout, ScalarField, SplineTrack, Track, Validate,
    VisitArrays,
};
use super::{
    CameraKind, ModelFlags, PackedParticleBlend, PackedParticleGenerator, ParticleBlend,
    ParticleFlags, ParticleGenerator, ParticleHeadTail,
};
use crate::model::{common::Ownership, m2};
use m2::codec::{Decoder, Encoder};
use m2::ModelLayout;
use m2::{Read, ReadError, RecordLayout, Write, WriteError};
use zerovec::ZeroVec;

/// Additional ribbon draw ordering and animation lookup fields in modern layouts.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct RibbonModern {
    /// Signed drawing priority plane.
    pub priority_plane: i16,
    /// Ribbon color-table selection; retained as a signed native index.
    pub color_index: i8,
    /// Index into the core texture-transform combination table when its
    /// ribbon-transform flag is set.
    pub texture_transform_index: i8,
}
/// Legacy particle blend and generator codes, each stored in a full u16.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct WideParticleHeader {
    /// Particle blending code: 0 opaque, 1 alpha blend, 2 additive, 3 alpha test, 4
    /// additive alpha test.
    pub blending: ParticleBlend,
    /// Generator code: 1 plane, 2 sphere, 3 spline, 4 bone.
    pub emitter: ParticleGenerator,
}
/// Packed particle blend and generator bytes followed by a color-table index.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct PackedParticleHeader {
    /// Particle blending code in one byte; values 0 through 4 name the established modes.
    pub blending: PackedParticleBlend,
    /// Generator code in one byte: 1 plane, 2 sphere, 3 spline, 4 bone.
    pub emitter: PackedParticleGenerator,
    /// ParticleColor.dbc selection, retained without resolving game color tables.
    pub color_index: u16,
}
/// Legacy particle shape and head/tail selection occupying the later texture-scale slot.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct LegacyTextureScale {
    /// Legacy particle shape code; retained without interpreting undocumented values.
    pub particle_type: u8,
    /// Head/tail selection: 0 head, 1 tail, 2 both.
    pub head_or_tail: ParticleHeadTail,
}
/// Two per-layer UV scales retained in their one-byte fixed8_5 representation.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct ModernTextureScale {
    /// Per-layer UV scales as unsigned fixed8_5 bytes; divide each encoded value by 32.
    pub scales: [u8; 2],
}
/// Three-point particle-age appearance and atlas-cell data in legacy layouts.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct LegacyParticleLifetime {
    /// Middle appearance key position as a fraction of particle lifetime, normally between 0 and 1.
    pub midpoint: f32,
    /// Blue, green, red, and alpha bytes at the beginning, midpoint, and end of life.
    pub colors_bgra: [[u8; 4]; 3],
    /// Uniform particle scales at the beginning, midpoint, and end of life.
    pub scales: [f32; 3],
    /// Beginning head atlas-cell pair and its preserved third padding word.
    pub head_lifespan_uv: [u16; 3],
    /// Ending head atlas-cell pair and its preserved third padding word.
    pub head_decay_uv: [u16; 3],
    /// Beginning tail atlas-cell pair retained as signed native words.
    pub tail_lifespan_uv: [i16; 2],
    /// Ending tail atlas-cell pair retained as signed native words.
    pub tail_decay_uv: [i16; 2],
}
/// Appearance and atlas animation keyed by normalized particle age, not model sequences.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct ModernParticleLifetime<'data> {
    /// RGB appearance keys over normalized particle age; native component magnitudes are retained.
    pub color: ParticleTrack<'data, [f32; 3]>,
    /// Opacity keys over particle age, retaining native fixed16 words (0 transparent,
    /// 0x7fff opaque).
    pub alpha: ParticleTrack<'data, u16>,
    /// Two-dimensional size keys over particle age.
    pub scale: ParticleTrack<'data, [f32; 2]>,
    /// Random size variation for the two scale components.
    pub scale_variation: [f32; 2],
    /// Head flipbook-cell keys over particle age, retained as native u16 values.
    pub head_uv: ParticleTrack<'data, u16>,
    /// Tail flipbook-cell keys over particle age, retained as native u16 values.
    pub tail_uv: ParticleTrack<'data, u16>,
}
/// Initial billboard rotation and angular-speed parameters in modern layouts.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct ModernSpin {
    /// Initial billboard rotation angle in radians.
    pub base: f32,
    /// Random variation of the initial rotation angle in radians.
    pub base_variation: f32,
    /// Billboard angular velocity in radians per second.
    pub speed: f32,
    /// Random variation of angular velocity in radians per second.
    pub speed_variation: f32,
}
/// Per-layer UV scrolling centers and ranges in signed fixed16_9 storage.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct ExtendedScroll {
    /// Four layer/component UV centers followed by four ranges, each signed fixed16_9
    /// (divide by 512).
    pub values: [i16; 8],
}

/// Runtime-flag-selected scrolling fields used only by intermediate layouts.
#[derive(Clone, Copy, Debug, PartialEq, Default, Ownership)]
pub struct FlaggedScroll {
    /// Four UV centers followed by four ranges as signed fixed16_9 (divide by 512),
    /// present when core flag 0x200 is set.
    pub values: Option<[i16; 8]>,
}
impl<'data> Read<'data> for FlaggedScroll {
    fn read(i: &mut Decoder<'data>) -> Result<Self, ReadError> {
        Ok(Self {
            values: if i.flags.particle_multi_texture() {
                Some(<[i16; 8]>::read(i)?)
            } else {
                None
            },
        })
    }
}
impl Write for FlaggedScroll {
    fn write(&self, o: &mut Encoder) -> Result<(), WriteError> {
        if (self.values.is_some()) != (o.flags.particle_multi_texture()) {
            return Err(WriteError::invalid("flag-selected particle scrolling"));
        }
        if let Some(v) = self.values {
            v.write(o)?;
        }
        Ok(())
    }
}
impl RecordLayout for FlaggedScroll {
    fn size(_: u32, flags: u32) -> usize {
        if ModelFlags::from_bits_retain(flags).particle_multi_texture() {
            16
        } else {
            0
        }
    }
}
impl Validate for FlaggedScroll {
    fn validate(&self) -> Result<(), ReadError> {
        Ok(())
    }
}
impl<'data> VisitArrays<'data> for FlaggedScroll {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, _: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
/// Construction of a statically selected scrolling representation.
pub trait ScrollField: ScalarField {
    /// Creates the native representation for runtime core flags.
    fn from_flags(flags: ModelFlags) -> Self;
}
impl ScrollField for () {
    fn from_flags(_: ModelFlags) -> Self {}
}
impl ScrollField for FlaggedScroll {
    fn from_flags(flags: ModelFlags) -> Self {
        Self {
            values: (flags.particle_multi_texture()).then_some([0; 8]),
        }
    }
}
impl ScrollField for ExtendedScroll {
    fn from_flags(_: ModelFlags) -> Self {
        Self::default()
    }
}

/// Portrait, character-info, or flyby camera with native spline keys and clipping distances.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct Camera<'data, V: ModelLayout> {
    /// Camera role: 0 portrait, 1 character-info, u32::MAX flyby.
    pub kind: CameraKind,
    /// Legacy diagonal field of view in radians; `()` in layouts selecting an animated
    /// field of view.
    pub field_of_view: V::FovPrefix,
    /// Far clipping distance in model-space units.
    pub far_clip: f32,
    /// Near clipping distance in model-space units.
    pub near_clip: f32,
    /// Camera position spline keys, retaining value and incoming/outgoing tangents.
    pub positions: SplineTrack<'data, [f32; 3], V>,
    /// Base camera position in model coordinates.
    pub position_base: [f32; 3],
    /// Look-at target spline keys, retaining value and incoming/outgoing tangents.
    pub target_positions: SplineTrack<'data, [f32; 3], V>,
    /// Base look-at target in model coordinates.
    pub target_position_base: [f32; 3],
    /// Camera roll spline keys in radians.
    pub roll: SplineTrack<'data, f32, V>,
    /// Modern diagonal field-of-view spline keys in radians; `()` in legacy layouts.
    pub field_of_view_track: V::FovTrack<'data>,
}
/// A bone-attached ribbon trail with animated appearance and atlas selection.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct RibbonEmitter<'data, V: ModelLayout> {
    /// Ribbon identifier, commonly u32::MAX for an unassigned identifier.
    pub id: u32,
    /// Index of the bone carrying this ribbon.
    pub bone_id: u32,
    /// Emitter position relative to its attached bone.
    pub position: [f32; 3],
    /// Indices into the core texture table used by this ribbon.
    pub texture_indices: ZeroVec<'data, u16>,
    /// Indices into the core material table used by this ribbon.
    pub material_indices: ZeroVec<'data, u16>,
    /// Animated RGB color multiplier.
    pub color: Track<'data, [f32; 3], V>,
    /// Animated opacity as signed fixed16: 0 transparent and 0x7fff opaque.
    pub alpha: Track<'data, i16, V>,
    /// Animated distance above the trail center in model-space units.
    pub height_above: Track<'data, f32, V>,
    /// Animated distance below the trail center in model-space units.
    pub height_below: Track<'data, f32, V>,
    /// Rate at which trail edges are generated, in edges per second.
    pub edges_per_second: f32,
    /// Time each trail edge persists, in seconds.
    pub edge_lifetime: f32,
    /// Native emission-angle parameter, historically named gravity; not an acceleration value.
    pub gravity: f32,
    /// Number of rows in the ribbon texture atlas.
    pub texture_rows: u16,
    /// Number of columns in the ribbon texture atlas.
    pub texture_columns: u16,
    /// Animated atlas slot selection retained as native u16 keys.
    pub texture_slot: Track<'data, u16, V>,
    /// Animated visibility toggle, with zero disabling the ribbon.
    pub visibility: Track<'data, u8, V>,
    /// Layout-selected draw-order and lookup fields, or `()` when absent.
    pub modern: <V as RibbonLayout>::Tail,
}
/// A bone-attached particle generator with model-time emission and particle-age appearance.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct ParticleEmitter<'data, V: ModelLayout> {
    /// Particle emitter identifier, commonly u32::MAX when unassigned.
    pub id: u32,
    /// Native particle behavior flags, including 0x800000 for packed-vector gravity keys.
    pub flags: ParticleFlags,
    /// Emitter position relative to its attached bone.
    pub position: [f32; 3],
    /// Index into the core bone table.
    pub bone_id: u16,
    /// Texture index, or three packed five-bit texture indices for multi-textured
    /// particles; the high bit is retained.
    pub texture_id: u16,
    /// Native filename bytes for model-geometry particles, including any encoded terminator.
    pub particle_model_filename: ZeroVec<'data, u8>,
    /// Native filename bytes for the child-emitter model, including any encoded terminator.
    pub child_model_filename: ZeroVec<'data, u8>,
    /// Layout-selected blend and generator codes, with a color index in packed layouts.
    pub kind: V::Header,
    /// Legacy shape/head-tail bytes or modern per-layer fixed8_5 UV scales.
    pub texture_scale: V::TextureScale,
    /// Native texture-tile rotation selector, also used as a priority-plane value.
    pub texture_tile_rotation: i16,
    /// Number of rows in the particle texture atlas.
    pub rows: u16,
    /// Number of columns in the particle texture atlas.
    pub columns: u16,
    /// Animated initial particle speed in model-space units per second.
    pub emission_speed: Track<'data, f32, V>,
    /// Animated fractional variation of initial speed, normally between 0 and 1.
    pub speed_variation: Track<'data, f32, V>,
    /// Animated vertical emission spread in radians.
    pub vertical_range: Track<'data, f32, V>,
    /// Animated horizontal emission spread in radians.
    pub horizontal_range: Track<'data, f32, V>,
    /// Raw gravity words: IEEE f32 bits normally, or little-endian i8 X, i8 Y, i16
    /// magnitude under flag 0x800000.
    pub gravity: Track<'data, u32, V>,
    /// Animated particle lifetime in seconds.
    pub lifespan: Track<'data, f32, V>,
    /// Layout-selected random lifetime variation in seconds, or `()` when absent.
    pub lifespan_variation: V::Variation,
    /// Animated particle emission rate in particles per second.
    pub emission_rate: Track<'data, f32, V>,
    /// Layout-selected random variation of emission rate, or `()` when absent.
    pub emission_rate_variation: V::Variation,
    /// Animated plane Y extent or spherical maximum radius, in model-space units.
    pub emission_area_width: Track<'data, f32, V>,
    /// Animated plane X extent or spherical minimum radius, in model-space units.
    pub emission_area_length: Track<'data, f32, V>,
    /// Animated Z position of the source point used to aim initial particle velocity.
    pub z_source: Track<'data, f32, V>,
    /// Layout-selected particle-age appearance block; separate from model-time emission tracks.
    pub lifetime: V::Lifetime<'data>,
    /// Native particle-tail extent parameter; its precise time-versus-distance
    /// interpretation is unresolved.
    pub tail_length: f32,
    /// Native twinkle frequency parameter; exact client interpretation remains unverified.
    pub twinkle_speed: f32,
    /// Native twinkle visibility fraction parameter; exact client interpretation
    /// remains unverified.
    pub twinkle_percent: f32,
    /// Minimum and maximum scale parameters for the twinkle effect.
    pub twinkle_scale: [f32; 2],
    /// Multiplier for velocity inherited from the emitter parent; historically called
    /// burstMultiplier.
    pub inherit_velocity_scale: f32,
    /// Velocity damping coefficient used in exponential decay over elapsed seconds.
    pub drag: f32,
    /// Legacy lifetime rotation scalar or modern initial-angle and angular-speed group.
    pub spin: V::Spin,
    /// Minimum model-particle angular-velocity components; native angular units are retained.
    pub tumble_minimum: [f32; 3],
    /// Maximum model-particle angular-velocity components; native angular units are retained.
    pub tumble_maximum: [f32; 3],
    /// Static wind vector; particle flags can select dynamic wind instead.
    pub wind: [f32; 3],
    /// Native wind timing parameter; exact client interpretation remains unresolved.
    pub wind_time: f32,
    /// First native follow-speed parameter; its relation to follow scales remains unresolved.
    pub follow_speed_1: f32,
    /// First native follow-scale parameter, retained without inferring a response curve.
    pub follow_scale_1: f32,
    /// Second native follow-speed parameter; its relation to follow scales remains unresolved.
    pub follow_speed_2: f32,
    /// Second native follow-scale parameter, retained without inferring a response curve.
    pub follow_scale_2: f32,
    /// Model-space path points for spline particle generators.
    pub spline_points: ZeroVec<'data, [f32; 3]>,
    /// Animated emission toggle, with zero disabling emission.
    pub enabled: Track<'data, u8, V>,
    /// Layout-selected signed fixed16_9 UV centers and ranges; intermediate layouts
    /// depend on core flag 0x200.
    pub multi_texture_scroll: V::Scroll,
}
impl<'data, V: ModelLayout> Camera<'data, V> {
    /// Creates a portrait camera with empty native spline tracks.
    pub fn new() -> Self {
        Self::default()
    }
}
impl<'data, V: ModelLayout> ParticleEmitter<'data, V> {
    /// Creates an empty emitter with scrolling selected from the owning core flags.
    pub fn new(flags: ModelFlags) -> Self {
        Self {
            id: u32::MAX,
            rows: 1,
            columns: 1,
            multi_texture_scroll: V::Scroll::from_flags(flags),
            ..Self::default()
        }
    }
}
