//! Shared aimed cameras keep projection conventions explicit.
use crate::model::profiles::{m2, m3, mdlx};
use crate::model::{scene, Animated, Link, ValidateLinks};
use m2::values::CameraKind;
/// Complete authored camera models.
#[derive(Clone, Debug)]
pub enum Camera<'a> {
    /// Portable photographic camera.
    Portable(Box<scene::Camera>),
    /// Camera with independently animated eye, target offset, and roll.
    Aimed(Box<AimedCamera>),
    /// Camera with versioned optical programs and property metadata.
    Optical(Box<m3::Camera<'a>>),
}
/// Shared model-space aim and displacement components.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AimedCamera<F = Animated<f32>, C = CameraConvention> {
    /// Eye displacement from the carrying node's rest translation.
    pub eye_offset: Animated<[f32; 3]>,
    /// Base model-space look-at target.
    pub target: [f32; 3],
    /// Displacement from the base target.
    pub target_offset: Animated<[f32; 3]>,
    /// Field of view in radians, interpreted by the projection convention.
    pub field_of_view: F,
    /// Near and far clipping distance.
    pub clipping: [f32; 2],
    /// Roll angle in radians.
    pub roll: Animated<f32>,
    /// Explicit projection and authored optional controls.
    pub convention: C,
}
/// Projection conventions with complete authored controls.
#[derive(Clone, Debug)]
pub enum CameraConvention {
    /// Vertical field of view, with visibility and photographic controls.
    Vertical(Box<PhotographicControls>),
    /// Diagonal field of view and an application camera role.
    Diagonal(CameraKind),
}
/// Visibility and optional photographic parameters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PhotographicControls {
    /// Camera visibility multiplier.
    pub visibility: Animated<f32>,
    /// Depth-of-field focus distance.
    pub focus_distance: Option<Animated<f32>>,
    /// Photographic focal length.
    pub focal_length: Option<Animated<f32>>,
    /// Photographic aperture f-stop.
    pub f_stop: Option<Animated<f32>>,
}
impl From<mdlx::Camera> for AimedCamera {
    fn from(v: mdlx::Camera) -> Self {
        Self {
            eye_offset: v.eye_offset,
            target: v.target,
            target_offset: v.target_offset,
            field_of_view: Animated::constant(v.field_of_view),
            clipping: v.clipping,
            roll: v.roll,
            convention: CameraConvention::Vertical(Box::new(v.convention)),
        }
    }
}
impl From<m2::Camera> for AimedCamera {
    fn from(v: m2::Camera) -> Self {
        Self {
            eye_offset: v.eye_offset,
            target: v.target,
            target_offset: v.target_offset,
            field_of_view: v.field_of_view,
            clipping: v.clipping,
            roll: v.roll,
            convention: CameraConvention::Diagonal(v.convention),
        }
    }
}
impl TryFrom<AimedCamera> for mdlx::Camera {
    type Error = Box<AimedCamera>;
    fn try_from(v: AimedCamera) -> Result<Self, Self::Error> {
        if v.field_of_view.binding.is_some() {
            return Err(Box::new(v));
        }
        let CameraConvention::Vertical(control) = v.convention else {
            return Err(Box::new(v));
        };
        Ok(Self {
            eye_offset: v.eye_offset,
            target: v.target,
            target_offset: v.target_offset,
            field_of_view: v.field_of_view.default,
            clipping: v.clipping,
            roll: v.roll,
            convention: *control,
        })
    }
}
impl TryFrom<AimedCamera> for m2::Camera {
    type Error = Box<AimedCamera>;
    fn try_from(v: AimedCamera) -> Result<Self, Self::Error> {
        let CameraConvention::Diagonal(kind) = v.convention else {
            return Err(Box::new(v));
        };
        Ok(Self {
            convention: kind,
            eye_offset: v.eye_offset,
            target: v.target,
            target_offset: v.target_offset,
            field_of_view: v.field_of_view,
            clipping: v.clipping,
            roll: v.roll,
        })
    }
}

impl<F: ValidateLinks, C: ValidateLinks> ValidateLinks for AimedCamera<F, C> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.eye_offset.visit_links(visitor);
        self.target_offset.visit_links(visitor);
        self.field_of_view.visit_links(visitor);
        self.roll.visit_links(visitor);
        self.convention.visit_links(visitor);
    }
}
impl ValidateLinks for PhotographicControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.visibility.visit_links(visitor);
        self.focus_distance.visit_links(visitor);
        self.focal_length.visit_links(visitor);
        self.f_stop.visit_links(visitor);
    }
}
impl ValidateLinks for CameraConvention {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::Vertical(value) = self {
            value.visit_links(visitor);
        }
    }
}
impl ValidateLinks for CameraKind {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
