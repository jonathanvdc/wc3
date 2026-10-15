//! Native flags wire codes and flags.
use bitfield::bitfield;

bitfield! {
    /// Native ModelFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct ModelFlags(u32);
    /// Exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the texture combiners bit.
    pub texture_combiners, set_texture_combiners: 3;
    /// Returns or changes the external physics bit.
    pub external_physics, set_external_physics: 5;
    /// Returns or changes the particle multi texture bit.
    pub particle_multi_texture, set_particle_multi_texture: 9;
    /// Returns or changes the chunked animations bit.
    pub chunked_animations, set_chunked_animations: 21;
    /// Uses skin section levels to extend the vertex index base.
    pub global_skin_vertex_offsets, set_global_skin_vertex_offsets: 13;
    /// Returns or changes the tilt x bit.
    pub tilt_x, set_tilt_x: 0;
    /// Returns or changes the tilt y bit.
    pub tilt_y, set_tilt_y: 1;
    /// Returns or changes the camera related bit.
    pub camera_related, set_camera_related: 8;
    /// Returns or changes the texture transform bone sequences bit.
    pub texture_transform_bone_sequences, set_texture_transform_bone_sequences: 11;
}
impl ModelFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for ModelFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<ModelFlags> for u32 {
    fn from(flags: ModelFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native SkeletonFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct SkeletonFlags(u32);
    /// Exact stored bits.
    pub bits, _: 31, 0;
}
impl SkeletonFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for SkeletonFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<SkeletonFlags> for u32 {
    fn from(flags: SkeletonFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native LodFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct LodFlags(u16);
    /// Exact stored bits.
    pub bits, _: 15, 0;
    /// Returns or changes the fixed scale bit.
    pub fixed_scale, set_fixed_scale: 3;
}
impl LodFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<u16> for LodFlags {
    fn from(bits: u16) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<LodFlags> for u16 {
    fn from(flags: LodFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native DetailLightFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct DetailLightFlags(u16);
    /// Exact stored bits.
    pub bits, _: 15, 0;
}
impl DetailLightFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<u16> for DetailLightFlags {
    fn from(bits: u16) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<DetailLightFlags> for u16 {
    fn from(flags: DetailLightFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native PivotFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct PivotFlags(u32);
    /// Exact stored bits.
    pub bits, _: 31,0;
}
impl PivotFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}

bitfield! {
    /// Native WaterfallFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct WaterfallFlags(u16);
    /// Exact stored bits.
    pub bits, _: 15,0;
}
impl WaterfallFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}

bitfield! {
    /// Native CollisionFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct CollisionFlags(u16);
    /// Exact stored bits.
    pub bits, _: 15,0;
}
impl CollisionFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
