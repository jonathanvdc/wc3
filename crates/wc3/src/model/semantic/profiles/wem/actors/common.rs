//! Common editable-model actors retain their full authored parameter sets.
use crate::model::semantic::{Animated, Link, ModelId, ValidateLinks};
use std::borrow::Cow;
/// Independently authored numeric and textual resource identities.
#[derive(Clone, Debug, Default)]
pub struct AssetAddress<'a> {
    /// Resource namespace, including game-specific numeric asset groups.
    pub namespace: u32,
    /// Numeric identity within that namespace, when declared.
    pub numeric: Option<u32>,
    /// Independently authored resource path; empty means no textual address.
    pub path: Cow<'a, [u8]>,
}
/// Spatial illumination category.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LightKind {
    /// Emits in all directions.
    #[default]
    Omnidirectional,
    /// Illuminates from one direction without spatial attenuation.
    Directional,
    /// Illuminates within a directional cone.
    Spot,
    /// Supplies ambient illumination.
    Ambient,
    /// Producer-defined illumination category.
    Unknown(u8),
}
/// Additional light-volume controls, when the authoring vocabulary declares them.
#[derive(Clone, Debug)]
pub struct LightVolume {
    /// Enables shadow casting.
    pub shadow_casting: bool,
    /// Near and far distances of the shadow volume.
    pub shadow_range: [f32; 2],
    /// Quadratic and linear distance attenuation coefficients.
    pub attenuation_coefficients: [f32; 2],
    /// Exponential distance attenuation coefficient.
    pub damping: f32,
}
/// Complete common light attached to a hierarchy node.
#[derive(Clone, Debug)]
pub struct Light {
    /// Illumination geometry.
    pub kind: LightKind,
    /// Light color multiplier.
    pub color: Animated<[f32; 3]>,
    /// Brightness multiplier.
    pub intensity: Animated<f32>,
    /// Distances at which attenuation starts and ends.
    pub attenuation: [Animated<f32>; 2],
    /// Inner and outer spotlight cone angles.
    pub cone: [f32; 2],
    /// Additional authoring volume controls.
    pub volume: Option<LightVolume>,
}
/// Authored depth-of-field controls.
#[derive(Clone, Debug)]
pub struct DepthOfField {
    /// Focus distance; zero retains the author's unset selection.
    pub focus_distance: f32,
    /// Focal-length parameter; no conversion to physical lens units is implied.
    pub focal_length: f32,
    /// Aperture parameter.
    pub aperture: f32,
}
/// Complete common viewpoint attached to its actual hierarchy owner.
#[derive(Clone, Debug)]
pub struct Camera {
    /// Authored field-of-view parameter; the producer's angular convention is retained.
    pub field_of_view: Animated<f32>,
    /// Near and far clipping distances.
    pub clipping: [Animated<f32>; 2],
    /// Authored model-space aim point, when declared.
    pub target: Option<Animated<[f32; 3]>>,
    /// Additional lens controls, when declared.
    pub depth_of_field: Option<DepthOfField>,
}
/// Resource and model references mounted at a hierarchy node.
#[derive(Clone, Debug)]
pub struct Attachment<'a> {
    /// Independently authored external asset address.
    pub external: AssetAddress<'a>,
    /// Referenced model in the owning model collection, when supplied.
    pub internal: Option<ModelId>,
}
/// Type of a common event command.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EventCategory {
    /// No event category is assigned.
    #[default]
    Unspecified,
    /// Sound playback.
    Sound,
    /// Transient ground mark.
    Splat,
    /// Persistent footprint.
    Footprint,
    /// Persistent large ground mark.
    LargeSplat,
    /// Spawn an external model.
    SpawnModel,
    /// Reveal a morph target.
    ShowMorph,
    /// Hide a morph target.
    HideMorph,
    /// Producer-defined command category.
    Unknown(u8),
}
/// Additional event classification and executable text, when declared.
#[derive(Clone, Debug)]
pub struct EventCode<'a> {
    /// Event interpretation.
    pub category: EventCategory,
    /// Counted executable text without an imposed language or terminator.
    pub code: Cow<'a, [u8]>,
}
/// A node-local event command whose occurrences belong to playback programs.
#[derive(Clone, Debug)]
pub struct Event<'a> {
    /// Game-defined resource or command identity.
    pub identity: u32,
    /// Additional command classification and source text.
    pub code: Option<EventCode<'a>>,
}
/// Common attached behavior admitted by the editable-model vocabulary.
#[derive(Clone, Debug)]
pub enum Feature<'a> {
    /// Illumination source.
    Light(Light),
    /// Viewpoint.
    Camera(Camera),
    /// Internal or external mounted actor.
    Attachment(Attachment<'a>),
    /// External particle-system instance.
    ParticleSystem(AssetAddress<'a>),
    /// External ribbon-system instance.
    RibbonSystem(AssetAddress<'a>),
    /// Event command declaration.
    Event(Event<'a>),
}
impl ValidateLinks for Feature<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Light(value) => {
                value.color.visit_links(visitor);
                value.intensity.visit_links(visitor);
                for field in &value.attenuation {
                    field.visit_links(visitor);
                }
                if value.cone.iter().any(|v| !v.is_finite())
                    || value.volume.as_ref().is_some_and(|v| {
                        v.shadow_range
                            .iter()
                            .chain(&v.attenuation_coefficients)
                            .chain([&v.damping])
                            .any(|v| !v.is_finite())
                    })
                {
                    visitor(Link::InvalidValue);
                }
            }
            Self::Camera(value) => {
                value.field_of_view.visit_links(visitor);
                for field in &value.clipping {
                    field.visit_links(visitor);
                }
                value.target.visit_links(visitor);
                if value.depth_of_field.as_ref().is_some_and(|v| {
                    [v.focus_distance, v.focal_length, v.aperture]
                        .iter()
                        .any(|v| !v.is_finite())
                }) {
                    visitor(Link::InvalidValue);
                }
            }
            Self::Attachment(value) => {
                value.external.visit_links(visitor);
                if let Some(model) = value.internal {
                    visitor(Link::Model(model));
                }
            }
            Self::ParticleSystem(value) | Self::RibbonSystem(value) => value.visit_links(visitor),
            Self::Event(_) => {}
        }
    }
}

impl ValidateLinks for AssetAddress<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.numeric == Some(u32::MAX) {
            visitor(Link::InvalidValue);
        }
    }
}
