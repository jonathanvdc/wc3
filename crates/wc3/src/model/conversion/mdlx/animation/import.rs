//! Import mappings for animation.
use super::*;
pub(in crate::model::conversion::mdlx) fn property_scalar(
    track: &Track<'_, f32>,
    owner: u32,
    tag: [u8; 4],
    start: f32,
    globals: &[u32],
) -> s::Track {
    let data = read_track(track);
    semantic_track(
        TrackData {
            interpolation: data.interpolation,
            global: data.global,
            keys: data
                .keys
                .into_iter()
                .map(|(time, value, tangents)| {
                    (
                        time,
                        [value],
                        tangents.map(|(incoming, outgoing)| ([incoming], [outgoing])),
                    )
                })
                .collect(),
        },
        s::Target::Property {
            owner,
            channel: u32::from_le_bytes(tag),
        },
        start,
        globals,
    )
}

pub(in crate::model::conversion::mdlx) fn property_color(
    track: &Track<'_, [f32; 3]>,
    owner: u32,
    tag: [u8; 4],
    start: f32,
    globals: &[u32],
) -> s::Track {
    semantic_track(
        read_track(track),
        s::Target::Property {
            owner,
            channel: u32::from_le_bytes(tag),
        },
        start,
        globals,
    )
}

pub(in crate::model::conversion::mdlx) fn import_properties<D: ModelDialect>(
    source: &native::Model<'_, D>,
    model: &mut s::Model<'_>,
    globals: &[u32],
    context: &mut Context,
) -> Result<(), Error> {
    let identities: BTreeMap<_, _> = source
        .nodes()
        .iter()
        .enumerate()
        .map(|(index, node)| (node.object_id, index as u32))
        .collect();
    for clip in &mut model.clips {
        let start = clip.id as f32 / 1000.;
        macro_rules! scalar {
            ($owner:expr,$tag:expr,$value:expr) => {
                if let Some(track) = $value.track() {
                    clip.tracks
                        .push(property_scalar(track, $owner, *$tag, start, globals));
                }
            };
        }
        macro_rules! option {
            ($owner:expr,$tag:expr,$value:expr) => {
                if let Some(track) = $value {
                    clip.tracks
                        .push(property_scalar(track, $owner, *$tag, start, globals));
                }
            };
        }
        macro_rules! color {
            ($owner:expr,$tag:expr,$value:expr) => {
                if let Some(track) = $value.track() {
                    clip.tracks
                        .push(property_color(track, $owner, *$tag, start, globals));
                }
            };
        }
        for chunk in &source.chunks {
            match chunk {
                ModelChunk::ParticleEmitters(chunk) => {
                    for record in &chunk.records {
                        let owner = identities[&record.node.object_id];
                        scalar!(owner, b"KPEE", record.emission_rate);
                        scalar!(owner, b"KPEG", record.gravity);
                        scalar!(owner, b"KPLN", record.longitude);
                        scalar!(owner, b"KPLT", record.latitude);
                        scalar!(owner, b"KPEL", record.life_span);
                        scalar!(owner, b"KPES", record.initial_velocity);
                        option!(owner, b"KPEV", &record.visibility);
                    }
                }
                ModelChunk::ParticleEmitters2(chunk) => {
                    for record in &chunk.records {
                        let owner = identities[&record.node.object_id];
                        scalar!(owner, b"KP2S", record.speed);
                        scalar!(owner, b"KP2R", record.variation);
                        scalar!(owner, b"KP2L", record.latitude);
                        scalar!(owner, b"KP2G", record.gravity);
                        scalar!(owner, b"KP2E", record.emission_rate);
                        scalar!(owner, b"KP2W", record.width);
                        scalar!(owner, b"KP2N", record.length);
                        option!(owner, b"KP2V", &record.visibility);
                    }
                }
                ModelChunk::RibbonEmitters(chunk) => {
                    for record in &chunk.records {
                        let owner = identities[&record.node.object_id];
                        scalar!(owner, b"KRHA", record.height_above);
                        scalar!(owner, b"KRHB", record.height_below);
                        scalar!(owner, b"KRAL", record.alpha);
                        color!(owner, b"KRCO", record.color);
                        option!(owner, b"KRVS", &record.visibility);
                    }
                }
                ModelChunk::PopcornEmitters(chunk) => {
                    for record in &chunk.records {
                        let owner = identities[&record.node.object_id];
                        scalar!(owner, b"KPPL", record.life_span);
                        scalar!(owner, b"KPPE", record.emission_rate);
                        scalar!(owner, b"KPPS", record.speed);
                        scalar!(owner, b"KPPA", record.alpha);
                        color!(owner, b"KPPC", record.color);
                        option!(owner, b"KPPV", &record.visibility);
                    }
                }
                ModelChunk::Attachments(chunk) => {
                    for record in &chunk.records {
                        option!(
                            identities[&record.node.object_id],
                            b"KATV",
                            &record.visibility
                        );
                    }
                }
                ModelChunk::Lights(chunk) => {
                    for record in &chunk.records {
                        let owner = identities[&record.node.object_id];
                        scalar!(owner, b"KLAS", record.attenuation_start);
                        scalar!(owner, b"KLAE", record.attenuation_end);
                        color!(owner, b"KLAC", record.color);
                        scalar!(owner, b"KLAI", record.intensity);
                        color!(owner, b"KLBC", record.ambient_color);
                        scalar!(owner, b"KLBI", record.ambient_intensity);
                        option!(owner, b"KLAV", &record.visibility);
                    }
                }
                _ => {}
            }
        }
        for (index, camera) in source.cameras().enumerate() {
            let owner = identities.len() as u32 + index as u32;
            option!(owner, b"KCRL", &camera.rotation);
            option!(owner, b"KCVS", &camera.visibility);
            option!(owner, b"KCFD", &camera.focus_distance);
            option!(owner, b"KCFL", &camera.focal_length);
            option!(owner, b"KCFS", &camera.f_stop);
            if let Some(track) = &camera.target_translation {
                clip.tracks
                    .push(property_color(track, owner, *b"KTTR", start, globals));
            }
        }
        for (index, animation) in source.texture_animations().enumerate() {
            let owner = 0x2000_0000 | index as u32;
            if let Some(track) = &animation.translation {
                clip.tracks
                    .push(property_color(track, owner, *b"KTAT", start, globals));
            }
            if let Some(track) = &animation.rotation {
                clip.tracks.push(semantic_track(
                    read_track(track),
                    s::Target::Property {
                        owner,
                        channel: u32::from_le_bytes(*b"KTAR"),
                    },
                    start,
                    globals,
                ));
            }
            if let Some(track) = &animation.scaling {
                clip.tracks
                    .push(property_color(track, owner, *b"KTAS", start, globals));
            }
        }
        for animation in source.geoset_animations() {
            let owner = 0x4000_0000 | animation.geoset_id;
            scalar!(owner, b"KGAO", animation.alpha);
            color!(owner, b"KGAC", animation.color);
        }
        for record in source.ribbon_emitters() {
            if let Some(track) = record.texture_slot.track() {
                clip.integer_tracks.push(property_integer(
                    track,
                    identities[&record.node.object_id],
                    *b"KRTX",
                    start,
                    globals,
                    context,
                )?);
            }
        }
        for (material_index, material) in source.materials().enumerate() {
            for (layer_index, layer) in material.layers.iter().enumerate() {
                if material_index > 0x7fff || layer_index > 0xffff {
                    return Err(Error::new(
                        "materials",
                        "material/layer identity exceeds native property namespace",
                    ));
                }
                let owner = 0x8000_0000 | ((material_index as u32) << 16) | layer_index as u32;
                if let Some(track) = layer.texture_id.track() {
                    clip.integer_tracks.push(property_integer(
                        track, owner, *b"KMTF", start, globals, context,
                    )?);
                }
                scalar!(owner, b"KMTA", layer.alpha);
                if let Ok(emissive) = layer.try_emissive_gain() {
                    scalar!(owner, b"KMTE", emissive);
                }
                if let Ok(fresnel) = layer.try_fresnel() {
                    color!(owner, b"KFC3", fresnel.color);
                    scalar!(owner, b"KFCA", fresnel.opacity);
                    scalar!(owner, b"KFTC", fresnel.team_color);
                }
            }
        }
    }
    Ok(())
}

