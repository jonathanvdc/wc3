//! Region-local point illumination and authored flicker programs.
use crate::model::{FeatureId, TextureId};
/// A point light whose activation belongs to a region rather than the root actor table.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PointLight {
    /// Authored RGBA color bytes.
    pub color: [u8; 4],
    /// Intensity multiplier.
    pub intensity: f32,
    /// Start and end attenuation distances.
    pub attenuation: [f32; 2],
    /// Unresolved authored header control.
    pub control: u32,
    /// Unresolved floating-point control.
    pub scalar_control: f32,
    /// Unresolved trailing controls.
    pub trailing_controls: [u32; 2],
}
/// An independently placed flicker/cookie program associated with a region point light.
#[derive(Clone, Debug, PartialEq)]
pub struct PointProgram {
    /// Light receiving this program.
    pub light: FeatureId,
    /// Authored color bytes.
    pub color: [u8; 4],
    /// Intensity multiplier.
    pub intensity: f32,
    /// Attenuation start and end distances.
    pub attenuation: [f32; 2],
    /// Authored Euler orientation, in radians; axis ordering remains unspecified.
    pub orientation: [f32; 3],
    /// Flicker variation and speed.
    pub flicker: [f32; 2],
    /// Flicker behavior.
    pub mode: super::world::Flicker,
    /// Optional projected-light cookie.
    pub cookie: Option<TextureId>,
    /// Unresolved controls preceding cookie selection.
    pub controls: [i32; 4],
    /// Unresolved trailing controls.
    pub trailing_controls: [i32; 5],
}
