//! Native particles wire codes and flags.
use super::*;

/// Native ParticleBlend code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u16)]
pub enum ParticleBlend {
    #[default]
    /// Draws without blending.
    #[m2(value = 0)]
    Opaque,
    /// Discards fragments below the alpha-test threshold.
    #[m2(value = 1)]
    AlphaKey,
    /// Blends particle color using source alpha.
    #[m2(value = 2)]
    Alpha,
    /// Adds particle color without multiplying by source alpha.
    #[m2(value = 3)]
    NoAlphaAdd,
    /// Adds particle color weighted by source alpha.
    #[m2(value = 4)]
    Add,
    /// Multiplies the destination by particle color.
    #[m2(value = 5)]
    Modulate,
    /// Doubles the modulation of destination and particle color.
    #[m2(value = 6)]
    Modulate2x,
    /// Uses premultiplied source color with inverse-alpha destination blending.
    #[m2(value = 7)]
    BlendAdd,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u16),
}

/// Native PackedParticleBlend code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u8)]
pub enum PackedParticleBlend {
    #[default]
    /// Draws without blending.
    #[m2(value = 0)]
    Opaque,
    /// Discards fragments below the alpha-test threshold.
    #[m2(value = 1)]
    AlphaKey,
    /// Blends particle color using source alpha.
    #[m2(value = 2)]
    Alpha,
    /// Adds particle color without multiplying by source alpha.
    #[m2(value = 3)]
    NoAlphaAdd,
    /// Adds particle color weighted by source alpha.
    #[m2(value = 4)]
    Add,
    /// Multiplies the destination by particle color.
    #[m2(value = 5)]
    Modulate,
    /// Doubles the modulation of destination and particle color.
    #[m2(value = 6)]
    Modulate2x,
    /// Uses premultiplied source color with inverse-alpha destination blending.
    #[m2(value = 7)]
    BlendAdd,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u8),
}

/// Native ParticleGenerator code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u16)]
pub enum ParticleGenerator {
    /// Spawns particles across a planar emitter area.
    #[m2(value = 1)]
    Plane,
    /// Spawns particles using a spherical emitter.
    #[m2(value = 2)]
    Sphere,
    /// Spawns particles using a spline generator.
    #[m2(value = 3)]
    Spline,
    /// Spawns particles using a bone generator.
    #[m2(value = 4)]
    Bone,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u16),
}

/// Native PackedParticleGenerator code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u8)]
pub enum PackedParticleGenerator {
    /// Spawns particles across a planar emitter area.
    #[m2(value = 1)]
    Plane,
    /// Spawns particles using a spherical emitter.
    #[m2(value = 2)]
    Sphere,
    /// Spawns particles using a spline generator.
    #[m2(value = 3)]
    Spline,
    /// Spawns particles using a bone generator.
    #[m2(value = 4)]
    Bone,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u8),
}

/// Native ParticleHeadTail code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u8)]
pub enum ParticleHeadTail {
    #[default]
    /// Draws the particle head.
    #[m2(value = 0)]
    Head,
    /// Draws the particle trail.
    #[m2(value = 1)]
    Tail,
    /// Draws both the head and trail.
    #[m2(value = 2)]
    Both,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u8),
}

