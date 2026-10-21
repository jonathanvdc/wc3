//! Ordering contracts shared by the plugin and consuming applications.
use bevy::camera::visibility::VisibilitySystems;
use bevy::camera::CameraUpdateSystems;
use bevy::prelude::*;
use bevy::transform::TransformSystems;

/// Integration points for WC3 instance loading, animation, and effects.
///
/// `SpawnInstances`, `BindTextures`, `AdvanceAnimation`, and `AnimateInstances`
/// run in `Update`. Spawning precedes the other sets; animation advances before
/// the `AnimateInstances` application hook. Authored pose, material, and feature
/// sampling occurs later in `PostUpdate`. Texture binding updates may run
/// alongside clock advancement.
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
    /// Synchronizes mounted equipment and advances playback clocks in `Update`.
    AdvanceAnimation,
    /// Application hook after clock advancement in `Update`; authored sampling
    /// occurs in the PostUpdate pose and model-particle stages.
    AnimateInstances,
    /// Plays authored views on bound application cameras in `PostUpdate`.
    AnimateCameras,
    /// Evaluates animation and optional secondary motion before propagation.
    EvaluateNodePoses,
    /// Advances MDLX and M2 model-particle sources, then samples their owned
    /// actor subtrees and ready births before transform propagation.
    SimulateModelParticles,
    /// Samples quad-particle births and ribbon sections after transform propagation.
    SimulateEffects,
    /// Publishes exact affine bone corrections after normal transform propagation.
    PublishAffinePoses,
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
            Wc3Systems::PublishAffinePoses
                .after(TransformSystems::Propagate)
                .before(Wc3Systems::SimulateEffects)
                .before(Wc3Systems::DispatchEvents)
                .before(Wc3Systems::SelectLod)
                .before(CameraUpdateSystems),
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
