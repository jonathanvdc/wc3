//! Export mappings for animation.
use super::*;
pub(in crate::model::conversion::mdlx) fn scalar_from_array(
    data: TrackData<[f32; 1]>,
) -> Result<Track<'static, f32>, Error> {
    write_track(&TrackData {
        interpolation: data.interpolation,
        global: data.global,
        keys: data
            .keys
            .into_iter()
            .map(|(time, value, tangents)| {
                (
                    time,
                    value[0],
                    tangents.map(|(incoming, outgoing)| (incoming[0], outgoing[0])),
                )
            })
            .collect(),
    })
    .map_err(|error| failure("property", error))
}

pub(in crate::model::conversion::mdlx) fn overlay_properties<D: ModelDialect>(
    model: &mut native::Model<'static, D>,
    semantic: &s::Model<'_>,
    active_nodes: &[wr::Node<wr::V19>],
    ids: &[u32],
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
) -> Result<(), Error> {
    let mut scalar_tracks: BTreeMap<(u32, u32), Option<Track<'static, f32>>> = BTreeMap::new();
    let mut color_tracks: BTreeMap<(u32, u32), Option<Track<'static, [f32; 3]>>> = BTreeMap::new();
    let mut quaternion_tracks: BTreeMap<(u32, u32), Option<Track<'static, [f32; 4]>>> =
        BTreeMap::new();
    for (clip_index, clip) in semantic.clips.iter().enumerate() {
        for (track_index, track) in clip.tracks.iter().enumerate() {
            let s::Target::Property { owner, channel } = track.target else {
                continue;
            };
            let path = format!("clips[{clip_index}].tracks[{track_index}]");
            let width = track.keys.first().map(|key| key.value.len()).unwrap_or(1);
            match width {
                1 => {
                    let value = scalar_from_array(native_track_data::<1>(
                        track,
                        starts[clip_index] as f32 / 1000.,
                        globals,
                        context,
                        &path,
                    )?)?;
                    merge_track(
                        scalar_tracks.entry((owner, channel)).or_default(),
                        value,
                        &path,
                    )?;
                }
                3 => {
                    let value = native_track::<3>(
                        track,
                        starts[clip_index] as f32 / 1000.,
                        globals,
                        context,
                        &path,
                    )?;
                    merge_track(
                        color_tracks.entry((owner, channel)).or_default(),
                        value,
                        &path,
                    )?;
                }
                _ => context
                    .unsupported(path, "MDLX property channel has unsupported value width")?,
            }
        }
    }
    let mut scalar = |owner: u32, tag: [u8; 4]| {
        scalar_tracks
            .remove(&(owner, u32::from_le_bytes(tag)))
            .flatten()
    };
    let mut color = |owner: u32, tag: [u8; 4]| {
        color_tracks
            .remove(&(owner, u32::from_le_bytes(tag)))
            .flatten()
    };
    let identities: BTreeMap<_, _> = ids
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index as u32))
        .collect();
    macro_rules! scalar {
        ($owner:expr,$tag:expr,$field:expr) => {
            if let Some(track) = scalar($owner, *$tag) {
                $field.set_track(track);
            }
        };
    }
    macro_rules! option {
        ($owner:expr,$tag:expr,$field:expr) => {
            if let Some(track) = scalar($owner, *$tag) {
                $field = Some(track);
            }
        };
    }
    macro_rules! color {
        ($owner:expr,$tag:expr,$field:expr) => {
            if let Some(track) = color($owner, *$tag) {
                $field.set_track(track);
            }
        };
    }
    for chunk in &mut model.chunks {
        match chunk {
            ModelChunk::ParticleEmitters(chunk) => {
                for record in &mut chunk.records {
                    let owner = identities[&record.node.object_id];
                    scalar!(owner, b"KPEE", record.emission_rate);
                    scalar!(owner, b"KPEG", record.gravity);
                    scalar!(owner, b"KPLN", record.longitude);
                    scalar!(owner, b"KPLT", record.latitude);
                    scalar!(owner, b"KPEL", record.life_span);
                    scalar!(owner, b"KPES", record.initial_velocity);
                    option!(owner, b"KPEV", record.visibility);
                }
            }
            ModelChunk::ParticleEmitters2(chunk) => {
                for record in &mut chunk.records {
                    let owner = identities[&record.node.object_id];
                    scalar!(owner, b"KP2S", record.speed);
                    scalar!(owner, b"KP2R", record.variation);
                    scalar!(owner, b"KP2L", record.latitude);
                    scalar!(owner, b"KP2G", record.gravity);
                    scalar!(owner, b"KP2E", record.emission_rate);
                    scalar!(owner, b"KP2W", record.width);
                    scalar!(owner, b"KP2N", record.length);
                    option!(owner, b"KP2V", record.visibility);
                }
            }
            ModelChunk::RibbonEmitters(chunk) => {
                for record in &mut chunk.records {
                    let owner = identities[&record.node.object_id];
                    if let Some(track) = scalar(owner, *b"KRTX") {
                        record.texture_slot.set_track(integer_from_scalar(
                            track,
                            context,
                            "ribbon.texture_slot",
                        )?);
                    }
                    scalar!(owner, b"KRHA", record.height_above);
                    scalar!(owner, b"KRHB", record.height_below);
                    scalar!(owner, b"KRAL", record.alpha);
                    color!(owner, b"KRCO", record.color);
                    option!(owner, b"KRVS", record.visibility);
                }
            }
            ModelChunk::PopcornEmitters(chunk) => {
                for record in &mut chunk.records {
                    let owner = identities[&record.node.object_id];
                    scalar!(owner, b"KPPL", record.life_span);
                    scalar!(owner, b"KPPE", record.emission_rate);
                    scalar!(owner, b"KPPS", record.speed);
                    scalar!(owner, b"KPPA", record.alpha);
                    color!(owner, b"KPPC", record.color);
                    option!(owner, b"KPPV", record.visibility);
                }
            }
            ModelChunk::Attachments(chunk) => {
                for record in &mut chunk.records {
                    option!(
                        identities[&record.node.object_id],
                        b"KATV",
                        record.visibility
                    );
                }
            }
            ModelChunk::Lights(chunk) => {
                for record in &mut chunk.records {
                    let owner = identities[&record.node.object_id];
                    scalar!(owner, b"KLAS", record.attenuation_start);
                    scalar!(owner, b"KLAE", record.attenuation_end);
                    color!(owner, b"KLAC", record.color);
                    scalar!(owner, b"KLAI", record.intensity);
                    color!(owner, b"KLBC", record.ambient_color);
                    scalar!(owner, b"KLBI", record.ambient_intensity);
                    option!(owner, b"KLAV", record.visibility);
                }
            }
            _ => {}
        }
    }
    for (material_index, material) in model.materials_mut().enumerate() {
        for (layer_index, layer) in material.layers.iter_mut().enumerate() {
            let owner = 0x8000_0000 | ((material_index as u32) << 16) | layer_index as u32;
            if let Some(track) = scalar(owner, *b"KMTF") {
                layer.texture_id.set_track(integer_from_scalar(
                    track,
                    context,
                    "material.texture_id",
                )?);
            }
            scalar!(owner, b"KMTA", layer.alpha);
            if let Some(track) = scalar(owner, *b"KMTE") {
                let mut value = layer.try_emissive_gain().cloned().unwrap_or_default();
                value.set_track(track);
                layer
                    .try_set_emissive_gain(value)
                    .map_err(|error| failure("material.emissive", error))?;
            }
            let mut fresnel = layer.try_fresnel().cloned().unwrap_or_default();
            color!(owner, b"KFC3", fresnel.color);
            scalar!(owner, b"KFCA", fresnel.opacity);
            scalar!(owner, b"KFTC", fresnel.team_color);
            let animated = fresnel.color.track().is_some()
                || fresnel.opacity.track().is_some()
                || fresnel.team_color.track().is_some();
            if let Err(error) = layer.try_set_fresnel(fresnel) {
                if animated {
                    context.unsupported("material.fresnel", error.to_string())?;
                }
            }
        }
    }
    let mut camera_index = 0usize;
    for (owner, node) in active_nodes.iter().enumerate() {
        if !matches!(node.payload, wr::NodePayload::Camera(_)) {
            continue;
        }
        if let Some(camera) = model.cameras_mut().nth(camera_index) {
            let owner = owner as u32;
            option!(owner, b"KCRL", camera.rotation);
            option!(owner, b"KCVS", camera.visibility);
            option!(owner, b"KCFD", camera.focus_distance);
            option!(owner, b"KCFL", camera.focal_length);
            option!(owner, b"KCFS", camera.f_stop);
            if let Some(track) = color(owner, *b"KTTR") {
                camera.target_translation = Some(track);
            }
        }
        camera_index += 1;
    }
    let count = semantic
        .clips
        .iter()
        .flat_map(|clip| &clip.tracks)
        .filter_map(|track| match track.target {
            s::Target::Property { owner, .. } if owner & 0xe000_0000 == 0x2000_0000 => {
                Some((owner & 0x1fff_ffff) as usize + 1)
            }
            _ => None,
        })
        .max()
        .unwrap_or(0);
    let mut textures: Vec<_> = model.texture_animations().cloned().collect();
    textures.resize_with(textures.len().max(count), native::TextureAnimation::new);
    for (index, animation) in textures.iter_mut().enumerate() {
        let owner = 0x2000_0000 | index as u32;
        if let Some(track) = color(owner, *b"KTAT") {
            animation.translation = Some(track);
        }
        if let Some(track) = color(owner, *b"KTAS") {
            animation.scaling = Some(track);
        }
        if let Some(track) = quaternion_tracks
            .remove(&(owner, u32::from_le_bytes(*b"KTAR")))
            .flatten()
        {
            animation.rotation = Some(track);
        }
    }
    model.set_texture_animations(textures);
    let mut animations: Vec<_> = model.geoset_animations().cloned().collect();
    for mesh in 0..semantic.meshes.len() {
        let owner = 0x4000_0000 | mesh as u32;
        let alpha = scalar(owner, *b"KGAO");
        let value = color(owner, *b"KGAC");
        if alpha.is_some() || value.is_some() {
            let record = if let Some(index) = animations
                .iter()
                .position(|record| record.geoset_id == mesh as u32)
            {
                &mut animations[index]
            } else {
                animations.push(native::GeosetAnimation::new(mesh as u32));
                animations.last_mut().unwrap()
            };
            if let Some(track) = alpha {
                record.alpha.set_track(track);
            }
            if let Some(track) = value {
                record.color.set_track(track);
                record.flags.set_color(true);
            }
        }
    }
    model.set_geoset_animations(animations);
    drop(scalar);
    drop(color);
    for ((owner, channel), _) in quaternion_tracks {
        context.unsupported(
            format!("properties[{owner}].{channel}"),
            "Quaternion property channel has no matching owner",
        )?;
    }
    for ((owner, channel), _) in scalar_tracks
        .into_iter()
        .chain(color_tracks.into_keys().map(|key| (key, None)))
    {
        context.unsupported(
            format!(
                "properties[{owner}].{}",
                String::from_utf8_lossy(&channel.to_le_bytes())
            ),
            "Native property channel has no matching exported owner",
        )?;
    }
    Ok(())
}

pub(in crate::model::conversion::mdlx) fn integer_from_scalar(
    track: Track<'static, f32>,
    context: &mut Context,
    path: &str,
) -> Result<Track<'static, u32>, Error> {
    let data = read_track(&track);
    let mut keys = Vec::new();
    for (time, value, tangents) in data.keys {
        let mut convert = |value: f32| -> Result<u32, Error> {
            if !value.is_finite() || value < 0. || f64::from(value) > f64::from(u32::MAX) {
                return Err(Error::new(path, "integer channel value outside u32 range"));
            }
            let integer = value.round() as u32;
            context.approximate(
                path,
                (value - integer as f32).abs(),
                "Integer channel value rounded",
            )?;
            Ok(integer)
        };
        keys.push((
            time,
            convert(value)?,
            tangents
                .map(|(incoming, outgoing)| {
                    Ok::<_, Error>((convert(incoming)?, convert(outgoing)?))
                })
                .transpose()?,
        ));
    }
    write_track(&TrackData {
        interpolation: data.interpolation,
        global: data.global,
        keys,
    })
    .map_err(|error| failure(path, error))
}
