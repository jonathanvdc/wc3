//! Owner-local Diablo III authoring declarations contain no native document backing.
use crate::model::{Bounds, Error, Link, MaterialSlotId, NodeId, Transform, ValidateLinks};
use std::borrow::Cow;

/// Model identity and appearance-wide declarations.
#[derive(Clone, Debug, Default)]
pub struct ModelMetadata {
    /// Source appearance resource identity, independent of its filename.
    pub sno_id: i32,
    /// Authored object classification, including unresolved codes.
    pub object_type: i32,
    /// Independently authored collision sphere center.
    pub collision_center: [f32; 3],
    /// Independently authored collision sphere radius.
    pub collision_radius: f32,
    /// Look-at controller declarations; no simulation is implied by storage.
    pub look_at: LookAtControls,
    /// Unclassified appearance-wide vector.
    pub unclassified_vector: [f32; 3],
    /// Unclassified signed scalar declarations in native declaration order.
    pub unclassified_integers: [i32; 11],
    /// Unclassified wide signed declarations in native declaration order.
    pub unclassified_wide_integers: [i64; 2],
    /// Unclassified appearance-wide floating scalar.
    pub unclassified_scalar: f32,
}
impl ValidateLinks for ModelMetadata {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self
            .collision_center
            .iter()
            .chain(self.unclassified_vector.iter())
            .chain([&self.collision_radius, &self.unclassified_scalar])
            .any(|v| !v.is_finite())
            || self.collision_radius < 0.
        {
            visitor(Link::InvalidValue);
        }
        self.look_at.visit_links(visitor);
    }
}
/// Actual hierarchy relationship addressed by a look-at controller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LookAtTarget {
    /// Actual skeleton bone; native output derives its current name.
    Bone(NodeId),
    /// Unresolved authored spelling retained only while the controller is disabled.
    Unresolved(String),
}
impl Default for LookAtTarget {
    fn default() -> Self {
        Self::Unresolved(String::new())
    }
}
impl ValidateLinks for LookAtTarget {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::Bone(id) = self {
            id.visit_links(visitor);
        }
    }
}
/// Authored look-at behavior, independent of a native root layout.
#[derive(Clone, Debug, Default)]
pub struct LookAtControls {
    /// Native enable category, retaining unresolved values.
    pub enabled: i32,
    /// Actual skeleton relationship, or an inactive unresolved declaration.
    pub target: LookAtTarget,
    /// Minimum and maximum yaw.
    pub yaw: [f32; 2],
    /// Minimum and maximum pitch.
    pub pitch: [f32; 2],
    /// Angular velocity.
    pub angular_velocity: f32,
    /// Unclassified authored scalar.
    pub unclassified_scalar: f32,
}
impl ValidateLinks for LookAtControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.target.visit_links(visitor);
        if self.enabled != 0 && matches!(self.target, LookAtTarget::Unresolved(_)) {
            visitor(Link::InvalidValue);
        }

        if self
            .yaw
            .iter()
            .chain(self.pitch.iter())
            .chain([&self.angular_velocity, &self.unclassified_scalar])
            .any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
