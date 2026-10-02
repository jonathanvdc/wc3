//! Ordering contracts shared by the plugin and consuming applications.
use bevy::camera::CameraUpdateSystems;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

/// Integration points for WC3 instance loading, animation, and effects.
///
/// `SpawnInstances`, `BindTextures`, `AdvanceAnimation`, and `AnimateInstances`
/// run in `Update`. Spawning precedes the other sets; animation advances before
/// instance animation. Texture binding updates may run alongside animation.
///
/// The remaining sets run in `PostUpdate`: cameras precede node poses and model particles,
/// both precede transform propagation, and quad particles/ribbons follow it.
/// Order application systems in the same schedule as their target set.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Wc3Systems {
    SpawnInstances,
    BindTextures,
    AdvanceAnimation,
    AnimateInstances,
    AnimateCameras,
    EvaluateNodePoses,
    SimulateModelParticles,
    SimulateEffects,
}

pub(crate) fn configure(app: &mut App) {
    app.configure_sets(
        Update,
        (
            (
                Wc3Systems::SpawnInstances,
                Wc3Systems::AdvanceAnimation,
                Wc3Systems::AnimateInstances,
            )
                .chain(),
            Wc3Systems::BindTextures.after(Wc3Systems::SpawnInstances),
        ),
    );
    app.configure_sets(
        PostUpdate,
        (
            Wc3Systems::AnimateCameras.before(CameraUpdateSystems),
            (
                Wc3Systems::AnimateCameras,
                Wc3Systems::EvaluateNodePoses,
                Wc3Systems::SimulateModelParticles,
            )
                .chain()
                .before(TransformSystems::Propagate),
            Wc3Systems::SimulateEffects.after(TransformSystems::Propagate),
        ),
    );
}

#[cfg(test)]
#[path = "schedule_tests.rs"]
mod tests;
