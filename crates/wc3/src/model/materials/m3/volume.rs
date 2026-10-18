//! Logical M3 volume values.
use super::{PropertyBinding, TextureLayer};
use crate::model::values::programmed::{
    BlendMode, VolumeCameraMode, VolumeFalloff, VolumeFlags, VolumeNoiseFlags,
};
use crate::model::Construct;

/// Editable M3 VolumeMaterialStep values.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct VolumeMaterialStep {
    /// Selects the behavior used for alpha blend mode.
    pub blend_mode: BlendMode,
    /// Selects the behavior used for density falloff type.
    pub falloff_type: VolumeFalloff,
    /// Controls density through its animation value.
    pub density: PropertyBinding<f32>,
    /// Supplies the map supplying color.
    pub color_map: Option<TextureLayer>,
    /// Supplies the first noise texture.
    pub noise_map1: Option<TextureLayer>,
    /// Supplies the second noise texture.
    pub noise_map2: Option<TextureLayer>,
    /// Sets alpha test threshold.
    pub alpha_threshold: u32,
    /// Retains the native material control bits; unresolved bits remain intact.
    pub flags: VolumeFlags,
}

/// Editable M3 VolumeNoiseMaterialStep values.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct VolumeNoiseMaterialStep {
    /// Selects the behavior used for density falloff type.
    pub falloff_type: VolumeFalloff,
    /// Selects the transparency behavior for a camera inside or outside the volume.
    pub draw_transparency: VolumeCameraMode,
    /// Controls density through its animation value.
    pub density: PropertyBinding<f32>,
    /// Controls near-plane clip through its animation value.
    pub near_plane: PropertyBinding<f32>,
    /// Controls falloff distance through its animation value.
    pub falloff: PropertyBinding<f32>,
    /// Supplies the map supplying color.
    pub color_map: Option<TextureLayer>,
    /// Supplies the first noise texture.
    pub noise_map1: Option<TextureLayer>,
    /// Supplies the second noise texture.
    pub noise_map2: Option<TextureLayer>,
    /// Controls noise scroll rate through its animation value.
    pub scroll_rate: PropertyBinding<[f32; 3]>,
    /// Controls volume position through its animation value.
    pub position: PropertyBinding<[f32; 3]>,
    /// Controls volume scale through its animation value.
    pub scale: PropertyBinding<[f32; 3]>,
    /// Controls volume rotation through its animation value.
    pub rotation: PropertyBinding<[f32; 3]>,
    /// Sets alpha test threshold.
    pub alpha_threshold: u32,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: VolumeNoiseFlags,
}
