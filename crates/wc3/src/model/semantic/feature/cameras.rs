//! Shared aimed cameras keep projection conventions explicit.
use crate::model::semantic::profiles::{m2, m3, mdlx};
use crate::model::semantic::{scene, Animated};
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
#[derive(Clone, Debug)]
pub struct AimedCamera {
    /// Eye displacement from the carrying node's rest translation.
    pub eye_offset: Animated<[f32; 3]>,
    /// Base model-space look-at target.
    pub target: [f32; 3],
    /// Displacement from the base target.
    pub target_offset: Animated<[f32; 3]>,
    /// Field of view in radians, interpreted by the projection convention.
    pub field_of_view: Animated<f32>,
    /// Near and far clipping distance.
    pub clipping: [f32; 2],
    /// Roll angle in radians.
    pub roll: Animated<f32>,
    /// Explicit projection and authored optional controls.
    pub convention: CameraConvention,
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
#[derive(Clone, Debug)]
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
            convention: CameraConvention::Vertical(Box::new(PhotographicControls {
                visibility: v.visibility,
                focus_distance: v.focus_distance,
                focal_length: v.focal_length,
                f_stop: v.f_stop,
            })),
        }
    }
}
impl From<m2::Camera> for AimedCamera {
    fn from(v: m2::Camera) -> Self {
        Self {
            eye_offset: v.position,
            target: v.target_base,
            target_offset: v.target,
            field_of_view: v.field_of_view,
            clipping: v.clip,
            roll: v.roll,
            convention: CameraConvention::Diagonal(v.kind),
        }
    }
}
impl TryFrom<AimedCamera> for mdlx::Camera {
    type Error = Box<AimedCamera>;
    fn try_from(v: AimedCamera) -> Result<Self, Self::Error> {
        if v.field_of_view.property.is_some() {
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
            visibility: control.visibility,
            focus_distance: control.focus_distance,
            focal_length: control.focal_length,
            f_stop: control.f_stop,
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
            kind,
            position: v.eye_offset,
            target_base: v.target,
            target: v.target_offset,
            field_of_view: v.field_of_view,
            clip: v.clipping,
            roll: v.roll,
        })
    }
}
