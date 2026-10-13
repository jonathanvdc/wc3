//! Ribbons imported into active WEM hierarchy records.
use super::*;

pub(super) fn import_ribbons<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut sm::Model<'static>,
    patches: &mut Vec<NodePatch>,
) -> Result<(), Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let core = source.model.core();
    for (index, ribbon) in core.ribbons.iter().enumerate() {
        let node = model.nodes.len();
        model.nodes.push(sm::Node {
            name: Cow::Owned(format!("ribbon-{}", ribbon.id)),
            parent: Some(ribbon.bone_id),
            pivot: ribbon.position,
            ..Default::default()
        });
        let tail = D::Layout::ribbon_tail(&ribbon.modern);
        patches.push(NodePatch {
            particle: None,
            texts: vec![
                (
                    "m2.ribbon.textures",
                    ribbon
                        .texture_indices
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                ),
                (
                    "m2.ribbon.materials",
                    ribbon
                        .material_indices
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(","),
                ),
            ],
            index: node,
            kind: wr::NodeKind::RibbonEmitter,
            payload: wr::NodePayload::Wc3RibbonEmitter(Box::new(wr::Wc3RibbonEmitterPayload {
                height_above: initial(&ribbon.height_above).unwrap_or(0.),
                height_below: initial(&ribbon.height_below).unwrap_or(0.),
                alpha: f32::from(initial(&ribbon.alpha).unwrap_or(32767)) / 32767.,
                color: initial(&ribbon.color).unwrap_or([1.; 3]),
                lifespan: ribbon.edge_lifetime,
                texture_slot: u32::from(initial(&ribbon.texture_slot).unwrap_or(0)),
                emission_rate: ribbon.edges_per_second.max(0.) as u32,
                rows: u32::from(ribbon.texture_rows),
                columns: u32::from(ribbon.texture_columns),
                material_slot: u32::MAX,
                gravity: ribbon.gravity,
            })),
            metadata: vec![
                ("m2.ribbon.id", i64::from(ribbon.id)),
                (
                    "m2.ribbon.rate",
                    i64::from(ribbon.edges_per_second.to_bits()),
                ),
                ("m2.ribbon.priority", i64::from(tail[0])),
                ("m2.ribbon.color", i64::from(tail[1])),
                ("m2.ribbon.transform", i64::from(tail[2])),
            ],
        });
        for sequence in 0..model.clips.len() {
            let start = core
                .sequences
                .get(sequence)
                .map_or(0, |sequence| sequence.timing.window().0);
            for (property, track) in [(1100, &ribbon.height_above), (1101, &ribbon.height_below)] {
                if let Some(track) = import_track(
                    source,
                    track,
                    &source.key,
                    sequence,
                    start,
                    sm::Target::Property {
                        owner: node as u32,
                        channel: property,
                    },
                    |v| vec![v],
                )? {
                    model.clips[sequence].tracks.push(track);
                }
            }
            if let Some(track) = import_track(
                source,
                &ribbon.color,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: node as u32,
                    channel: 1102,
                },
                |v| v.to_vec(),
            )? {
                model.clips[sequence].tracks.push(track);
            }
            if let Some(track) = import_track(
                source,
                &ribbon.alpha,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: node as u32,
                    channel: 1103,
                },
                |v| vec![f32::from(v) / 32767.],
            )? {
                model.clips[sequence].tracks.push(track);
            }
            if let Some(track) = import_track(
                source,
                &ribbon.texture_slot,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: node as u32,
                    channel: 1104,
                },
                |v| vec![f32::from(v)],
            )? {
                model.clips[sequence].tracks.push(track);
            }
            if let Some(track) = import_track(
                source,
                &ribbon.visibility,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: node as u32,
                    channel: 1105,
                },
                |v| vec![f32::from(v)],
            )? {
                model.clips[sequence].tracks.push(track);
            }
        }
        let _ = index;
    }
    Ok(())
}
