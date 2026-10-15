//! Static local transforms supplied as constant curves where no clip overrides them.
use super::*;
pub(in super::super) fn rest_tracks(model: &mut sm::Model<'_>) {
    for (index, node) in model.nodes.iter().enumerate() {
        for (target, value, default) in [
            (
                sm::Target::Translation(index as u32),
                node.translation.to_vec(),
                vec![0.; 3],
            ),
            (
                sm::Target::Rotation(index as u32),
                node.rotation.to_vec(),
                vec![0., 0., 0., 1.],
            ),
            (
                sm::Target::Scale(index as u32),
                node.scale.to_vec(),
                vec![1.; 3],
            ),
        ] {
            if value == default {
                continue;
            }
            let mode = model
                .clips
                .iter()
                .flat_map(|clip| &clip.tracks)
                .find(|track| track.target == target)
                .map_or(sm::Interpolation::Step, |track| track.interpolation);
            for clip in &mut model.clips {
                if !clip.tracks.iter().any(|track| track.target == target) {
                    let tangent = match mode {
                        sm::Interpolation::Hermite => Some(vec![0.; value.len()]),
                        sm::Interpolation::Bezier => Some(value.clone()),
                        _ => None,
                    };
                    clip.tracks.push(sm::Track {
                        target,
                        interpolation: mode,
                        global_duration: None,
                        keys: vec![sm::Key {
                            time: 0.,
                            value: value.clone(),
                            incoming: tangent.clone(),
                            outgoing: tangent,
                        }],
                        layer: Default::default(),
                    });
                }
            }
        }
    }
}