bitfield! {
    /// Native ParticleFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout, m2::Validate, m2::VisitArrays)]
    pub struct ParticleFlags(u32);
    /// Exact stored bits, including unresolved flags.
    pub bits, _: 31, 0;
    /// Selects the unlit particle shader rather than the lit shader.
    pub disable_lighting, set_disable_lighting: 0;
    /// Orders living particles far to near in each view.
    pub sort_particles, set_sort_particles: 1;
    /// Aligns particle heads with their projected velocity.
    pub velocity_oriented, set_velocity_oriented: 2;
    /// Disables fog for particle rendering.
    pub unfogged, set_unfogged: 3;
    /// Keeps particle simulation local to the moving emitter frame.
    pub attached, set_attached: 4;
    /// Multiplies rendered particle size by the inherited bone scale.
    pub inherit_bone_scale, set_inherit_bone_scale: 5;
    /// Adds inherited emitter velocity to new particles.
    pub inherit_velocity, set_inherit_velocity: 6;
    /// Discards sphere particles moving away from the emitter center.
    pub implosion_filter, set_implosion_filter: 7;
    /// Directs sphere-generator initial velocity along its positive Z axis.
    pub sphere_up, set_sphere_up: 8;
    /// Negates particle spin for one random seed parity.
    pub negate_spin_random, set_negate_spin_random: 9;
    /// Limits the tail time interval to the particle age.
    pub clamp_tail_to_age, set_clamp_tail_to_age: 10;
    /// Aligns the particle quad with the emitter XY plane.
    pub align_xy, set_align_xy: 12;
    /// Requests particle position projection onto the ground.
    pub ground_clamp, set_ground_clamp: 13;
    /// Applies the emitter position delta to existing particles.
    pub follow_position, set_follow_position: 14;
    /// Selects authored keyed burst emission behavior.
    pub squirt, set_squirt: 15;
    /// Selects a random particle texture frame.
    pub random_texture, set_random_texture: 16;
    /// Enables particle head geometry.
    pub head, set_head: 17;
    /// Enables particle tail geometry.
    pub tail, set_tail: 18;
    /// Uses independent random X and Y scale variation.
    pub independent_scale, set_independent_scale: 19;
    /// Selects the scene-refraction particle programme when multi-texture is inactive.
    pub refraction, set_refraction: 20;
    /// Randomizes the starting flipbook frame.
    pub random_flipbook, set_random_flipbook: 21;
    /// Stores gravity keys as signed direction components and a signed magnitude.
    pub packed_gravity, set_packed_gravity: 23;
    /// Selects the bone variant of the bone/joint generator.
    pub bone_generator, set_bone_generator: 24;
    /// Retains the native distance-throttling bypass request.
    pub no_throttle, set_no_throttle: 26;
    /// Offsets the particle head along the spin basis.
    pub offset_head_by_spin, set_offset_head_by_spin: 27;
    /// Uses three packed five-bit texture identifiers and multi-texture composition.
    pub multi_texture, set_multi_texture: 28;
    /// Selects fourfold alpha gain in the multi-texture programme.
    pub multi_texture_modulate4x, set_multi_texture_modulate4x: 29;
    /// Selects three color inputs in the multi-texture programme.
    pub multi_texture_three_colors, set_multi_texture_three_colors: 30;
}
impl ParticleFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for ParticleFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<ParticleFlags> for u32 {
    fn from(flags: ParticleFlags) -> Self {
        flags.bits()
    }
}

impl Default for ParticleGenerator {
    fn default() -> Self {
        Self::Unknown(0)
    }
}

#[cfg(test)]
mod tests {
    use super::{PackedParticleBlend, ParticleBlend, ParticleFlags};
    use crate::model::values::model::ParticleFlags as SemanticFlags;
    use crate::model::values::model::{
        PackedParticleBlend as SemanticPackedBlend, ParticleBlend as SemanticBlend,
    };

