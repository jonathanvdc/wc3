//! Lights imported into active WEM hierarchy records.
use super::*;

pub(super) fn import_lights<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut sm::Model<'static>,
    patches: &mut Vec<NodePatch>,
) -> Result<(), Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let core = source.model.core();
    for (index, light) in core.lights.iter().enumerate() {
        let node = model.nodes.len();
        model.nodes.push(sm::Node {
            name: Cow::Owned(format!("light-{index}")),
            parent: (light.bone_id >= 0).then_some(light.bone_id as u32),
            pivot: light.position,
            ..Default::default()
        });
        patches.push(NodePatch {
            particle: None,
            texts: Vec::new(),
            index: node,
            kind: wr::NodeKind::LIGHT,
            payload: wr::NodePayload::Light(Box::new(wr::LightPayload {
                kind: if light.kind == 0 {
                    wr::LightKind::DIRECTIONAL
                } else {
                    wr::LightKind::OMNI
                },
                color: initial(&light.diffuse_color).unwrap_or([1.; 3]),
                intensity: initial(&light.diffuse_intensity).unwrap_or(1.),
                attenuation_start: initial(&light.attenuation_start).unwrap_or(0.),
                attenuation_end: initial(&light.attenuation_end).unwrap_or(0.),
                ..Default::default()
            })),
            metadata: vec![("m2.light.kind", i64::from(light.kind))],
        });
        for sequence in 0..model.clips.len() {
            let start = core
                .sequences
                .get(sequence)
                .map_or(0, |sequence| sequence.timing.window().0);
            for (channel, track) in [
                (wr::Channel::INTENSITY.0, &light.diffuse_intensity),
                (wr::Channel::ATTENUATION_START.0, &light.attenuation_start),
                (wr::Channel::ATTENUATION_END.0, &light.attenuation_end),
                (201, &light.ambient_intensity),
            ] {
                if let Some(track) = import_track(
                    source,
                    track,
                    &source.key,
                    sequence,
                    start,
                    sm::Target::Property {
                        owner: node as u32,
                        channel: u32::from(channel),
                    },
                    |v| vec![v],
                )? {
                    model.clips[sequence].tracks.push(track);
                }
            }
            for (channel, track) in [
                (wr::Channel::COLOR.0, &light.diffuse_color),
                (200, &light.ambient_color),
            ] {
                if let Some(track) = import_track(
                    source,
                    track,
                    &source.key,
                    sequence,
                    start,
                    sm::Target::Property {
                        owner: node as u32,
                        channel: u32::from(channel),
                    },
                    |v| v.to_vec(),
                )? {
                    model.clips[sequence].tracks.push(track);
                }
            }
            if let Some(track) = import_track(
                source,
                &light.visibility,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: node as u32,
                    channel: u32::from(wr::Channel::VISIBILITY.0),
                },
                |v| vec![f32::from(v)],
            )? {
                model.clips[sequence].tracks.push(track);
            }
        }
    }
    Ok(())
}
