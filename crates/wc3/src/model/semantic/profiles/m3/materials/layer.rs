//! Logical M3 layer values.
use super::super::values::{ColorChannel, FresnelMode, LayerFlags, UvMappingMode};
use super::Property;
use crate::model::semantic::TextureId;

/// Editable M3 TextureLayer values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureLayer {
    /// Identifies this texture layer within the material setup.
    pub id: u32,
    /// Names the texture filename.
    pub texture: Option<TextureId>,
    /// Controls color tint through its animation value.
    pub color: Property<[f32; 4]>,
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: LayerFlags,
    /// Selects the behavior used for uv mapping source.
    pub uv_mapping: UvMappingMode,
    /// Selects the behavior used for channel selection.
    pub color_type: ColorChannel,
    /// Sets RGB multiply factor.
    pub rgb_multiply: Property<f32>,
    /// Sets RGB additive factor.
    pub rgb_add: Property<f32>,
    /// Stores the POC texture selector; its target namespace is not established.
    pub poc_texture: u32,
    /// Sets noise amplitude.
    pub noise_amplitude: f32,
    /// Sets noise frequency.
    pub noise_frequency: f32,
    /// Sets texture source override.
    pub texture_source: u32,
    /// Sets AVI playback frame rate.
    pub avi_frame_rate: u32,
    /// Sets AVI start frame.
    pub avi_start: u32,
    /// Sets AVI stop frame.
    pub avi_stop: u32,
    /// Selects the behavior used for avi loop mode.
    pub avi_loop: u32,
    /// Selects the behavior used for avi sync mode.
    pub avi_sync: u32,
    /// Sets AVI play control.
    pub avi_play: Property<u32>,
    /// Sets AVI restart control.
    pub avi_restart: Property<u32>,
    /// Sets flipbook grid rows.
    pub flipbook_rows: u32,
    /// Sets flipbook grid columns.
    pub flipbook_columns: u32,
    /// Controls flipbook frame index through its animation value.
    pub current_frame: Property<u16>,
    /// Controls UV offset through its animation value.
    pub uv_offset: Property<[f32; 2]>,
    /// Controls UV rotation angles through its animation value.
    pub uv_angle: Property<[f32; 3]>,
    /// Controls UV tiling through its animation value.
    pub uv_tiling: Property<[f32; 2]>,
    /// Controls w offset (3D textures) through its animation value.
    pub w_offset: Property<f32>,
    /// Controls w tiling (3D textures) through its animation value.
    pub w_tiling: Property<f32>,
    /// Controls the texture contribution’s alpha through an animation value.
    pub map_alpha: Property<f32>,
    /// Sets tri-planar UV offset.
    pub triplanar_offset: Property<[f32; 3]>,
    /// Sets tri-planar UV scale.
    pub triplanar_scale: Property<[f32; 3]>,
    /// Selects the layer supplying shared UV settings; all bits set selects this layer’s own settings.
    pub uv_source_related: u32,
    /// Selects the behavior used for fresnel effect mode.
    pub fresnel_mode: FresnelMode,
    /// Sets fresnel exponent (edge sharpness).
    pub fresnel_exponent: f32,
    /// Sets fresnel minimum intensity.
    pub fresnel_min: f32,
    /// Sets fresnel maximum intensity.
    pub fresnel_max: f32,
    /// Sets fresnel UV translation.
    pub fresnel_translation: [f32; 3],
    /// Sets fresnel mask vector.
    pub fresnel_mask: [f32; 3],
    /// Sets fresnel UV rotation.
    pub fresnel_rotation: [f32; 2],
    /// Sets UV density hint.
    pub uv_density: u32,
}
