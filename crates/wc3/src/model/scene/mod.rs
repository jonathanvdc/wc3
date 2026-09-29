//! Model metadata and objects in the transform hierarchy.
//!
//! [`Node`] supplies a name, object ID, parent reference, and transform animation.
//! Bones, attachments, lights, and other scene objects build on that hierarchy.
//! Cameras define a view and target independently. Object IDs also index the
//! model pivot-point collection.
mod node;
pub(crate) use node::{impl_node_flags, set_node_kind, validate_node_kind};
pub use node::{Bone, Node, NodeFlagInterpretation, NodeFlags};
mod camera;
pub use camera::{Camera, CameraLayout, CameraVariant};
mod light;
pub use light::{Light, LightFalloff, LightLayout, LightShadowRange};
mod attachment;
pub use attachment::Attachment;
mod event;
pub use event::EventObject;
mod face_fx;
pub use face_fx::FaceFx;
mod model_info;
pub use model_info::ModelInfo;

pub use camera::CameraTrack;
pub use light::LightTrack;
pub use node::NodeTrack;

mod glider;
pub use glider::Glider;
