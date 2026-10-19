//! Logical cameras, illumination, resource systems, and attachments.
use crate::model::features::IndexedChildModel;
use crate::model::Construct;
use crate::model::{EffectResourceId, ModelResourceId};
use crate::model::{Link, Property, ValidateLinks};
use glam::{Vec2, Vec3};
/// A portable feature attached to a graph hierarchy node.
#[derive(Clone, Debug)]
pub enum Feature {
    /// Authored camera parameters.
    Camera(Box<Camera>),
    /// Authored illumination.
    Light(Box<Light>),
    /// External model attachment.
    Attachment(Attachment),
    /// External particle-system instance.
    Particles(EffectResourceId),
    /// External ribbon-system instance.
    Ribbon(EffectResourceId),
    /// External face-animation actor.
    FaceActor(EffectResourceId),
}
impl ValidateLinks for Feature {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Attachment(value) => value.visit_links(visitor),
            Self::Camera(value) => {
                value.field_of_view.visit_links(visitor);
                value.target.visit_links(visitor);
                value.focus_distance.visit_links(visitor);
                value.focal_length.visit_links(visitor);
                value.f_stop.visit_links(visitor);
            }
            Self::Light(value) => {
                value.color.visit_links(visitor);
                value.intensity.visit_links(visitor);
                for parameter in &value.attenuation {
                    parameter.visit_links(visitor);
                }
                for parameter in &value.cone {
                    parameter.visit_links(visitor);
                }
                if let Some(range) = &value.shadow_range {
                    for parameter in range {
                        parameter.visit_links(visitor);
                    }
                }
            }
            Self::Particles(value) | Self::Ribbon(value) | Self::FaceActor(value) => {
                value.visit_links(visitor)
            }
        }
    }
}
/// Camera and photographic parameters with explicit optional values.
#[derive(Clone, Debug, Construct)]
pub struct Camera {
    /// Authored field of view in radians.
    pub field_of_view: Property<f32>,
    /// Near and far clipping distance.
    pub clipping: Vec2,
    /// Aim point in model coordinates.
    pub target: Option<Property<Vec3>>,
    /// Depth-of-field focus distance.
    pub focus_distance: Option<Property<f32>>,
    /// Photographic focal length.
    pub focal_length: Option<Property<f32>>,
    /// Photographic aperture f-stop.
    pub f_stop: Option<Property<f32>>,
}
/// Portable illumination behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LightKind {
    /// Point light.
    #[default]
    Point,
    /// Directional light.
    Directional,
    /// Spot light.
    Spot,
    /// Ambient light.
    Ambient,
    /// Unresolved authored kind.
    Unknown(u32),
}
/// Light emission, range, cone, and shadow parameters.
#[derive(Clone, Debug, Construct)]
pub struct Light {
    /// Illumination behavior.
    pub kind: LightKind,
    /// Animated RGB color.
    pub color: Property<Vec3>,
    /// Animated intensity multiplier.
    pub intensity: Property<f32>,
    /// Range-attenuation start and end.
    pub attenuation: [Property<f32>; 2],
    /// Inner and outer cone angles.
    pub cone: [Property<f32>; 2],
    /// Explicit shadow-casting setting.
    pub shadow_casting: Option<bool>,
    /// Shadow-volume near and far distances.
    pub shadow_range: Option<[Property<f32>; 2]>,
    /// Quadratic, linear, and exponential attenuation coefficients.
    pub attenuation_coefficients: [Option<f32>; 3],
}
/// External attachment resource and animation visibility.
pub type Attachment = IndexedChildModel<ModelResourceId>;
