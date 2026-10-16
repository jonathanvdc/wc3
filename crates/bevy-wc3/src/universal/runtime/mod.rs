//! Per-instance playback, spawning, pose evaluation and event dispatch.
mod effects;
mod evaluation;
mod event_dispatch;
mod instance;
mod playback;
mod spawn;

pub(crate) use effects::simulate_universal_particles;
pub(crate) use evaluation::{animate_universal_instances, apply_universal_lod_visibility};
pub(crate) use event_dispatch::dispatch_universal_events;
pub use instance::{
    UniversalAnimationError, UniversalDraw, UniversalInstance, UniversalNode,
    UniversalPoseOverrides,
};
pub(crate) use playback::advance_universal_animation;
pub use playback::{UniversalControllerSelection, UniversalPlayback};
pub(in crate::universal) use spawn::populate_universal_model;
pub use spawn::spawn_universal_model;
