//! Authored constraint solvers and influence volumes.
use crate::model::animation::programmed::PropertyBinding;
use crate::model::NodeId;
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
    pub length: PropertyBinding<f32>,
    /// Width-axis extent.
    pub width: PropertyBinding<f32>,
    /// Height-axis extent.
    pub height: PropertyBinding<f32>,
}
/// Animated half-size of a node influence volume.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewVolume {
    /// Half-size along each axis.
    pub size: PropertyBinding<[f32; 3]>,
}
