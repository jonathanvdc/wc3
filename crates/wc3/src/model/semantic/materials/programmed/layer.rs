//! Programmed image inputs, coordinates, playback, and color contributions.
use super::Property;
use crate::model::semantic::materials::bindings::{Coordinates, TextureBinding};
use crate::model::semantic::values::programmed::{
    ColorChannel, FresnelMode, LayerFlags, UvMappingMode,
};
use crate::model::semantic::{Link, TextureId, ValidateLinks};
/// Animated 2D, 3D, and triplanar coordinate program.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoordinateTransform {
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
    /// Sets tri-planar UV offset.
    pub triplanar_offset: Property<[f32; 3]>,
    /// Sets tri-planar UV scale.
    pub triplanar_scale: Property<[f32; 3]>,
    /// Selects the layer supplying shared UV settings; all bits set selects this layer’s own settings.
    pub uv_source_related: u32,
    /// Sets UV density hint.
    pub uv_density: u32,
}
/// Noise, flipbook, and video playback controls for one sampled image.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SamplingProgram {
    /// Sets layer flags (wrap, flipbook, video, etc.).
    pub flags: LayerFlags,
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
}
/// Channel selection and animated sampled-color contribution.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ColorContribution {
    /// Controls color tint through its animation value.
    pub color: Property<[f32; 4]>,
    /// Selects the behavior used for channel selection.
    pub color_type: ColorChannel,
    /// Sets RGB multiply factor.
    pub rgb_multiply: Property<f32>,
    /// Sets RGB additive factor.
    pub rgb_add: Property<f32>,
    /// Controls the texture contribution’s alpha through an animation value.
    pub map_alpha: Property<f32>,
}
/// A programmed shader input, retaining independently authored Fresnel behavior.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureLayer {
    /// Identifies this input within the shader setup.
    pub id: u32,
    /// Image, coordinate program, playback, and color contribution.
    pub binding: TextureBinding<
        Option<TextureId>,
        Coordinates<UvMappingMode, CoordinateTransform>,
        SamplingProgram,
        ColorContribution,
    >,
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
}
impl ValidateLinks for CoordinateTransform {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.uv_offset.visit_links(visitor);
        self.uv_angle.visit_links(visitor);
        self.uv_tiling.visit_links(visitor);
        self.w_offset.visit_links(visitor);
        self.w_tiling.visit_links(visitor);
        self.triplanar_offset.visit_links(visitor);
        self.triplanar_scale.visit_links(visitor);
    }
}
impl ValidateLinks for SamplingProgram {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.noise_amplitude.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.noise_frequency.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.avi_play.visit_links(visitor);
        self.avi_restart.visit_links(visitor);
        self.current_frame.visit_links(visitor);
    }
}
impl ValidateLinks for ColorContribution {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.color.visit_links(visitor);
        self.rgb_multiply.visit_links(visitor);
        self.rgb_add.visit_links(visitor);
        self.map_alpha.visit_links(visitor);
    }
}

impl ValidateLinks for UvMappingMode {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