    #[test]
    fn particle_blend_codes_match_the_native_eight_mode_table() {
        for (code, native, packed, semantic, semantic_packed) in [
            (
                0,
                ParticleBlend::Opaque,
                PackedParticleBlend::Opaque,
                SemanticBlend::Opaque,
                SemanticPackedBlend::Opaque,
            ),
            (
                1,
                ParticleBlend::AlphaKey,
                PackedParticleBlend::AlphaKey,
                SemanticBlend::AlphaKey,
                SemanticPackedBlend::AlphaKey,
            ),
            (
                2,
                ParticleBlend::Alpha,
                PackedParticleBlend::Alpha,
                SemanticBlend::Alpha,
                SemanticPackedBlend::Alpha,
            ),
            (
                3,
                ParticleBlend::NoAlphaAdd,
                PackedParticleBlend::NoAlphaAdd,
                SemanticBlend::NoAlphaAdd,
                SemanticPackedBlend::NoAlphaAdd,
            ),
            (
                4,
                ParticleBlend::Add,
                PackedParticleBlend::Add,
                SemanticBlend::Add,
                SemanticPackedBlend::Add,
            ),
            (
                5,
                ParticleBlend::Modulate,
                PackedParticleBlend::Modulate,
                SemanticBlend::Modulate,
                SemanticPackedBlend::Modulate,
            ),
            (
                6,
                ParticleBlend::Modulate2x,
                PackedParticleBlend::Modulate2x,
                SemanticBlend::Modulate2x,
                SemanticPackedBlend::Modulate2x,
            ),
            (
                7,
                ParticleBlend::BlendAdd,
                PackedParticleBlend::BlendAdd,
                SemanticBlend::BlendAdd,
                SemanticPackedBlend::BlendAdd,
            ),
        ] {
            assert_eq!(native.raw(), code);
            assert_eq!(semantic.raw(), code);
            assert_eq!(packed.raw(), code as u8);
            assert_eq!(semantic_packed.raw(), code as u8);
            assert_eq!(ParticleBlend::from_raw(code), native);
            assert_eq!(SemanticBlend::from_raw(code), semantic);
            assert_eq!(PackedParticleBlend::from_raw(code as u8), packed);
            assert_eq!(SemanticPackedBlend::from_raw(code as u8), semantic_packed);
        }
        assert_eq!(ParticleBlend::from_raw(500).raw(), 500);
        assert_eq!(PackedParticleBlend::from_raw(255).raw(), 255);
    }

    #[test]
    fn particle_control_names_preserve_native_bits_across_semantic_boundary() {
        macro_rules! check {
            ($read:ident, $write:ident, $bit:literal) => {{
                let mut native = ParticleFlags::from_bits_retain(0);
                let mut semantic = SemanticFlags::from_bits_retain(0);
                native.$write(true);
                semantic.$write(true);
                assert!(native.$read());
                assert!(semantic.$read());
                assert_eq!(native.bits(), 1 << $bit);
                assert_eq!(semantic.bits(), native.bits());
            }};
        }
        check!(disable_lighting, set_disable_lighting, 0);
        check!(sort_particles, set_sort_particles, 1);
        check!(velocity_oriented, set_velocity_oriented, 2);
        check!(unfogged, set_unfogged, 3);
        check!(refraction, set_refraction, 20);
        check!(attached, set_attached, 4);
        check!(inherit_bone_scale, set_inherit_bone_scale, 5);
        check!(inherit_velocity, set_inherit_velocity, 6);
        check!(implosion_filter, set_implosion_filter, 7);
        check!(sphere_up, set_sphere_up, 8);
        check!(negate_spin_random, set_negate_spin_random, 9);
        check!(clamp_tail_to_age, set_clamp_tail_to_age, 10);
        check!(head, set_head, 17);
        check!(tail, set_tail, 18);
        check!(packed_gravity, set_packed_gravity, 23);
        check!(multi_texture, set_multi_texture, 28);
    }

    #[test]
    fn unrelated_unknown_bits_survive_corrected_control_edits() {
        let raw = 0x8000_000a;
        let mut native = ParticleFlags::from_bits_retain(raw);
        let mut semantic = SemanticFlags::from_bits_retain(raw);
        native.set_attached(true);
        semantic.set_attached(true);
        assert_eq!(native.bits(), raw | 0x10);
        assert_eq!(semantic.bits(), native.bits());
        native.set_attached(false);
        semantic.set_attached(false);
        assert_eq!(native.bits(), raw);
        assert_eq!(semantic.bits(), raw);
    }
}

impl Default for PackedParticleGenerator {
    fn default() -> Self {
        Self::Unknown(0)
    }
}
wire_ule!(ParticleBlend, u16, raw, from_raw);
wire_ule!(PackedParticleBlend, u8, raw, from_raw);
wire_ule!(ParticleGenerator, u16, raw, from_raw);
wire_ule!(PackedParticleGenerator, u8, raw, from_raw);
wire_ule!(ParticleHeadTail, u8, raw, from_raw);
wire_ule!(ParticleFlags, u32, bits, from_bits_retain);
