//! Private lowering of native ribbons into the shared ribbon transport.
use wc3::model::profiles::{m2, mdlx};
use wc3::model::{
    M2RibbonControls, M2RibbonRendering, MdlxRibbonControls, MdlxRibbonRendering, Property,
    TrailAppearance, TrailGeometry,
};
/// Prepared fields shared by the MDLX and M2 ribbon transport.
#[derive(Clone, Debug)]
pub(super) struct TrailEmitter {
    pub emission_rate: f64,
    pub lifetime: f32,
    pub geometry: TrailGeometry,
    pub appearance: TrailAppearance<TrailAtlasSlot>,
    pub rendering: TrailRendering,
    pub controls: TrailControls,
}
/// Typed atlas selectors retain animation identity and value domain.
#[derive(Clone, Debug)]
pub(super) enum TrailAtlasSlot {
    /// Full-width cell selection.
    Wide(Property<u32>),
    /// Compact cell selection.
    Compact(Property<u16>),
}
/// Logical rendering alternatives rather than source-file layouts.
#[derive(Clone, Debug)]
pub(super) enum TrailRendering {
    /// A layered material supplies the complete surface program.
    Layered(MdlxRibbonRendering),
    /// Ordered texture and shader-surface bindings.
    Surfaces(M2RibbonRendering),
}
/// Physical and application controls with explicit meanings.
#[derive(Clone, Debug)]
#[allow(
    clippy::large_enum_variant,
    reason = "Prepared controls are cloned once per compiled emitter."
)]
pub(super) enum TrailControls {
    /// Gravitational acceleration and fractional visibility.
    Gravity(MdlxRibbonControls),
    /// Emission direction and discrete activation with texture controls.
    EmissionAngle(M2RibbonControls),
}
impl From<mdlx::RibbonEmitter> for TrailEmitter {
    fn from(v: mdlx::RibbonEmitter) -> Self {
        Self {
            emission_rate: f64::from(v.emission_rate),
            lifetime: v.lifetime,
            geometry: v.geometry,
            appearance: TrailAppearance {
                color: v.appearance.color,
                opacity: v.appearance.opacity,
                atlas: v.appearance.atlas,
                texture_slot: TrailAtlasSlot::Wide(v.appearance.texture_slot),
            },
            rendering: TrailRendering::Layered(v.rendering),
            controls: TrailControls::Gravity(v.controls),
        }
    }
}
impl From<m2::RibbonEmitter> for TrailEmitter {
    fn from(v: m2::RibbonEmitter) -> Self {
        Self {
            emission_rate: f64::from(v.emission_rate),
            lifetime: v.lifetime,
            geometry: v.geometry,
            appearance: TrailAppearance {
                color: v.appearance.color,
                opacity: v.appearance.opacity,
                atlas: v.appearance.atlas.map(u32::from),
                texture_slot: TrailAtlasSlot::Compact(v.appearance.texture_slot),
            },
            rendering: TrailRendering::Surfaces(v.rendering),
            controls: TrailControls::EmissionAngle(v.controls),
        }
    }
}
