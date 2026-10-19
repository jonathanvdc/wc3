//! Camera pose, projection, and optical parameters.
use crate::model::features::{FromFeature, TryFromFeature};
use crate::model::profiles::{m2, mdlx};
use crate::model::{Construct, Link, Property, PropertyValues, ValidateLinks};
use glam::{Vec2, Vec3};
use m2::values::CameraKind;

/// Camera pose interpretations.
#[derive(Clone, Debug)]
pub enum Camera {
    /// Camera whose eye and orientation follow its hierarchy node.
    Node(Box<NodeCamera>),
    /// Independently animated model-space eye, target, and roll.
    LookAt(Box<LookAtCamera>),
}
/// Axis along which a perspective field-of-view angle is measured.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldOfViewAxis {
    /// Angle spanning the image height.
    #[default]
    Vertical,
    /// Angle spanning the image diagonal.
    Diagonal,
}
/// Model-space aim and displacement with independent projection and optical parameters.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct LookAtCamera<F = Property<f32>> {
    /// Eye displacement from the carrying node's rest translation.
    pub eye_offset: Property<Vec3>,
    /// Base model-space look-at target.
    pub target: Vec3,
    /// Displacement from the base target.
    pub target_offset: Property<Vec3>,
    /// Perspective projection, independent of camera placement.
    pub projection: PerspectiveProjection<F>,
    /// Roll angle in radians.
    pub roll: Property<f32>,
    /// Optional application camera role.
    pub role: Option<CameraKind>,
    /// Optional authored visibility multiplier; absence means always visible.
    pub visibility: Option<Property<f32>>,
    /// Independently authored optical parameters.
    pub lens: CameraLens,
}
/// Optional photographic lens parameters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CameraLens {
    /// Depth-of-field focus distance.
    pub focus_distance: Option<Property<f32>>,
    /// Photographic focal length.
    pub focal_length: Option<Property<f32>>,
    /// Photographic aperture f-stop.
    pub f_stop: Option<Property<f32>>,
}
impl FromFeature<mdlx::Camera> for LookAtCamera {
    fn from_feature(v: mdlx::Camera, values: &mut PropertyValues) -> Self {
        Self {
            eye_offset: v.eye_offset,
            target: v.target,
            target_offset: v.target_offset,
            projection: v.projection.map_field_of_view(|angle| values.add(*angle)),
            roll: v.roll,
            role: v.role,
            visibility: v.visibility,
            lens: v.lens,
        }
    }
}
impl TryFromFeature<LookAtCamera> for mdlx::Camera {
    type Error = Box<LookAtCamera>;
    fn try_from_feature(v: LookAtCamera, values: &PropertyValues) -> Result<Self, Self::Error> {
        if !v.is_mdlx() {
            return Err(Box::new(v));
        }
        Ok(Self {
            eye_offset: v.eye_offset,
            target: v.target,
            target_offset: v.target_offset,
            projection: v
                .projection
                .map_field_of_view(|angle| *values.get(*angle).expect("validated source slot")),
            roll: v.roll,
            role: v.role,
            visibility: v.visibility,
            lens: v.lens,
        })
    }
}
impl<F> LookAtCamera<F> {
    pub(crate) fn is_mdlx(&self) -> bool {
        self.projection.axis == FieldOfViewAxis::Vertical && self.role.is_none()
    }
    pub(crate) fn is_m2(&self) -> bool {
        self.projection.axis == FieldOfViewAxis::Diagonal
            && self.role.is_some()
            && self.visibility.is_none()
            && self.lens == CameraLens::default()
    }
}
impl<F: ValidateLinks> ValidateLinks for LookAtCamera<F> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.eye_offset.visit_links(visitor);
        self.target_offset.visit_links(visitor);
        self.projection.visit_links(visitor);
        self.roll.visit_links(visitor);
        self.visibility.visit_links(visitor);
        self.lens.visit_links(visitor);
        if self
            .target
            .to_array()
            .iter()
            .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for CameraLens {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.focus_distance.visit_links(visitor);
        self.focal_length.visit_links(visitor);
        self.f_stop.visit_links(visitor);
    }
}

/// Camera whose eye and orientation follow its carrying hierarchy node.
#[derive(Clone, Debug, Construct)]
pub struct NodeCamera {
    /// Perspective projection, independent of camera placement.
    pub projection: PerspectiveProjection,
    /// Optional aim point in model coordinates.
    pub target: Option<Property<Vec3>>,
    /// Optional photographic parameters.
    pub lens: CameraLens,
}
impl ValidateLinks for NodeCamera {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.projection.visit_links(visitor);
        self.target.visit_links(visitor);
        self.lens.visit_links(visitor);
    }
}

/// A perspective angle, its measurement axis, and authored clipping distances.
#[derive(Clone, Copy, Debug, Construct, PartialEq)]
pub struct PerspectiveProjection<F = Property<f32>> {
    /// Field of view in radians.
    pub field_of_view: F,
    /// Axis along which the angle is measured.
    pub axis: FieldOfViewAxis,
    /// Near and far clipping distance.
    pub clipping: Vec2,
}
impl<F> PerspectiveProjection<F> {
    /// Maps the authored angle while preserving its axis and clipping distances.
    pub fn map_field_of_view<G>(&self, map: impl FnOnce(&F) -> G) -> PerspectiveProjection<G> {
        PerspectiveProjection {
            field_of_view: map(&self.field_of_view),
            axis: self.axis,
            clipping: self.clipping,
        }
    }
}
impl PerspectiveProjection<f32> {
    /// Returns the vertical angle for a viewport width-to-height ratio.
    pub fn vertical_field_of_view(&self, aspect: f32) -> f32 {
        match self.axis {
            FieldOfViewAxis::Vertical => self.field_of_view,
            FieldOfViewAxis::Diagonal => {
                2. * ((self.field_of_view * 0.5).tan() / (aspect * aspect + 1.).sqrt()).atan()
            }
        }
    }
}
impl<F: ValidateLinks> ValidateLinks for PerspectiveProjection<F> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.field_of_view.visit_links(visitor);
        if self
            .clipping
            .to_array()
            .iter()
            .any(|value| !value.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
