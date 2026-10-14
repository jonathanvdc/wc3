//! Editable Warcraft III effects and scene features.
use super::Blend;
use super::{GeometryAnimationFlags, Particle2Flags, ParticleFlags, PopcornFlags};
use crate::model::semantic::{Animated, MaterialId, MeshId, TextureId};
use std::borrow::Cow;

/// Complete profile-specific feature attached to a hierarchy node.
#[derive(Clone, Debug, PartialEq)]
pub enum Node<'a> {
    /// Individually referenced geometry opacity and tint controller.
    GeometryAnimation(GeometryAnimation),
    /// Model-resource particles.
    Particle(Particle<'a>),
    /// Atlas-based sprite and tail particles.
    Particle2(Particle2),
    /// Generated ribbon geometry.
    Ribbon(Ribbon),
    /// External PopcornFX effect.
    Popcorn(Popcorn<'a>),
    /// Authored local light.
    Light(Light),
    /// Authored camera and target.
    Camera(Camera),
    /// External model attachment.
    Attachment(Attachment<'a>),
    /// Gameplay event source.
    Events(Events<'a>),
    /// Collision primitive.
    Collision(Collision),
}
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Particle<'a> {
    /// Model or image resource instantiated per particle.
    pub resource: Cow<'a, str>,
    /// Number of particles emitted each second.
    pub emission_rate: Animated<f32>,
    /// Acceleration along the emitter gravity direction.
    pub gravity: Animated<f32>,
    /// Emission-cone longitude in degrees.
    pub longitude: Animated<f32>,
    /// Emission-cone latitude in degrees.
    pub latitude: Animated<f32>,
    /// Particle lifetime in seconds.
    pub life_span: Animated<f32>,
    /// Initial particle speed.
    pub initial_velocity: Animated<f32>,
    /// Authored visibility multiplier.
    pub visibility: Animated<f32>,
    /// Interpret the resource as a model.
    pub model_resource: bool,
    /// Interpret the resource as a text image.
    pub text_resource: bool,
    /// Emitter controls with unresolved meanings.
    pub unclassified: ParticleFlags,
}
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Particle2 {
    /// Initial particle speed.
    pub speed: Animated<f32>,
    /// Speed variation factor.
    pub variation: Animated<f32>,
    /// Emission-cone latitude in degrees.
    pub latitude: Animated<f32>,
    /// Gravity acceleration.
    pub gravity: Animated<f32>,
    /// Particle lifetime in seconds.
    pub life_span: f32,
    /// Emission rate or burst count.
    pub emission_rate: Animated<f32>,
    /// Length of the emission region.
    pub length: Animated<f32>,
    /// Width of the emission region.
    pub width: Animated<f32>,
    /// Color composition operation.
    pub blend: Blend,
    /// Atlas dimensions in rows and columns.
    pub atlas: [u32; 2],
    /// Sprite and trail rendering choice.
    pub frames: Frames,
    /// Trail time span in seconds.
    pub tail_length: f32,
    /// Lifetime fraction at the middle appearance stage.
    pub middle_time: f32,
    /// RGB colors at the three lifetime stages.
    pub colors: [[f32; 3]; 3],
    /// Opacity at the three lifetime stages.
    pub alpha: [u8; 3],
    /// Scale at the three lifetime stages.
    pub scale: [f32; 3],
    /// Start, end, and repeat counts for head/lifespan/decay/tail atlas animations.
    pub uv_intervals: [[u32; 3]; 4],
    /// Atlas texture resource.
    pub texture: Option<TextureId>,
    /// Emit one burst at emission-rate keys.
    pub squirt: bool,
    /// Render-order priority.
    pub priority_plane: u32,
    /// External replaceable-texture category.
    pub replaceable: u32,
    /// Visibility multiplier.
    pub visibility: Animated<f32>,
    /// Model-space rather than world-space simulation.
    pub model_space: bool,
    /// Keep particles aligned to the XY plane.
    pub xy_quad: bool,
    /// Apply linear rather than exponential variation.
    pub line_emitter: bool,
    /// Sort particles far to near.
    pub sort_far_z: bool,
    /// Bypass lighting.
    pub unlit: bool,
    /// Bypass fog.
    pub unfogged: bool,
    /// Emitter controls with unresolved meanings.
    pub unclassified: Particle2Flags,
}
/// Ribbon trail generation parameters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ribbon {
    /// Height above the emission point.
    pub height_above: Animated<f32>,
    /// Height below the emission point.
    pub height_below: Animated<f32>,
    /// Opacity multiplier.
    pub opacity: Animated<f32>,
    /// Ribbon RGB tint.
    pub color: Animated<[f32; 3]>,
    /// Trail-section lifetime in seconds.
    pub life_span: f32,
    /// Atlas cell selection.
    pub texture_slot: Animated<u32>,
    /// Trail sections emitted per second.
    pub emission_rate: u32,
    /// Atlas dimensions in rows and columns.
    pub atlas: [u32; 2],
    /// Full layered material used to render the ribbon.
    pub material: Option<MaterialId>,
    /// Gravity acceleration.
    pub gravity: f32,
    /// Visibility multiplier.
    pub visibility: Animated<f32>,
}
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Light {
    /// Illumination behavior.
    pub kind: LightKind,
    /// Attenuation start and end distance.
    pub attenuation: [Animated<f32>; 2],
    /// Direct RGB color.
    pub color: Animated<[f32; 3]>,
    /// Direct light intensity.
    pub intensity: Animated<f32>,
    /// Ambient RGB color.
    pub ambient_color: Animated<[f32; 3]>,
    /// Ambient intensity.
    pub ambient_intensity: Animated<f32>,
    /// Shadow casting, when explicitly authored.
    pub shadow_casting: Option<bool>,
    /// Shadow strength, when explicitly authored.
    pub shadow_intensity: Option<f32>,
    /// Animated shadow distance range.
    pub shadow_range: Option<[Animated<f32>; 2]>,
    /// Animated quadratic, linear, and damping attenuation coefficients.
    pub falloff: Option<[Animated<f32>; 3]>,
    /// Visibility multiplier.
    pub visibility: Animated<f32>,
}
/// Authored camera with animated eye, target, roll, and photographic parameters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Camera {
    /// Additive eye offset from the hierarchy node's rest translation.
    pub eye_offset: Animated<[f32; 3]>,
    /// Authored model-space aim point.
    pub target: [f32; 3],
    /// Additive offset from the authored aim point.
    pub target_offset: Animated<[f32; 3]>,
    /// Vertical field of view in radians.
    pub field_of_view: f32,
    /// Near and far clipping distance.
    pub clipping: [f32; 2],
    /// Camera roll in radians.
    pub roll: Animated<f32>,
    /// Visibility multiplier.
    pub visibility: Animated<f32>,
    /// Depth-of-field focus distance.
    pub focus_distance: Option<Animated<f32>>,
    /// Photographic focal length.
    pub focal_length: Option<Animated<f32>>,
    /// Photographic aperture f-stop.
    pub f_stop: Option<Animated<f32>>,
}
/// External model attachment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attachment<'a> {
    /// Attached resource path.
    pub resource: Cow<'a, str>,
    /// Gameplay attachment-point identifier.
    pub id: u32,
    /// Visibility multiplier.
    pub visibility: Animated<f32>,
}
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
