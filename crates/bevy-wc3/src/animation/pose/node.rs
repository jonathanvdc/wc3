use super::evaluation::NodeSample;
use crate::animation::Wc3Animation;
use bevy::prelude::*;
use wc3::model::animation::{Interpolate, Track};
use wc3::model::scene::NodeFlags;

/// An event key's exact authored clock position, including loop-end keys.
#[derive(Clone, Copy)]
pub(crate) struct NodeFrame {
    pub(crate) frame: i32,
    pub(crate) global_sequence_id: Option<u32>,
}

#[derive(Component)]
pub(crate) struct AnimatedNode {
    pub(crate) root: Entity,
    pub(crate) flags: NodeFlags,
    pub(crate) camera: Option<Transform>,
    pub(crate) pivot: Vec3,
    pub(crate) parent_pivot: Vec3,
    pub(crate) translation: Option<Track<[f32; 3]>>,
    pub(crate) rotation: Option<Track<[f32; 4]>>,
    pub(crate) scaling: Option<Track<[f32; 3]>>,
}

impl AnimatedNode {
    pub(crate) fn camera_anchor(&self) -> Option<Vec3> {
        self.flags
            .camera_anchored()
            .then_some(self.camera)
            .flatten()
            .map(|camera| camera.translation)
    }

    pub(crate) fn pose_sample(&self, camera: Option<Transform>) -> NodeSample {
        NodeSample {
            root: self.root,
            flags: self.flags,
            pivot_offset: self.pivot - self.parent_pivot,
            camera,
        }
    }

    pub(crate) fn sample_transform(&self, animation: &Wc3Animation) -> Transform {
        self.sample_transform_at_frame(animation, None)
    }

    /// An explicit frame preserves end-key poses at a loop boundary.
    /// Tracks on other clocks use the occurrence's elapsed time.
    pub(crate) fn sample_transform_at_frame(
        &self,
        animation: &Wc3Animation,
        frame: Option<NodeFrame>,
    ) -> Transform {
        let translation = self
            .translation
            .as_ref()
            .and_then(|track| sample_node_track(track, animation, frame))
            .unwrap_or([0.0; 3]);
        let rotation = self
            .rotation
            .as_ref()
            .and_then(|track| sample_node_track(track, animation, frame))
            .unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let scaling = self
            .scaling
            .as_ref()
            .and_then(|track| sample_node_track(track, animation, frame))
            .unwrap_or([1.0; 3]);
        Transform {
            translation: self.pivot - self.parent_pivot + Vec3::from_array(translation),
            rotation: Quat::from_xyzw(rotation[0], rotation[1], rotation[2], rotation[3]),
            scale: Vec3::from_array(scaling),
        }
    }
}

fn sample_node_track<T: Interpolate>(
    track: &Track<T>,
    animation: &Wc3Animation,
    frame: Option<NodeFrame>,
) -> Option<T> {
    if let Some(frame) =
        frame.filter(|frame| frame.global_sequence_id == track.global_sequence_id())
    {
        let (_, interval) = animation.time().track_time(track)?;
        track.evaluate_in(f64::from(frame.frame), interval)
    } else {
        track.sample(&animation.time())
    }
}
