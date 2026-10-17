//! Logical M3 flare values.
use super::{Property, TextureLayer};
use std::borrow::Cow;

/// Editable M3 SubFlare values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SubFlare {
    /// Sets flare element index.
    pub index: u32,
    /// Controls volume position through its animation value.
    pub position: f32,
    /// Sets the flare element’s width and height.
    pub size_xy: [f32; 2],
    /// Scales the flare element along its width and height.
    pub scale_xy: [f32; 2],
    /// Sets fade-in range (start, end).
    pub fade_in: [f32; 2],
    /// Sets fade-out range (start, end).
    pub fade_out: [f32; 2],
    /// Sets flare color and alpha.
    pub color_alpha: [f32; 4],
    /// Controls whether the element points toward the flare’s center.
    pub face_center: u32,
    /// Sets offset from flare center.
    pub offset: [f32; 2],
}

/// Editable M3 LensFlareMaterialStep values.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LensFlareMaterialStep<'a> {
    /// Sets flare texture atlas.
    pub flare_map: Option<TextureLayer>,
    /// Supplies the mask delimiting the creep effect.
    pub mask_map: Option<TextureLayer>,
    /// Sets sub-flare elements (LFSB).
    pub sub_flares: Vec<SubFlare>,
    /// Sets flipbook grid columns.
    pub columns: u32,
    /// Sets flipbook grid rows.
    pub rows: u32,
    /// Sets distance fade start.
    pub distance_fade: f32,
    /// Links to the CHAR bytes naming the flare library.
    pub lib_name: Cow<'a, str>,
    /// Controls intensity through its animation value.
    pub intensity: Property<f32>,
    /// Controls color tint through its animation value.
    pub color: Property<[f32; 4]>,
    /// Controls HDR multiplier through its animation value.
    pub hdr: Property<f32>,
    /// Controls size through its animation value.
    pub size: Property<f32>,
}
