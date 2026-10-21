//! Per-instance playback, spawning, pose evaluation and event dispatch.
mod affine;
mod current_affine;
mod effects;
mod evaluation;
mod event_dispatch;
mod instance;
pub(crate) mod model_driver;
mod model_step_order;
mod mounts;
mod playback;
mod spawn;

pub(crate) use affine::publish_affine_poses;
pub(crate) use effects::simulate_particles;
pub(crate) use evaluation::{animate_instances, apply_lod_visibility};
pub(crate) use event_dispatch::dispatch_events;
pub use instance::{ModelDraw, ModelNode, ModelRig, PlaybackError, PoseOverrides};
pub(crate) use model_driver::simulate_model_particles;
pub(crate) use mounts::{synchronize_model_cameras, synchronize_mounted_models};
pub(crate) use playback::advance_animation;
pub use playback::Playback;
pub(crate) use spawn::populate_model_with_context;
pub use spawn::spawn_model;
