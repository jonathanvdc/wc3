//! Native scene selectors and bit masks.
use crate::model::m3;
use bitfield::bitfield;

/// Native ProjectionKind selector.
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
#[m3(value = u32)]
pub enum ProjectionKind {
    /// Selects Perspective.
    #[m3(value = 0)]
    Perspective,
    /// Selects Orthogonal.
    #[m3(value = 1)]
    Orthogonal,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for ProjectionKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native VolumeShape selector.
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
#[m3(value = u32)]
pub enum VolumeShape {
    /// Selects Box.
    #[m3(value = 0)]
    Box,
    /// Selects Sphere.
    #[m3(value = 1)]
    Sphere,
    /// Selects Capsule.
    #[m3(value = 2)]
    Capsule,
    /// Selects Cylinder.
    #[m3(value = 3)]
    Cylinder,
    /// Selects Mesh.
    #[m3(value = 4)]
    Mesh,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for VolumeShape {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native BillboardKind selector.
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
#[m3(value = u8)]
pub enum BillboardKind {
    /// Selects WorldX.
    #[m3(value = 0)]
    WorldX,
    /// Selects WorldY.
    #[m3(value = 1)]
    WorldY,
    /// Selects WorldZ.
    #[m3(value = 2)]
    WorldZ,
    /// Selects LocalX.
    #[m3(value = 3)]
    LocalX,
    /// Selects LocalY.
    #[m3(value = 4)]
    LocalY,
    /// Selects LocalZ.
    #[m3(value = 5)]
    LocalZ,
    /// Selects Full.
    #[m3(value = 6)]
    Full,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u8),
}
impl Default for BillboardKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native ForceKind selector.
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
#[m3(value = u32)]
pub enum ForceKind {
    /// Selects Directional.
    #[m3(value = 0)]
    Directional,
    /// Selects Radial.
    #[m3(value = 1)]
    Radial,
    /// Selects Damping.
    #[m3(value = 2)]
    Damping,
    /// Selects Vortex.
    #[m3(value = 3)]
    Vortex,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for ForceKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native ForceShape selector.
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
#[m3(value = u32)]
pub enum ForceShape {
    /// Selects Sphere.
    #[m3(value = 0)]
    Sphere,
    /// Selects Cylinder.
    #[m3(value = 1)]
    Cylinder,
    /// Selects Box.
    #[m3(value = 2)]
    Box,
    /// Selects Hemisphere.
    #[m3(value = 3)]
    Hemisphere,
    /// Selects ConeDome.
    #[m3(value = 4)]
    ConeDome,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for ForceShape {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native LightKind selector.
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
pub enum LightKind {
    /// Selects Directional.
    #[m3(value = 0)]
    Directional,
    /// Selects Point.
    #[m3(value = 1)]
    Point,
    /// Selects Spot.
    #[m3(value = 2)]
    Spot,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u16),
}
impl Default for LightKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// Native ProjectorFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct ProjectorFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the static control.
    pub is_static, set_static: 0;
}
impl From<u32> for ProjectorFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ProjectorFlags> for u32 {
    fn from(value: ProjectorFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native OneBoneFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct OneBoneFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
}
impl From<u32> for OneBoneFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<OneBoneFlags> for u32 {
    fn from(value: OneBoneFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native ForceFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct ForceFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the falloff control.
    pub falloff, set_falloff: 0;
    /// Returns or changes the height gradient control.
    pub height_gradient, set_height_gradient: 1;
    /// Returns or changes the unbounded control.
    pub unbounded, set_unbounded: 2;
}
impl From<u32> for ForceFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ForceFlags> for u32 {
    fn from(value: ForceFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native LightFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct LightFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the shadows control.
    pub shadows, set_shadows: 0;
    /// Returns or changes the spec control.
    pub spec, set_spec: 1;
    /// Returns or changes the ao control.
    pub ao, set_ao: 2;
    /// Returns or changes the light opaque control.
    pub light_opaque, set_light_opaque: 3;
    /// Returns or changes the light transparent control.
    pub light_transparent, set_light_transparent: 4;
    /// Returns or changes the team color control.
    pub team_color, set_team_color: 5;
}
impl From<u32> for LightFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<LightFlags> for u32 {
    fn from(value: LightFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native ForceChannels mask, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct ForceChannels(u32);
    /// Complete native bit mask.
    pub bits, _: 31, 0;
}
impl From<u32> for ForceChannels {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ForceChannels> for u32 {
    fn from(value: ForceChannels) -> Self {
        value.bits()
    }
}
