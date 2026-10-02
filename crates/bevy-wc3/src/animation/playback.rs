use bevy::prelude::*;
use wc3::model::animation::{AnimationTime, Sequence};

#[derive(Component, Clone)]
pub struct Wc3Animation {
    pub(crate) sequence: usize,
    pub(crate) elapsed_ms: f64,
    pub speed: f64,
    pub playing: bool,
    pub(crate) sequences: Vec<Sequence>,
    pub(crate) global_sequences: Vec<u32>,
    pub(crate) event_playback: EventPlayback,
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

    /// Starts or restarts a valid sequence, including its start-time events once.
    /// Global tracks share this clock and restart as well.
    pub fn play(&mut self, sequence: usize) {
        if sequence < self.sequences.len() {
            self.sequence = sequence;
            self.restart();
        }
    }

    /// Restarts the current clock, including start-time events once.
    /// Also works for global tracks in models without animation sequences.
    pub fn restart(&mut self) {
        self.elapsed_ms = 0.0;
        self.playing = true;
        self.reset_events(0.0, true);
    }

    /// Moves the sampling clock without dispatching events crossed by the seek.
    /// Returns false for nonfinite times. Subsequent forward playback resumes
    /// event traversal strictly after this position, even before the next update.
    pub fn seek(&mut self, elapsed_ms: f64) -> bool {
        if !elapsed_ms.is_finite() {
            return false;
        }
        self.elapsed_ms = elapsed_ms;
        self.reset_events(elapsed_ms, false);
        true
    }

    fn reset_events(&mut self, origin_ms: f64, include_origin: bool) {
        self.event_playback.revision = self.event_playback.revision.wrapping_add(1);
        self.event_playback.origin_ms = origin_ms;
        self.event_playback.include_origin = include_origin;
    }
}

pub(crate) fn advance_animation(time: Res<Time>, mut instances: Query<&mut Wc3Animation>) {
    for mut instance in &mut instances {
        if instance.playing {
            let elapsed = instance.elapsed_ms + time.delta_secs_f64() * 1000.0 * instance.speed;
            if elapsed.is_finite() {
                instance.elapsed_ms = elapsed;
            }
        }
    }
}
