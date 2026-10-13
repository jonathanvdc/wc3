//! Animated extents represented by minimum, maximum, and radius channels.
use super::super::effects::metadata as native_metadata;
use super::*;
pub(super) fn import<E: Envelope>(
    input: ImportBlock<'_, '_, E>,
    target: BoundsTarget<'_>,
) -> Result<(), Error> {
    let ImportBlock {
        source,
        controller,
        selector,
        id,
        start,
        clip: clip_index,
    } = input;
    let BoundsTarget {
        raw,
        model,
        channels,
        tracks,
        fresh,
    } = target;
    let block = source
        .resolve::<nr::AnimationBlockExtent>(controller.sdmb.cast())
        .map_err(read_error)?
        .get((selector & 0xffff) as usize)
        .ok_or_else(|| Error::new("m3.bounds", "missing extent block"))?
        .map_err(read_error)?;
    let times: Vec<_> = source
        .resolve::<nr::I32>(block.timestamps.cast())
        .map_err(read_error)?
        .iter()
        .map(|row| {
            row.map(|row| wr::Real {
                value: (row.value as f64 - f64::from(start)) as f32 / 1000.,
            })
        })
        .collect::<Result<_, _>>()
        .map_err(read_error)?;
    let extents: Vec<_> = source
        .resolve::<nr::ExtentRecord>(block.keys.cast())
        .map_err(read_error)?
        .iter()
        .map(|row| row.map(|row| row.value))
        .collect::<Result<_, _>>()
        .map_err(read_error)?;
    if times.len() != extents.len() {
        return Err(Error::new("m3.bounds", "extent/time count differs"));
    }
    let mut nodes: Vec<_> = raw
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let mut existing = None;
    for (index, node) in nodes.iter().enumerate() {
        let data = native_metadata::read(raw, &node.native)?;
        if data.get("m3.bounds.source_id") == Some(&i64::from(id)) {
            existing = Some((index, data));
            break;
        }
    }
    let (owner, ids) = if let Some((owner, data)) = existing {
        (
            owner,
            [
                id,
                data["m3.bounds.maximum_id"] as u32,
                data["m3.bounds.radius_id"] as u32,
            ],
        )
    } else {
        let a = *fresh;
        *fresh = fresh
            .checked_add(2)
            .ok_or_else(|| Error::new("m3.bounds.id", "channel identity overflow"))?;
        let ids = [id, a, a + 1];
        let fields = [
            ("m3.effect.bone", 0),
            ("m3.auxiliary", 1),
            ("m3.bounds.source_id", i64::from(id)),
            ("m3.bounds.maximum_id", i64::from(a)),
            ("m3.bounds.radius_id", i64::from(a + 1)),
        ];
        let mut entries = Vec::new();
        for (name, value) in fields {
            entries.push(wr::NativeEntry::<wr::V2> {
                name: text(raw, name)?,
                value,
                ..Default::default()
            });
        }
        let owner = nodes.len();
        nodes.push(wr::Node::<wr::V19> {
            name: text(raw, &format!("animated-bounds-{id}"))?,
            parent: if owner == 0 { u32::MAX } else { 0 },
            kind: wr::NodeKind::HELPER,
            native: wr::NativeBag {
                entries: put(raw, &entries)?.cast(),
            },
            ..Default::default()
        });
        (owner, ids)
    };
    let mut entries: Vec<_> = raw
        .resolve::<wr::NativeEntry<wr::V2>>(nodes[owner].native.entries.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    for (name, value) in [
        ("flags", i64::from(block.flags)),
        ("end", i64::from(block.end_frame)),
    ] {
        entries.push(wr::NativeEntry::<wr::V2> {
            name: text(raw, &format!("m3.bounds.clip.{clip_index}.{name}"))?,
            value,
            ..Default::default()
        });
    }
    nodes[owner].native.entries = put(raw, &entries)?.cast();
    model.nodes.nodes = put(raw, &nodes)?.cast();
    for (part, &id) in ids.iter().enumerate() {
        let values: Vec<_> = extents
            .iter()
            .flat_map(|extent| match part {
                0 => extent.minimum.to_vec(),
                1 => extent.maximum.to_vec(),
                _ => vec![extent.radius],
            })
            .flat_map(f32::to_le_bytes)
            .collect();
        let kind = if part == 2 {
            wr::AttrType::F32
        } else {
            wr::AttrType::F32X3
        };
        let channel = wr::AnimChannel {
            id,
            target: wr::TrackTarget {
                kind: wr::TrackTargetKind::NODE,
                node: owner as u32,
                sub: id,
                channel: wr::Channel(255),
                ..Default::default()
            },
            value_type: kind,
            init_value: bytes(
                raw,
                &values[..values.len().checked_div(times.len().max(1)).unwrap_or(0)],
            )?,
        };
        if let Some(existing) = channels.iter().find(|channel| channel.id == id) {
            if existing.target != channel.target || existing.value_type != kind {
                return Err(Error::new(
                    "m3.bounds.id",
                    "bound identity conflicts with existing channel",
                ));
            }
        } else {
            channels.push(channel);
        }
        tracks.push(wr::SubTrack::<wr::V2> {
            channel: id,
            interp: wr::Interpolation::LINEAR,
            times: put(raw, &times)?.cast(),
            values: bytes(raw, &values)?,
            ..Default::default()
        });
    }
    Ok(())
}

pub(super) fn export<E: Envelope>(
    model: &sem::Model<'_>,
    clip_index: usize,
    clip: &wr::Clip<wr::V7>,
    node: &wr::Node<wr::V19>,
    doc: &mut native::Document<'static, E>,
    context: &mut Context,
) -> Result<nr::AnimationBlockExtent, Error> {
    let raw = model
        .native
        .as_ref()
        .ok_or_else(|| Error::new("wem.bounds", "missing native graph"))?
        .document
        .raw();
    let data = native_metadata::read(raw, &node.native)?;
    let ids = [
        data["m3.bounds.source_id"] as u32,
        data["m3.bounds.maximum_id"] as u32,
        data["m3.bounds.radius_id"] as u32,
    ];
    let mut all = Vec::new();
    for container in raw
        .resolve::<wr::SubTrackContainer>(clip.containers.cast())?
        .iter()
    {
        for track in raw
            .resolve::<wr::SubTrack<wr::V2>>(container?.sub_tracks.cast())?
            .iter()
        {
            all.push(track?);
        }
    }
    let mut parts = Vec::new();
    let mut times = None;
    for (part, id) in ids.into_iter().enumerate() {
        let track = all
            .iter()
            .find(|track| track.channel == id)
            .ok_or_else(|| Error::new("wem.bounds", "incomplete extent component channels"))?;
        if track.interp != wr::Interpolation::LINEAR {
            context.unsupported(
                "wem.bounds.interpolation",
                "extent components require the native linear convention",
            )?;
        }
        let current: Vec<_> = raw
            .resolve::<wr::Real>(track.times.cast())?
            .iter()
            .collect::<Result<_, _>>()?;
        if times.as_ref().is_some_and(|times| times != &current) {
            return Err(Error::new(
                "wem.bounds.times",
                "extent components use different key times",
            ));
        }
        times = Some(current);
        let bytes: Vec<_> = raw
            .resolve::<wr::Byte>(track.values.cast())?
            .iter()
            .map(|row| row.map(|row| row.value))
            .collect::<Result<_, _>>()?;
        let width = if part == 2 { 1 } else { 3 };
        if bytes.len() != times.as_ref().unwrap().len() * width * 4 {
            return Err(Error::new(
                "wem.bounds.values",
                "extent component width differs",
            ));
        }
        parts.push(
            bytes
                .as_chunks::<4>()
                .0
                .iter()
                .map(|bytes| f32::from_le_bytes(*bytes))
                .collect::<Vec<_>>(),
        );
    }
    let origin = clock::start(raw, clip)?;
    let times = times
        .unwrap()
        .iter()
        .map(|time| clock::frame(time.value, origin, context, "bounds.time"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut extents = Vec::new();
    for index in 0..times.len() {
        extents.push(nr::ExtentRecord {
            value: nr::Extent {
                minimum: parts[0][index * 3..index * 3 + 3].try_into().unwrap(),
                maximum: parts[1][index * 3..index * 3 + 3].try_into().unwrap(),
                radius: parts[2][index],
            },
        });
    }
    Ok(nr::AnimationBlockExtent {
        timestamps: append(doc, &times)?.cast(),
        keys: append(doc, &extents)?.cast(),
        flags: data
            .get(&format!("m3.bounds.clip.{clip_index}.flags"))
            .copied()
            .unwrap_or(0) as u32,
        end_frame: data
            .get(&format!("m3.bounds.clip.{clip_index}.end"))
            .copied()
            .unwrap_or((clip.duration * 1000.).round() as i64) as u32,
    })
}
