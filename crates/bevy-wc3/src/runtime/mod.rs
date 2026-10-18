//! Per-instance playback, spawning, pose evaluation and event dispatch.
mod effects;
mod evaluation;
mod event_dispatch;
mod instance;
mod mounts;
mod playback;
mod spawn;

pub(crate) use effects::simulate_particles;
pub(crate) use evaluation::{animate_instances, apply_lod_visibility};
pub(crate) use event_dispatch::dispatch_events;
pub use instance::{ModelDraw, ModelNode, ModelRig, PlaybackError, PoseOverrides};
pub(crate) use mounts::{synchronize_model_cameras, synchronize_mounted_models};
pub(crate) use playback::advance_animation;
pub use playback::Playback;
pub(crate) use spawn::populate_model;
pub use spawn::spawn_model;
