//! Cameras imported into active WEM hierarchy records.
use super::*;

pub(super) fn import_cameras<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut sm::Model<'static>,
    patches: &mut Vec<NodePatch>,
) -> Result<(), Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let core = source.model.core();
    for (index, camera) in core.cameras.iter().enumerate() {
        let node = model.nodes.len();
        model.nodes.push(sm::Node {
            name: Cow::Owned(format!("camera-{index}")),
            pivot: camera.position_base,
            ..Default::default()
        });
        patches.push(NodePatch {
            particle: None,
            texts: Vec::new(),
            index: node,
            kind: wr::NodeKind::CAMERA,
            payload: wr::NodePayload::Camera(Box::new(wr::CameraPayload {
                fov: D::Layout::static_fov(camera),
                near_clip: camera.near_clip,
                far_clip: camera.far_clip,
                target: camera.target_position_base,
                ..Default::default()
            })),
            metadata: vec![("m2.camera.kind", i64::from(camera.kind))],
        });
        for sequence in 0..model.clips.len() {
            let start = core
                .sequences
                .get(sequence)
                .map_or(0, |s| s.timing.window().0);
            for (property, track) in [(1200, &camera.positions), (1201, &camera.target_positions)] {
                if let Some(track) = import_spline(
                    source,
                    track,
                    sequence,
                    start,
                    sm::Target::Property {
                        owner: node as u32,
                        channel: property,
                    },
                    |v| v.to_vec(),
                )? {
                    model.clips[sequence].tracks.push(track);
                }
            }
            if let Some(track) = import_spline(
                source,
                &camera.roll,
                sequence,
                start,
                sm::Target::Property {
                    owner: node as u32,
                    channel: 1202,
                },
                |v| vec![v],
            )? {
                model.clips[sequence].tracks.push(track);
            }
            if let Some(track) = D::Layout::import_fov(
                source,
                camera,
                sequence,
                start,
                sm::Target::Property {
                    owner: node as u32,
                    channel: 1203,
                },
            )? {
                model.clips[sequence].tracks.push(track);
            }
        }
    }
    Ok(())
}
