//! M3 scene selectors and bit masks.
use bitfield::bitfield;

/// M3 ProjectionKind selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ProjectionKind {
    /// Selects Perspective.
    Perspective,
    /// Selects Orthogonal.
    Orthogonal,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl ProjectionKind {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Perspective => 0,
            Self::Orthogonal => 1,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Perspective,
            1 => Self::Orthogonal,
            value => Self::Unknown(value),
        }
    }
}

impl Default for ProjectionKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 VolumeShape selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VolumeShape {
    /// Selects Box.
    Box,
    /// Selects Sphere.
    Sphere,
    /// Selects Capsule.
    Capsule,
    /// Selects Cylinder.
    Cylinder,
    /// Selects Mesh.
    Mesh,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl VolumeShape {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Box => 0,
            Self::Sphere => 1,
            Self::Capsule => 2,
            Self::Cylinder => 3,
            Self::Mesh => 4,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Box,
            1 => Self::Sphere,
            2 => Self::Capsule,
            3 => Self::Cylinder,
            4 => Self::Mesh,
            value => Self::Unknown(value),
        }
    }
}

impl Default for VolumeShape {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 BillboardKind selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum BillboardKind {
    /// Selects WorldX.
    WorldX,
    /// Selects WorldY.
    WorldY,
    /// Selects WorldZ.
    WorldZ,
    /// Selects LocalX.
    LocalX,
    /// Selects LocalY.
    LocalY,
    /// Selects LocalZ.
    LocalZ,
    /// Selects Full.
    Full,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u8),
}
impl BillboardKind {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u8 {
        match self {
            Self::WorldX => 0,
            Self::WorldY => 1,
            Self::WorldZ => 2,
            Self::LocalX => 3,
            Self::LocalY => 4,
            Self::LocalZ => 5,
            Self::Full => 6,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::WorldX,
            1 => Self::WorldY,
            2 => Self::WorldZ,
            3 => Self::LocalX,
            4 => Self::LocalY,
            5 => Self::LocalZ,
            6 => Self::Full,
            value => Self::Unknown(value),
        }
    }
}

impl Default for BillboardKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 ForceKind selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ForceKind {
    /// Selects Directional.
    Directional,
    /// Selects Radial.
    Radial,
    /// Selects Damping.
    Damping,
    /// Selects Vortex.
    Vortex,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl ForceKind {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Directional => 0,
            Self::Radial => 1,
            Self::Damping => 2,
            Self::Vortex => 3,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Directional,
            1 => Self::Radial,
            2 => Self::Damping,
            3 => Self::Vortex,
            value => Self::Unknown(value),
        }
    }
}

impl Default for ForceKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 ForceShape selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ForceShape {
    /// Selects Sphere.
    Sphere,
    /// Selects Cylinder.
    Cylinder,
    /// Selects Box.
    Box,
    /// Selects Hemisphere.
    Hemisphere,
    /// Selects ConeDome.
    ConeDome,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl ForceShape {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Sphere => 0,
            Self::Cylinder => 1,
            Self::Box => 2,
            Self::Hemisphere => 3,
            Self::ConeDome => 4,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Sphere,
            1 => Self::Cylinder,
            2 => Self::Box,
            3 => Self::Hemisphere,
            4 => Self::ConeDome,
            value => Self::Unknown(value),
        }
    }
}

impl Default for ForceShape {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 LightKind selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LightKind {
    /// Selects Directional.
    Directional,
    /// Selects Point.
    Point,
    /// Selects Spot.
    Spot,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u16),
}
impl LightKind {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Directional => 0,
            Self::Point => 1,
            Self::Spot => 2,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Directional,
            1 => Self::Point,
            2 => Self::Spot,
            value => Self::Unknown(value),
        }
    }
}

impl Default for LightKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// M3 ProjectorFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 OneBoneFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 ForceFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 LightFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 ForceChannels mask, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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

bitfield! {
    /// Turret control mask retaining every unknown bit.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
    pub struct TurretPartFlags(u8);
    /// Complete authored control word.
    pub bits, _: 7, 0;
    /// Returns or changes the documented main part control.
    pub main_part, set_main_part: 0;
}

bitfield! {
    /// Turret control mask retaining every unknown bit.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
    pub struct TurretAxisFlags(u32);
    /// Complete authored control word.
    pub bits, _: 31, 0;
    /// Returns or changes the documented limited control.
    pub limited, set_limited: 1;
}
