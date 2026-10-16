//! Dense local-pose transitions, applied before hierarchy and camera corrections.
use wc3::model::Transform as ModelTransform;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TransitionRequest {
    pub revision: u64,
    /// None selects the model's authored default duration.
    pub duration: Option<f64>,
}
#[derive(Default)]
pub(super) struct TransitionState {
    revision: u64,
    current: Vec<ModelTransform>,
    source: Vec<ModelTransform>,
    duration: f64,
    elapsed: f64,
    active: bool,
}
#[derive(Clone, Copy)]
pub(super) struct TransitionView<'a> {
    source: &'a [ModelTransform],
    duration: f64,
    elapsed: f64,
}
impl TransitionView<'_> {
    pub fn at_offset(mut self, seconds: f64) -> Self {
        self.elapsed += seconds;
        self
    }
    pub fn blend(&self, index: usize, destination: ModelTransform) -> ModelTransform {
        if self.elapsed >= self.duration {
            return destination;
        }
        let Some(source) = self.source.get(index) else {
            return destination;
        };
        let t = (self.elapsed / self.duration).clamp(0., 1.) as f32;
        source.blend(destination, t)
    }
}
impl TransitionState {
    pub fn needs_update(&self, request: TransitionRequest, elapsed: f64) -> bool {
        self.revision != request.revision || (self.active && self.elapsed != elapsed)
    }
    pub fn prepare(
        &mut self,
        request: TransitionRequest,
        elapsed: f64,
        default_duration: f64,
        node_count: usize,
    ) {
        if self.revision != request.revision {
            self.revision = request.revision;
            self.duration = request.duration.unwrap_or(default_duration).max(0.);
            self.active = self.duration > 0. && !self.current.is_empty();
            if self.active {
                self.source.clone_from(&self.current);
            }
        }
        self.current.resize(node_count, ModelTransform::default());
        self.elapsed = elapsed;
    }
    pub fn view(&self) -> Option<TransitionView<'_>> {
        if self.elapsed >= self.duration {
            return None;
        }
        self.view_at(self.elapsed)
    }
    pub fn preview(
        &self,
        request: TransitionRequest,
        elapsed: f64,
        default_duration: f64,
    ) -> Option<TransitionView<'_>> {
        if self.revision == request.revision {
            return if elapsed < self.duration {
                self.view_at(elapsed)
            } else {
                None
            };
        }
        let duration = request.duration.unwrap_or(default_duration);
        (duration > 0. && elapsed < duration && !self.current.is_empty()).then_some(
            TransitionView {
                source: &self.current,
                duration,
                elapsed,
            },
        )
    }
    // Occurrence samplers rewind this view after it is constructed. Retain its
    // source even after the current frame has passed the transition endpoint.
    pub fn view_at(&self, elapsed: f64) -> Option<TransitionView<'_>> {
        (self.duration > 0. && !self.source.is_empty()).then_some(TransitionView {
            source: &self.source,
            duration: self.duration,
            elapsed,
        })
    }
    pub fn blend(&self, index: usize, destination: ModelTransform) -> ModelTransform {
        self.view()
            .map_or(destination, |view| view.blend(index, destination))
    }
    pub fn record(&mut self, index: usize, pose: ModelTransform) {
        if index == self.current.len() {
            self.current.push(pose);
        } else {
            self.current[index] = pose;
        }
    }
    pub fn finish(&mut self) {
        if self.elapsed >= self.duration {
            self.active = false;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transitions_snapshot_latest_local_pose_and_preserve_destination_pivot() {
        let mut state = TransitionState::default();
        state.record(
            0,
            ModelTransform {
                translation: [2., 0., 0.],
                ..Default::default()
            },
        );
        state.prepare(
            TransitionRequest {
                revision: 1,
                duration: Some(2.),
            },
            1.,
            0.,
            1,
        );
        let destination = ModelTransform {
            translation: [10., 0., 0.],
            pivot: [0., 0., 3.],
            ..Default::default()
        };
        let midpoint = state.blend(0, destination);
        assert_eq!(midpoint.translation, [6., 0., 0.]);
        assert_eq!(midpoint.pivot, destination.pivot);
        state.record(0, midpoint);
        state.prepare(
            TransitionRequest {
                revision: 2,
                duration: Some(1.),
            },
            0.,
            0.,
            1,
        );
        assert_eq!(state.blend(0, destination).translation, [6., 0., 0.]);
        assert_eq!(
            state
                .view()
                .unwrap()
                .at_offset(0.5)
                .blend(0, destination)
                .translation,
            [8., 0., 0.]
        );
    }
    #[test]
    fn completed_fades_skip_current_work_but_retain_historical_poses() {
        let mut state = TransitionState::default();
        state.record(
            0,
            ModelTransform {
                translation: [2., 0., 0.],
                ..Default::default()
            },
        );
        let request = TransitionRequest {
            revision: 1,
            duration: Some(2.),
        };
        state.prepare(request, 2., 0., 1);
        state.finish();
        assert!(!state.active);
        assert!(state.view().is_none());
        assert!(state.preview(request, 3., 0.).is_none());
        assert!(state
            .preview(
                TransitionRequest {
                    revision: 2,
                    duration: Some(2.)
                },
                2.,
                0.
            )
            .is_none());
        let destination = ModelTransform {
            translation: [10., 0., 0.],
            ..Default::default()
        };
        assert_eq!(
            state.view_at(1.).unwrap().blend(0, destination).translation,
            [6., 0., 0.]
        );
        assert_eq!(
            state
                .view_at(3.)
                .unwrap()
                .at_offset(-2.)
                .blend(0, destination)
                .translation,
            [6., 0., 0.]
        );
        assert_eq!(
            state.blend(0, destination).translation,
            destination.translation
        );
    }
}