pub(in crate::model::conversion::mdlx) fn property_integer(
    track: &Track<'_, u32>,
    owner: u32,
    tag: [u8; 4],
    start: f32,
    globals: &[u32],
    _context: &mut Context,
) -> Result<s::IntegerTrack, Error> {
    let data = read_track(track);
    Ok(s::IntegerTrack::Unsigned(s::Track {
        layer: Default::default(),
        target: s::Target::Property {
            owner,
            channel: u32::from_le_bytes(tag),
        },
        interpolation: match data.interpolation {
            Interpolation::Step => s::Interpolation::Step,
            Interpolation::Linear => s::Interpolation::Linear,
            Interpolation::Hermite => s::Interpolation::Hermite,
            Interpolation::Bezier => s::Interpolation::Bezier,
        },
        global_duration: data.global.and_then(|index| {
            globals
                .get(index as usize)
                .map(|duration| *duration as f32 / 1000.)
        }),
        keys: data
            .keys
            .into_iter()
            .map(|(time, value, tangents)| s::Key {
                time: time as f32 / 1000. - if data.global.is_some() { 0. } else { start },
                value: vec![value],
                incoming: tangents.map(|(value, _)| vec![value]),
                outgoing: tangents.map(|(_, value)| vec![value]),
            })
            .collect(),
    }))
}