/// Additional bone bind poses, independently authored from the node's local rest pose.
#[derive(Clone, Debug)]
pub struct BoneControls {
    /// Model-space bind pose A.
    pub bind_pose: Transform,
    /// Inverse of the authored A bind pose; edits do not silently recompute it.
    pub inverse_bind_pose: Transform,
    /// Independently authored model-space bind pose B.
    pub alternate_bind_pose: Transform,
    /// Independently authored inverse B bind pose.
    pub inverse_alternate_bind_pose: Transform,
    /// Authored bone-local culling bounds.
    pub bounds: Bounds,
    /// Center of the independently authored sphere.
    pub sphere_center: [f32; 3],
    /// Radius of the independently authored sphere.
    pub sphere_radius: f32,
    /// External particle resource attached to this bone; negative codes remain authored.
    pub particle: i32,
}
impl Default for BoneControls {
    fn default() -> Self {
        Self {
            bind_pose: Transform::default(),
            inverse_bind_pose: Transform::default(),
            alternate_bind_pose: Transform::default(),
            inverse_alternate_bind_pose: Transform::default(),
            bounds: Bounds::default(),
            sphere_center: [0.; 3],
            sphere_radius: 0.,
            particle: -1,
        }
    }
}
impl ValidateLinks for BoneControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if [
            self.bind_pose,
            self.inverse_bind_pose,
            self.alternate_bind_pose,
            self.inverse_alternate_bind_pose,
        ]
        .iter()
        .any(|pose| pose.validate().is_err())
            || self.bounds.validate().is_err()
            || self.sphere_center.iter().any(|v| !v.is_finite())
            || !self.sphere_radius.is_finite()
            || self.sphere_radius < 0.
        {
            visitor(Link::InvalidValue);
        }
    }
}
/// Hierarchy role distinguishing skeleton bones from the implicit model frame and hardpoints.
#[derive(Clone, Debug, Default)]
pub enum NodeRole<'a> {
    /// Implicit model-space placement frame.
    #[default]
    ModelSpace,
    /// Authored skeleton bone with independent bind declarations.
    Bone(Box<BoneControls>),
    /// Named attachment frame whose actual parent is carried by the hierarchy.
    Hardpoint {
        /// Original attachment name.
        name: Cow<'a, str>,
    },
}
impl ValidateLinks for NodeRole<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::Bone(value) = self {
            value.visit_links(visitor);
        }
    }
}
/// Subobject behavior attached to shared geometry.
#[derive(Clone, Debug, Default)]
pub struct MeshControls {
    /// Vertex behavior qualifiers independent of the logical vertex streams.
    pub vertex_flags: i32,
    /// Native material-slot name addressed by this subobject.
    pub material_slot: Option<MaterialSlotId>,
    /// Maya shape and wardrobe selection spelling, independent of shader slot.
    pub shape_name: String,
    /// External surface resource, including negative sentinel values.
    pub surface: i32,
    /// Bone-relative subobject placement when declared.
    pub bone: Option<NodeId>,
    /// Authored scalar whose meaning is unresolved.
    pub unclassified_control: f32,
}
impl ValidateLinks for MeshControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.bone.visit_links(visitor);
        self.material_slot.visit_links(visitor);
        if !self.unclassified_control.is_finite() {
            visitor(Link::InvalidValue);
        }
    }
}
/// Selection and timing declarations for one animation permutation.
#[derive(Clone, Debug)]
pub struct ClipControls {
    /// Source animation resource identity.
    pub sno_id: i32,
    /// Appearance resource named by the source animation.
    pub appearance_sno_id: i32,
    /// Ordered animation bone palette, independent of skeleton storage order.
    pub bone_order: Vec<NodeId>,
    /// Authored root-motion samples in native sample order.
    pub root_motion: Vec<[f32; 3]>,
    /// Independently authored in-place root-motion samples.
    pub root_motion_in_place: Vec<[f32; 3]>,
    /// Authored hardpoint name associated with playback.
    pub hardpoint_name: String,
    /// Authored hardpoint selector, whose relation to the name is not inferred.
    pub hardpoint_index: i32,
    /// Animation-wide options.
    pub animation_flags: [i32; 2],
    /// Source animation name hash, independent of the permutation's display name.
    pub name_hash: i32,
    /// Permutation options.
    pub flags: i32,
    /// Authored frame rate; keys use seconds in the common program.
    pub frames_per_second: f64,
    /// Number of authored animation frames.
    pub frame_count: u32,
    /// Frames per translation, rotation, and scale key.
    pub frames_per_key: [f32; 3],
    /// Cross-animation and within-permutation blend durations, in engine ticks.
    pub blend_ticks: [i32; 2],
    /// Relative permutation selection weight.
    pub selection_weight: i32,
    /// Authored playback speed multiplier.
    pub speed_scalar: f32,
    /// Independently authored marker frames.
    pub marker_frames: [f32; 4],
    /// Independently authored marker positions.
    pub marker_points: [[f32; 3]; 4],
    /// Authored event frame.
    pub event_frame: f32,
    /// Model-space movement velocity.
    pub movement_velocity: [f32; 3],
    /// Unclassified authored scalar controls, in native declaration order.
    pub unclassified_controls: [f32; 4],
}
impl Default for ClipControls {
    fn default() -> Self {
        Self {
            sno_id: -1,
            appearance_sno_id: -1,
            bone_order: Vec::new(),
            root_motion: Vec::new(),
            root_motion_in_place: Vec::new(),
            hardpoint_name: String::new(),
            hardpoint_index: 0,
            animation_flags: [0; 2],
            name_hash: 0,
            flags: 0,
            frames_per_second: 60.,
            frame_count: 0,
            frames_per_key: [1.; 3],
            blend_ticks: [0; 2],
            selection_weight: 0,
            speed_scalar: 1.,
            marker_frames: [0.; 4],
            marker_points: [[0.; 3]; 4],
            event_frame: 0.,
            movement_velocity: [0.; 3],
            unclassified_controls: [0.; 4],
        }
    }
}
impl ValidateLinks for ClipControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.bone_order.visit_links(visitor);
        if self
            .root_motion
            .iter()
            .chain(self.root_motion_in_place.iter())
            .flatten()
            .any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        if !self.frames_per_second.is_finite()
            || self.frames_per_second <= 0.
            || self
                .frames_per_key
                .iter()
                .chain(self.marker_frames.iter())
                .chain(self.marker_points.iter().flatten())
                .chain(self.movement_velocity.iter())
                .chain(self.unclassified_controls.iter())
                .chain([&self.speed_scalar, &self.event_frame])
                .any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
/// Rejects transforms that require a nonuniform scale or pivot absent from Diablo's PRS records.
pub(crate) fn uniform_transform(transform: &Transform) -> Result<(), Error> {
    transform.validate()?;
    if transform.scale[0] != transform.scale[1]
        || transform.scale[0] != transform.scale[2]
        || transform.pivot != [0.; 3]
    {
        return Err(Error::new(
            "d3.transform",
            "Diablo III PRS transforms require uniform scale and no pivot",
        ));
    }
    Ok(())
}
