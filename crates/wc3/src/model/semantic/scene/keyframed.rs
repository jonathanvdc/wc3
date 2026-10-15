//! Editable Warcraft III effects and scene features.
use crate::model::semantic::values::layered::{GeometryAnimationFlags, PopcornFlags};
use crate::model::semantic::AtlasParticles;
use crate::model::semantic::GravityTrail;
use crate::model::semantic::IndexedAttachment;
use crate::model::semantic::ResourceParticle;
use crate::model::semantic::{
    AimedCamera, AmbientDiffuseLight, PhotographicControls, VisibleLight,
};
use crate::model::semantic::{Animated, MeshId};
use std::borrow::Cow;

/// Opacity, tint, and shadow behavior for one geometry controller.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GeometryAnimation {
    /// Geometry controlled by this feature.
    pub mesh: Option<MeshId>,
    /// Geometry opacity.
    pub opacity: Animated<f32>,
    /// Geometry RGB tint.
    pub color: Animated<[f32; 3]>,
    /// Enable authored tint.
    pub use_color: bool,
    /// Enable authored shadow-drop behavior.
    pub drop_shadow: bool,
    /// Behavior bits with unresolved meanings.
    pub unclassified: GeometryAnimationFlags,
}
/// Resource-particle emitter parameters.
pub type Particle<'a> = ResourceParticle<'a>;
/// Sprite-particle rendering selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Frames {
    /// Draw the particle head.
    #[default]
    Head,
    /// Draw only the trail.
    Tail,
    /// Draw both the particle and trail.
    Both,
    /// Authored selection with unresolved meaning.
    Unknown(u32),
}
/// Atlas sprite and tail emitter parameters.
pub type Particle2 = AtlasParticles;
/// Ribbon trail generation parameters.
pub type Ribbon = GravityTrail;
/// External PopcornFX resource and authored input properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Popcorn<'a> {
    /// Effect resource path.
    pub resource: Cow<'a, str>,
    /// Animation-name visibility rules with literal line breaks.
    pub visibility_guide: Cow<'a, str>,
    /// Particle lifetime.
    pub life_span: Animated<f32>,
    /// Particle emission rate.
    pub emission_rate: Animated<f32>,
    /// Initial particle speed.
    pub speed: Animated<f32>,
    /// Particle RGB tint.
    pub color: Animated<[f32; 3]>,
    /// Particle opacity.
    pub opacity: Animated<f32>,
    /// Replaceable-texture category.
    pub replaceable: u32,
    /// Visibility multiplier.
    pub visibility: Animated<f32>,
    /// Bypass lighting.
    pub unlit: bool,
    /// Sort generated primitives far to near.
    pub sort_far_z: bool,
    /// Bypass fog.
    pub unfogged: bool,
    /// Apply the authored popcorn scaling behavior.
    pub scaling: bool,
    /// Emitter controls with unresolved meanings.
    pub unclassified: PopcornFlags,
}
/// Illumination behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LightKind {
    /// Point illumination.
    #[default]
    Omni,
    /// Directional illumination.
    Directional,
    /// Ambient illumination.
    Ambient,
    /// Kind with unresolved meaning.
    Unknown(u32),
}
/// Complete authored Warcraft III light parameters.
pub type Light = AmbientDiffuseLight<LightKind, VisibleLight>;
/// Authored camera with animated eye, target, roll, and photographic parameters.
pub type Camera = AimedCamera<f32, PhotographicControls>;
/// External model attachment.
pub type Attachment<'a> = IndexedAttachment<Cow<'a, str>>;
/// Events generated on a model or independent looping clock.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Events<'a> {
    /// Gameplay event name.
    pub name: Cow<'a, str>,
}
/// Collision primitives independent of their native vertex packing.
#[derive(Clone, Debug, PartialEq)]
pub enum Collision {
    /// Axis-aligned box with minimum and maximum corners.
    Box([[f32; 3]; 2]),
    /// Plane represented by its two authored vertices.
    Plane([[f32; 3]; 2]),
    /// Sphere center and radius.
    Sphere {
        /// Center in model coordinates.
        center: [f32; 3],
        /// Radius in model units.
        radius: f32,
    },
    /// Cylinder endpoints and radius.
    Cylinder {
        /// Endpoints in model coordinates.
        endpoints: [[f32; 3]; 2],
        /// Radius in model units.
        radius: f32,
    },
    /// Collision kind whose geometric meaning is unresolved.
    Unknown {
        /// Source kind selector.
        kind: u32,
        /// Authored defining vertices.
        vertices: Vec<[f32; 3]>,
        /// Authored radius.
        radius: Option<f32>,
    },
}
