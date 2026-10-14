//! M3 geometry selectors and bit masks.
use bitfield::bitfield;

bitfield! {
    /// M3 RegionFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct RegionFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the hidden control.
    pub hidden, set_hidden: 0;
    /// Returns or changes the placeholder control.
    pub placeholder, set_placeholder: 1;
    /// Returns or changes the cloth simulated control.
    pub cloth_simulated, set_cloth_simulated: 2;
    /// Returns or changes the cloth influenced control.
    pub cloth_influenced, set_cloth_influenced: 3;
}
impl From<u32> for RegionFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<RegionFlags> for u32 {
    fn from(value: RegionFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// M3 BatchFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct BatchFlags(u16);
    /// Returns the complete stored bit mask.
    pub bits, _: 15, 0;
}
impl From<u16> for BatchFlags {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<BatchFlags> for u16 {
    fn from(value: BatchFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// M3 VertexFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct VertexFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the position field.
    pub position, set_position: 0;
    /// Returns or changes the skin0 field.
    pub skin0, set_skin0: 5;
    /// Returns or changes the skin1 field.
    pub skin1, set_skin1: 6;
    /// Returns or changes the color field.
    pub color, set_color: 9;
    /// Returns or changes the float uv0 field.
    pub float_uv0, set_float_uv0: 13;
    /// Returns or changes the float uv1 field.
    pub float_uv1, set_float_uv1: 14;
    /// Returns or changes the float uv2 field.
    pub float_uv2, set_float_uv2: 15;
    /// Returns or changes the float uv3 field.
    pub float_uv3, set_float_uv3: 16;
    /// Returns or changes the uv0 field.
    pub uv0, set_uv0: 17;
    /// Returns or changes the uv1 field.
    pub uv1, set_uv1: 18;
    /// Returns or changes the uv2 field.
    pub uv2, set_uv2: 19;
    /// Returns or changes the uv3 field.
    pub uv3, set_uv3: 20;
    /// Returns or changes the normal field.
    pub normal, set_normal: 23;
    /// Returns or changes the tangent field.
    pub tangent, set_tangent: 24;
    /// Returns or changes the uv4 field.
    pub uv4, set_uv4: 30;
}
impl From<u32> for VertexFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<VertexFlags> for u32 {
    fn from(value: VertexFlags) -> Self {
        value.bits()
    }
}
