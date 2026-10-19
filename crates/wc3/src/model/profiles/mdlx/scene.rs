//! Editable Warcraft III effects and scene features.
use crate::model::Construct;
use crate::model::EffectResourceId;
use crate::model::IndexedChildModel;
use crate::model::MdlxParticleEmitter1;
use crate::model::MdlxParticleEmitter2;
use crate::model::MdlxRibbonEmitter;
use crate::model::ModelResourceId;
use crate::model::{AmbientDiffuseLight, LookAtCamera};
use crate::model::{MeshId, Property};
use glam::Vec3;
use std::borrow::Cow;

/// Opacity, tint, and shadow behavior for one geometry controller.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct GeometryAnimation {
    /// Geometry controlled by this feature.
    pub mesh: Option<MeshId>,
    /// Geometry opacity.
    pub opacity: Property<f32>,
    /// Geometry RGB tint.
    pub color: Property<Vec3>,
    /// Enable authored tint.
    pub use_color: bool,
    /// Enable authored shadow-drop behavior.
    pub drop_shadow: bool,
}
/// Resource-particle emitter parameters.
pub type ParticleEmitter1 = MdlxParticleEmitter1;
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
pub type ParticleEmitter2 = MdlxParticleEmitter2;
/// Ribbon trail generation parameters.
pub type RibbonEmitter = MdlxRibbonEmitter;
/// External PopcornFX resource and authored input properties.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct Popcorn<'a> {
    /// Effect resource path.
    pub resource: Option<EffectResourceId>,
    /// Animation-name visibility rules with literal line breaks.
    pub visibility_guide: Cow<'a, str>,
    /// Particle lifetime.
    pub life_span: Property<f32>,
    /// Particle emission rate.
    pub emission_rate: Property<f32>,
    /// Initial particle speed.
    pub speed: Property<f32>,
    /// Particle RGB tint.
    pub color: Property<Vec3>,
    /// Particle opacity.
    pub opacity: Property<f32>,
    /// Replaceable-texture category.
    pub replaceable: u32,
    /// Visibility multiplier.
    pub visibility: Property<f32>,
    /// Bypass lighting.
    pub unlit: bool,
    /// Sort generated primitives far to near.
    pub sort_far_z: bool,
    /// Bypass fog.
    pub unfogged: bool,
    /// Apply the authored popcorn scaling behavior.
    pub scaling: bool,
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
pub type Light = AmbientDiffuseLight<LightKind, Property<f32>>;
/// Authored camera with animated eye, target, roll, and photographic parameters.
pub type Camera = LookAtCamera<f32>;
/// External model attachment.
pub type Attachment = IndexedChildModel<ModelResourceId>;
/// Collision primitives independent of their native vertex packing.
#[derive(Clone, Debug, PartialEq)]
pub enum Collision {
    /// Axis-aligned box with minimum and maximum corners.
    Box([Vec3; 2]),
    /// Plane represented by its two authored vertices.
    Plane([Vec3; 2]),
    /// Sphere center and radius.
    Sphere {
        /// Center in model coordinates.
        center: Vec3,
        /// Radius in model units.
        radius: f32,
    },
    /// Cylinder endpoints and radius.
    Cylinder {
        /// Endpoints in model coordinates.
        endpoints: [Vec3; 2],
        /// Radius in model units.
        radius: f32,
    },
    /// Collision kind whose geometric meaning is unresolved.
    Unknown {
        /// Source kind selector.
        kind: u32,
        /// Authored defining vertices.
        vertices: Vec<Vec3>,
        /// Authored radius.
        radius: Option<f32>,
    },
}

use crate::model::profiles::Mdlx;
use crate::model::FeatureSet;
/// Complete profile-specific feature attached to a hierarchy node.
pub type Node<'a> = FeatureSet<'a, Mdlx>;
