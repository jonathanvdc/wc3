use super::super::events::{EventClocks, EventTraversal};
use super::super::transition::TransitionRequest;
use super::super::{ClipTiming, PreparedUniversalModel};
use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Duration;
use wc3::model::{
    AnimationSample, ClipId, ControllerId, Error as AnimationError, Time as ModelTime,
};

/// Controller programmes selected independently of playback clocks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum UniversalControllerSelection {
    /// Use the selected clip's authored programme list (including native M3 STG selections).
    #[default]
    Authored,
    /// Override authored programmes; an empty list explicitly disables all controllers.
    Explicit(Vec<ControllerId>),
}

/// Playback clocks and application-selected controller programmes, in seconds.
#[derive(Component, Clone, Debug)]
pub struct UniversalPlayback {
    pub(in crate::universal) event_traversal: EventTraversal,
    pub(in crate::universal) transition_request: TransitionRequest,
    pub(in crate::universal) transition_clock: f64,
    /// Selected animation, or authored rest pose with global programmes.
    pub clip: Option<ClipId>,
    /// Elapsed local playback seconds, wrapped or clamped by clip policy.
    pub clip_time: f64,
    /// Absolute authoring timeline seconds.
    pub timeline_time: f64,
    /// Elapsed seconds for independent repeating clocks.
    pub shared_time: f64,
    /// Playback rate; negative values run backwards.
    pub speed: f64,
    /// Whether automatic clock advancement is paused.
    pub paused: bool,
    /// Authored programmes or an explicit ordered override.
    pub controllers: UniversalControllerSelection,
    /// Camera supplying billboard and camera-anchor poses.
    pub driving_camera: Option<Entity>,
    /// Selected authored LOD. Common geometry is always included.
    pub lod: u32,
}
impl UniversalPlayback {
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
        self.timeline_time = 0.;
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
        self.timeline_time = seconds;
        self.shared_time = seconds;
        self.event_traversal.revision = self.event_traversal.revision.wrapping_add(1);
        self.event_traversal.origin = EventClocks {
            clip: seconds,
            timeline: seconds,
            shared: seconds,
        };
        self.event_traversal.include_origin = false;
        Ok(())
    }
    pub(in crate::universal) fn blend_duration(&self, model: &PreparedUniversalModel) -> f64 {
        self.clip
            .and_then(|clip| model.timing.get(&clip))
            .and_then(|timing| timing.blend_duration)
            .unwrap_or(model.blend_time)
    }
    pub(in crate::universal) fn selection<'a>(
        &'a self,
        model: &'a PreparedUniversalModel,
    ) -> Result<AnimationSample<'a>, AnimationError> {
        self.selection_from_timing(&model.timing)
    }
    pub(in crate::universal) fn selection_from_timing<'a>(
        &'a self,
        timings: &'a HashMap<ClipId, ClipTiming>,
    ) -> Result<AnimationSample<'a>, AnimationError> {
        let timing = self.clip.and_then(|id| timings.get(&id));
        let (timeline, interval) = timing.map_or((self.timeline_time, None), |timing| {
            let t = if timing.duration <= 0. {
                0.
            } else if timing.looping {
                self.clip_time.rem_euclid(timing.duration)
            } else {
                self.clip_time.clamp(0., timing.duration)
            };
            (
                timing.origin + t,
                Some((timing.origin, timing.origin + timing.duration)),
            )
        });
        Ok(AnimationSample {
            clip: self.clip,
            clip_time: ModelTime::seconds(self.clip_time)?,
            timeline_time: ModelTime::seconds(timeline)?,
            shared_time: ModelTime::seconds(self.shared_time)?,
            timeline_interval: interval
                .map(|(a, b)| Ok((ModelTime::seconds(a)?, ModelTime::seconds(b)?)))
                .transpose()?,
            controllers: match &self.controllers {
                UniversalControllerSelection::Authored => {
                    timing.map_or(&[], |timing| timing.controllers.as_ref())
                }
                UniversalControllerSelection::Explicit(controllers) => controllers,
            },
        })
    }
}

impl Default for UniversalPlayback {
    fn default() -> Self {
        Self {
            event_traversal: EventTraversal::default(),
            transition_request: TransitionRequest::default(),
            transition_clock: 0.,
            clip: None,
            clip_time: 0.,
            timeline_time: 0.,
            shared_time: 0.,
            speed: 1.,
            paused: false,
            controllers: UniversalControllerSelection::Authored,
            driving_camera: None,
            lod: 0,
        }
    }
}

pub(crate) fn advance_universal_animation(
    time: Res<Time>,
    mut query: Query<&mut UniversalPlayback>,
) {
    for mut playback in &mut query {
        if !playback.paused {
            let delta = time.delta_secs_f64() * playback.speed;
            if delta == 0. {
                continue;
            }
            playback.transition_clock += delta.max(0.);
            playback.clip_time += delta;
            playback.timeline_time += delta;
            playback.shared_time += delta;
        }
    }
}
