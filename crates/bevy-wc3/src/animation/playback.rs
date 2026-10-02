use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use wc3::model::animation::{AnimationTime, Sequence};

/// Per-instance sequence playback and local-pose transitions.
/// Properties outside node transforms sample the selected sequence directly.
#[derive(Component, Clone)]
pub struct Wc3Animation {
    pub(crate) sequence: usize,
    pub(crate) elapsed_ms: f64,
    pub speed: f64,
    pub playing: bool,
    pub(crate) sequences: Vec<Sequence>,
    pub(crate) global_sequences: Vec<u32>,
    pub(crate) event_playback: EventPlayback,
    pub(crate) pose_playback: PosePlayback,
}

/// Authored local poses, before inheritance and camera-dependent corrections.
#[derive(Clone, Default)]
pub(crate) struct PosePlayback {
    pub(crate) blend_time: Duration,
    pub(crate) current: Arc<HashMap<Entity, Transform>>,
    transition: Option<PoseTransition>,
}

#[derive(Clone)]
struct PoseTransition {
    source: Arc<HashMap<Entity, Transform>>,
    duration_ms: f64,
    elapsed_ms: f64,
}

impl PosePlayback {
    pub(crate) fn new(blend_time: Duration) -> Self {
        Self {
            blend_time,
            ..Default::default()
        }
    }
}

/// The last explicit restart or seek, retained until event dispatch observes it.
#[derive(Clone)]
pub(crate) struct EventPlayback {
    pub(crate) revision: u64,
    pub(crate) origin_ms: f64,
    pub(crate) include_origin: bool,
}

impl Default for EventPlayback {
    fn default() -> Self {
        Self {
            revision: 0,
            origin_ms: 0.0,
            include_origin: true,
        }
    }
}

impl Wc3Animation {
    /// Selected sequence index. Use [`Self::play`] to change or restart it.
    pub fn sequence(&self) -> usize {
        self.sequence
    }

    /// Unwrapped elapsed playback time in milliseconds.
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed_ms
    }

    /// Returns the current model and global sequence sampling clocks.
    pub fn time(&self) -> AnimationTime<'_> {
        AnimationTime {
            sequence: self.sequences.get(self.sequence),
            elapsed_ms: self.elapsed_ms,
            global_sequences: &self.global_sequences,
        }
    }

    pub fn sequences(&self) -> &[Sequence] {
        &self.sequences
    }

    /// Starts or restarts a valid sequence using the model's BlendTime.
    /// Blends from the latest evaluated local pose into the advancing destination.
    /// Before the first pose evaluation, playback switches immediately.
    /// Only destination events dispatch, including its start-time events once.
    /// Global tracks share the destination clock and restart as well.
    pub fn play(&mut self, sequence: usize) {
        self.play_with_blend(sequence, self.pose_playback.blend_time);
    }

    /// Starts or restarts a sequence with an explicit pose transition duration.
    /// Repeated calls before pose evaluation retain the latest evaluated source;
    /// the last valid destination wins. Invalid indices leave playback untouched.
    /// The fade pauses with playback and advances at positive playback speed.
    /// Negative speed reverses the destination without reversing the fade.
    pub fn play_with_blend(&mut self, sequence: usize, duration: Duration) {
        if sequence < self.sequences.len() {
            self.pose_playback.transition = (!duration.is_zero()
                && !self.pose_playback.current.is_empty())
            .then(|| PoseTransition {
                source: self.pose_playback.current.clone(),
                duration_ms: duration.as_secs_f64() * 1000.0,
                elapsed_ms: 0.0,
            });
            self.sequence = sequence;
            self.elapsed_ms = 0.0;
            self.playing = true;
            self.reset_events(0.0, true);
        }
    }

    /// Starts or restarts a sequence without blending.
    pub fn play_immediately(&mut self, sequence: usize) {
        self.play_with_blend(sequence, Duration::ZERO);
    }

    /// Restarts the current clock, including start-time events once.
    /// Also works for global tracks in models without animation sequences.
    /// Cancels any transition and invalidates the cached pose until evaluation.
    pub fn restart(&mut self) {
        self.clear_pose();
        self.elapsed_ms = 0.0;
        self.playing = true;
        self.reset_events(0.0, true);
    }

    /// Moves the sampling clock without dispatching events crossed by the seek.
    /// Returns false for nonfinite times. Subsequent forward playback resumes
    /// event traversal strictly after this position, even before the next update.
    /// Cancels any transition and invalidates the cached pose until evaluation.
    pub fn seek(&mut self, elapsed_ms: f64) -> bool {
        if !elapsed_ms.is_finite() {
            return false;
        }
        self.elapsed_ms = elapsed_ms;
        self.clear_pose();
        self.reset_events(elapsed_ms, false);
        true
    }

    fn reset_events(&mut self, origin_ms: f64, include_origin: bool) {
        self.event_playback.revision = self.event_playback.revision.wrapping_add(1);
        self.event_playback.origin_ms = origin_ms;
        self.event_playback.include_origin = include_origin;
    }

    fn clear_pose(&mut self) {
        self.pose_playback.current = Arc::default();
        self.pose_playback.transition = None;
    }

    /// Retain the source past completion for within-update birth/event sampling.
    /// Storage remains bounded to one source pose and is replaced on the next play.
    pub(crate) fn blend_transform(&self, entity: Entity, destination: Transform) -> Transform {
        let Some(transition) = &self.pose_playback.transition else {
            return destination;
        };
        let weight = (transition.elapsed_ms / transition.duration_ms).clamp(0.0, 1.0) as f32;
        if weight >= 1.0 {
            return destination;
        }
        let Some(source) = transition.source.get(&entity) else {
            return destination;
        };
        Transform {
            translation: source.translation.lerp(destination.translation, weight),
            rotation: source
                .rotation
                .slerp(destination.rotation, weight)
                .normalize(),
            scale: source.scale.lerp(destination.scale, weight),
        }
    }

    pub(crate) fn preserves_particles(&self) -> bool {
        self.pose_playback.transition.is_some()
    }

    /// Reconstruct a within-update sample, including its transition progress.
    pub(crate) fn sample_at_elapsed(&self, elapsed_ms: f64) -> Self {
        let mut sample = self.clone();
        sample.set_sample_time(elapsed_ms);
        sample
    }

    /// Update a sampling snapshot without treating it as a playback discontinuity.
    pub(crate) fn set_sample_time(&mut self, elapsed_ms: f64) {
        if let Some(transition) = &mut self.pose_playback.transition {
            transition.elapsed_ms += elapsed_ms - self.elapsed_ms;
        }
        self.elapsed_ms = elapsed_ms;
    }
}

#[cfg(test)]
#[path = "playback_tests.rs"]
mod tests;

pub(crate) fn advance_animation(time: Res<Time>, mut instances: Query<&mut Wc3Animation>) {
    for mut instance in &mut instances {
        if instance.playing {
            let delta = time.delta_secs_f64() * 1000.0 * instance.speed;
            let elapsed = instance.elapsed_ms + delta;
            if elapsed.is_finite() {
                instance.elapsed_ms = elapsed;
                if let Some(transition) = &mut instance.pose_playback.transition {
                    transition.elapsed_ms += delta.max(0.0);
                }
            }
        }
    }
}
