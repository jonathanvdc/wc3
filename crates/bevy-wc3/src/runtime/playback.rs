use super::super::events::{EventClocks, EventTraversal};
use super::super::transition::TransitionRequest;
use super::super::PreparedModel;
use bevy::prelude::*;
use std::time::Duration;
use wc3::model::{AnimationSample, ClipId, Error as AnimationError, Time as ModelTime};

/// Playback clocks and clip selection, in seconds.
#[derive(Component, Clone, Debug)]
pub struct Playback {
    pub(crate) event_traversal: EventTraversal,
    pub(crate) transition_request: TransitionRequest,
    pub(crate) transition_clock: f64,
    /// Selected animation, or authored rest pose with global programmes.
    pub clip: Option<ClipId>,
    /// Elapsed local playback seconds, wrapped or clamped by clip policy.
    pub clip_time: f64,
    /// Elapsed seconds for independent repeating clocks.
    pub shared_time: f64,
    /// Playback rate; negative values run backwards.
    pub speed: f64,
    /// Whether automatic clock advancement is paused.
    pub paused: bool,
    /// Camera supplying billboard and camera-anchor poses.
    pub driving_camera: Option<Entity>,
    /// Selected authored LOD. Common geometry is always included.
    pub lod: u32,
}
impl Playback {
    /// Create default playback without exposing internal traversal state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a selected semantic clip from its beginning.
    /// The clip identity must belong to the model used by the instance.
    pub fn for_clip(clip: ClipId) -> Self {
        let mut playback = Self::default();
        playback.play(clip);
        playback
    }

    /// Restart a selected clip from its beginning, including start-time events.
    /// Identity validation occurs during instance evaluation.
    pub fn play(&mut self, clip: ClipId) {
        self.clip = Some(clip);
        self.paused = false;
        self.restart();
        self.transition_request.duration = None;
    }

    /// Restart a clip using an explicit local-pose fade duration.
    pub fn play_with_blend(&mut self, clip: ClipId, duration: Duration) {
        self.play(clip);
        self.transition_request.duration = Some(duration.as_secs_f64());
    }

    /// Restart a clip without a pose transition.
    pub fn play_immediately(&mut self, clip: ClipId) {
        self.play_with_blend(clip, Duration::ZERO);
    }

    /// Restart all instance clocks, including independent shared-clock events.
    pub fn restart(&mut self) {
        self.transition_request.revision = self.transition_request.revision.wrapping_add(1);
        self.transition_request.duration = Some(0.);
        self.transition_clock = 0.;
        self.clip_time = 0.;
        self.shared_time = 0.;
        self.event_traversal.revision = self.event_traversal.revision.wrapping_add(1);
        self.event_traversal.origin = EventClocks::default();
        self.event_traversal.include_origin = true;
    }

    /// Move all clocks without dispatching skipped events or destination keys.
    /// Direct clock-field edits instead retain ordinary forward traversal.
    pub fn seek(&mut self, seconds: f64) -> Result<(), AnimationError> {
        ModelTime::seconds(seconds)?;
        self.transition_request.revision = self.transition_request.revision.wrapping_add(1);
        self.transition_request.duration = Some(0.);
        self.transition_clock = 0.;
        self.clip_time = seconds;
        self.shared_time = seconds;
        self.event_traversal.revision = self.event_traversal.revision.wrapping_add(1);
        self.event_traversal.origin = EventClocks {
            clip: seconds,
            shared: seconds,
        };
        self.event_traversal.include_origin = false;
        Ok(())
    }
    pub(crate) fn blend_duration(&self, model: &PreparedModel) -> f64 {
        self.clip
            .and_then(|clip| model.timing.get(&clip))
            .and_then(|timing| timing.blend_duration)
            .unwrap_or(model.blend_time)
    }
    pub(crate) fn selection(&self) -> Result<AnimationSample, AnimationError> {
        Ok(AnimationSample {
            clip: self.clip,
            clip_time: ModelTime::seconds(self.clip_time)?,
            shared_time: ModelTime::seconds(self.shared_time)?,
        })
    }
}

impl Default for Playback {
    fn default() -> Self {
        Self {
            event_traversal: EventTraversal::default(),
            transition_request: TransitionRequest::default(),
            transition_clock: 0.,
            clip: None,
            clip_time: 0.,
            shared_time: 0.,
            speed: 1.,
            paused: false,
            driving_camera: None,
            lod: 0,
        }
    }
}

pub(crate) fn advance_animation(time: Res<Time>, mut query: Query<&mut Playback>) {
    for mut playback in &mut query {
        if !playback.paused {
            let delta = time.delta_secs_f64() * playback.speed;
            if delta == 0. {
                continue;
            }
            playback.transition_clock += delta.max(0.);
            playback.clip_time += delta;
            playback.shared_time += delta;
        }
    }
}
