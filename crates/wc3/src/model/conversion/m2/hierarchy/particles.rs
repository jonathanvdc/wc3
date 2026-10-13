//! Particles imported into active WEM hierarchy records.
use super::*;

pub(super) fn import_particles<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut sm::Model<'static>,
    patches: &mut Vec<NodePatch>,
) -> Result<(), Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let core = source.model.core();
    for (index, particle) in core.particles.iter().enumerate() {
        let node = model.nodes.len();
        model.nodes.push(sm::Node {
            name: Cow::Owned(format!("particle-{}", particle.id)),
            parent: Some(u32::from(particle.bone_id)),
            pivot: particle.position,
            ..Default::default()
        });
        patches.push(NodePatch {
            particle: Some(index),
            texts: Vec::new(),
            index: node,
            kind: wr::NodeKind::ParticleEmitter,
            payload: wr::NodePayload::Helper(Box::default()),
            metadata: D::Layout::legacy_texture_scale(particle).map_or_else(
                Vec::new,
                |[kind, head_tail]| {
                    vec![
                        ("m2.particle_type", i64::from(kind)),
                        ("m2.head_or_tail", i64::from(head_tail)),
                    ]
                },
            ),
        });
        for sequence in 0..model.clips.len() {
            let start = core
                .sequences
                .get(sequence)
                .map_or(0, |sequence| sequence.timing.window().0);
            for (property, track) in [
                (1000, &particle.emission_speed),
                (1001, &particle.speed_variation),
                (1002, &particle.vertical_range),
                (1003, &particle.horizontal_range),
                (1005, &particle.lifespan),
                (1006, &particle.emission_rate),
                (1007, &particle.emission_area_width),
                (1008, &particle.emission_area_length),
                (1009, &particle.z_source),
            ] {
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
            if let Some(track) = import_track_try(
                source,
                &particle.gravity,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: node as u32,
                    channel: 1004,
                },
                |v| gravity_vector(v, particle.flags.packed_gravity()).map(|v| v.to_vec()),
            )? {
                model.clips[sequence].tracks.push(track);
            }
            if let Some(track) = import_track(
                source,
                &particle.enabled,
                &source.key,
                sequence,
                start,
                sm::Target::Property {
                    owner: node as u32,
                    channel: 1010,
                },
                |v| vec![f32::from(v)],
            )? {
                model.clips[sequence].tracks.push(track);
            }
        }
    }
    Ok(())
}
