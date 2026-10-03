//! Ordering contracts shared by the plugin and consuming applications.
use bevy::camera::visibility::VisibilitySystems;
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
/// both precede transform propagation, and quad particles/ribbons and event
/// dispatch follow it.
/// Order application systems in the same schedule as their target set.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Wc3Systems {
    /// Prepares loaded models and populates pending instance roots in `Update`.
    SpawnInstances,
    /// Applies changed quad-particle texture bindings in `Update`.
    BindTextures,
    /// Advances instance playback clocks and pose transitions in `Update`.
    AdvanceAnimation,
    /// Samples materials, lights, attachments, and child-particle animation in `Update`.
    AnimateInstances,
    /// Plays authored views on bound application cameras in `PostUpdate`.
    AnimateCameras,
    /// Evaluates animated node transforms before propagation in `PostUpdate`.
    EvaluateNodePoses,
    /// Spawns and advances Classic model particles before transform propagation.
    SimulateModelParticles,
    /// Samples quad-particle births and ribbon sections after transform propagation.
    SimulateEffects,
    /// Publishes model event messages after transform propagation in `PostUpdate`.
    DispatchEvents,
    /// Selects geometry LOD after camera/transform updates, before visibility.
    SelectLod,
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
            Wc3Systems::DispatchEvents.after(TransformSystems::Propagate),
            Wc3Systems::SelectLod
                .after(CameraUpdateSystems)
                .after(TransformSystems::Propagate)
                .before(VisibilitySystems::VisibilityPropagate)
                .before(VisibilitySystems::CheckVisibility),
        ),
    );
}

#[cfg(test)]
#[path = "schedule_tests.rs"]
mod tests;
