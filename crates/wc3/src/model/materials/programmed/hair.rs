//! Anisotropic M3 hair surface parameters.
use super::{Property, TextureLayer};
use crate::model::materials::contributions::SurfaceContribution;
/// Two-lobe anisotropic hair shading with independently animated colors.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Hair {
    /// Map shifting the anisotropic specular response.
    pub layer_spec_shift: Option<TextureLayer>,
    /// Noise map varying the specular response.
    pub layer_spec_noise: Option<TextureLayer>,
    /// Ambient-occlusion map.
    pub layer_ambient_occlusion: Option<TextureLayer>,
    /// Offset of the primary specular lobe.
    pub shift_primary: f32,
    /// Offset of the secondary specular lobe.
    pub shift_secondary: f32,
    /// Specular tint in normalized RGBA order.
    pub color_specular: Property<[f32; 4]>,
    /// Primary specular exponent.
    pub specular_exponent_primary: f32,
    /// Secondary specular exponent.
    pub specular_exponent_secondary: f32,
    /// Authored diffuse input and response.
    pub diffuse: SurfaceContribution<Option<TextureLayer>, Property<[f32; 4]>>,
}
