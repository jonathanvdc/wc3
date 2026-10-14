//! Complete M3 behaviors attached to ordinary semantic nodes.
use super::{
    Attachment, AttachmentVolume, BillboardBehavior, Camera, EventCommand, ExtentBinding, Force,
    HitTestShape, Light, OneBoneSolver, Particle, ParticleCopy, Projector, Property, Ribbon,
    TurretBehavior, Warp,
};
use crate::model::semantic::NodeId;

/// A single M3 behavior; the shared graph stores its owning node independently.
#[derive(Clone, Debug, PartialEq)]
pub enum Feature<'a> {
    /// A timed engine command referenced by an event occurrence.
    Event(Box<EventCommand<'a>>),
    /// A structured animated spatial extent.
    Extent(Box<ExtentBinding>),
    /// Independent particle emission with linked secondary emitters.
    Particle(Box<Particle<'a>>),
    /// Particle instance with two authored overrides and an inherited source definition.
    ParticleCopy(Box<ParticleCopy>),
    /// Ribbon emission.
    Ribbon(Box<Ribbon<'a>>),
    /// Scene illumination.
    Light(Box<Light>),
    /// Named camera.
    Camera(Box<Camera<'a>>),
    /// Material projection.
    Projector(Box<Projector>),
    /// Force field.
    Force(Box<Force>),
    /// Vertex deformation field.
    Warp(Box<Warp>),
    /// Named attachment.
    Attachment(Box<Attachment<'a>>),
    /// Hit-test volume.
    HitTest(Box<HitTestShape>),
    /// Attachment volume with additional bone links.
    AttachmentVolume(Box<AttachmentVolume>),
    /// Camera-dependent orientation.
    Billboard(Box<BillboardBehavior>),
    /// Turret rotation constraints.
    Turret(Box<TurretBehavior>),
    /// Two-bone ground-contact inverse kinematics.
    IkJoint(Box<IkJoint>),
    /// Two-joint inverse kinematics with angular constraints.
    IkTwoJoint(Box<IkTwoJoint>),
    /// Cyclic-coordinate-descent inverse kinematics.
    IkCcd(Box<IkCcd>),
    /// Single-bone solver.
    OneBoneSolver(Box<OneBoneSolver>),
    /// Animated shadow bounds.
    ShadowBox(Box<ShadowBox>),
    /// Animated influence volume.
    ViewVolume(Box<ViewVolume>),
}
/// Ground-contact inverse kinematics for two bone links.
#[derive(Clone, Debug, PartialEq)]
pub struct IkJoint {
    /// Nodes depending on this solver.
    pub dependents: Vec<NodeId>,
    /// Second linked bone.
    pub second: NodeId,
    /// Positive raycast offset.
    pub raycast_up: f32,
    /// Signed lower raycast offset.
    pub raycast_down: f32,
    /// Maximum solve speed.
    pub max_speed: f32,
    /// Goal-distance threshold.
    pub goal_threshold: f32,
}
/// Angularly constrained two-joint solver.
#[derive(Clone, Debug, PartialEq)]
pub struct IkTwoJoint {
    /// Nodes depending on this solver.
    pub dependents: Vec<NodeId>,
    /// Intermediate joint.
    pub target: NodeId,
    /// Endpoint of the chain.
    pub end: NodeId,
    /// Unresolved native solver control.
    pub reserved: u16,
    /// Hinge rotation axis.
    pub hinge_axis: [f32; 3],
    /// Maximum inner angle.
    pub max_angle_inner: f32,
    /// Maximum outer angle.
    pub max_angle_outer: f32,
    /// Upward search range.
    pub search_up: f32,
    /// Downward search range.
    pub search_down: f32,
}
/// Cyclic-coordinate-descent inverse kinematics.
#[derive(Clone, Debug, PartialEq)]
pub struct IkCcd {
    /// Nodes depending on this solver.
    pub dependents: Vec<NodeId>,
    /// Target bone.
    pub target: NodeId,
    /// Upward search range.
    pub search_up: f32,
    /// Downward search range.
    pub search_down: f32,
}
/// Animated extents of a node's shadow volume.
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowBox {
    /// Unresolved native shadow control word.
    pub reserved: u16,
    /// Length-axis extent.
    pub length: Property<f32>,
    /// Width-axis extent.
    pub width: Property<f32>,
    /// Height-axis extent.
    pub height: Property<f32>,
}
/// Animated half-size of a node influence volume.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewVolume {
    /// Half-size along each axis.
    pub size: Property<[f32; 3]>,
}
