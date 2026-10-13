//! Events imported into active WEM hierarchy records.
use super::*;

pub(super) fn import_events<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut sm::Model<'static>,
    patches: &mut Vec<NodePatch>,
) -> Result<(), Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let core = source.model.core();
    for event in &core.events {
        let node = model.nodes.len();
        model.nodes.push(sm::Node {
            name: Cow::Owned(format!(
                "event-{:08x}",
                u32::from_le_bytes(event.identifier)
            )),
            parent: Some(event.bone_id),
            pivot: event.position,
            ..Default::default()
        });
        patches.push(NodePatch {
            particle: None,
            texts: Vec::new(),
            index: node,
            kind: wr::NodeKind::Event,
            payload: wr::NodePayload::Event(Box::new(wr::EventPayload {
                id: event.data,
                ..Default::default()
            })),
            metadata: vec![(
                "m2.event.identifier",
                i64::from(u32::from_le_bytes(event.identifier)),
            )],
        });
        for sequence in 0..model.clips.len() {
            let global = event.enabled.global_sequence != u16::MAX;
            let index = if global || D::VERSION < 264 {
                0
            } else {
                sequence
            };
            let Some(times) = event.enabled.timeline.leaf(index) else {
                continue;
            };
            let times = resolve_array(source, times, sequence)?;
            let range = (!global)
                .then(|| event.enabled.timeline.ranges().get(sequence))
                .flatten()
                .map(|[a, b]| a as usize..=b as usize);
            let start = core
                .sequences
                .get(sequence)
                .map_or(0, |seq| seq.timing.window().0);
            let keys = times
                .into_iter()
                .enumerate()
                .filter(|(index, _)| range.as_ref().is_none_or(|range| range.contains(index)))
                .map(|(_, time)| {
                    Ok(sm::Key {
                        time: if global {
                            time
                        } else {
                            time.checked_sub(start).ok_or_else(|| {
                                Error::new("events.time", "event precedes sequence start")
                            })?
                        } as f32
                            / 1000.,
                        value: vec![1.],
                        incoming: None,
                        outgoing: None,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            if !keys.is_empty() {
                let global_duration = if global {
                    Some(
                        core.global_sequences
                            .get(usize::from(event.enabled.global_sequence))
                            .ok_or_else(|| Error::new("events.clock", "missing global duration"))?
                            as f32
                            / 1000.,
                    )
                } else {
                    None
                };
                model.clips[sequence].tracks.push(sm::Track {
                    layer: Default::default(),
                    target: sm::Target::Property {
                        owner: node as u32,
                        channel: 1400,
                    },
                    interpolation: sm::Interpolation::Step,
                    global_duration,
                    keys,
                });
            }
        }
    }
    Ok(())
}
