//! Native light, attachment, collision, and liquid authoring values.
use super::*;
#[derive(Clone, Debug)]
pub(super) enum NodeSource {
    Light(nr::Light),
    Doodad(nr::DoodadDef),
    Point { group: usize, light: nr::PointLight },
}
pub(super) fn import<D: Dialect>(
    source: &Bundle<'_, D>,
    model: &mut s::Model<'static>,
) -> Result<Vec<NodeSource>, Error> {
    let mut nodes = Vec::new();
    let doodad_names = source.root.chunks().find_map(|chunk| {
        if let RootChunk::DoodadNames(rows) = chunk {
            Some(rows.as_bytes())
        } else {
            None
        }
    });
    let doodad_ids = source
        .root
        .chunks()
        .find_map(|chunk| {
            if let RootChunk::DoodadFileIds(rows) = chunk {
                Some(collect(rows, "doodad.ids"))
            } else {
                None
            }
        })
        .transpose()?
        .unwrap_or_default();
    for chunk in source.root.chunks() {
        match chunk {
            RootChunk::Lights(rows) => {
                for light in collect(rows, "lights")? {
                    model.nodes.push(s::Node {
                        name: Cow::Owned(format!("Light {}", nodes.len())),
                        translation: light.position,
                        rotation: light.rotation,
                        ..Default::default()
                    });
                    nodes.push(NodeSource::Light(light));
                }
            }
            RootChunk::Doodads(rows) => {
                for doodad in collect(rows, "doodads")? {
                    let identity = doodad.name_index();
                    let name = if let Some(names) = doodad_names {
                        text(names, identity, "doodad.name")?
                    } else {
                        format!(
                            "FileDataID {}",
                            doodad_ids.get(identity as usize).copied().ok_or_else(
                                || Error::new("doodad.id", "missing doodad resource identity")
                            )?
                        )
                    };
                    model.nodes.push(s::Node {
                        name: Cow::Owned(name),
                        translation: doodad.position,
                        rotation: doodad.rotation,
                        scale: [doodad.scale; 3],
                        ..Default::default()
                    });
                    nodes.push(NodeSource::Doodad(doodad));
                }
            }
            _ => {}
        }
    }
    for group_index in 0..source.root.header().group_count as usize {
        let group = source
            .group(GroupKey {
                index: group_index,
                lod: 0,
            })
            .ok_or_else(|| Error::new("group", "missing group"))?;
        for chunk in group.chunks() {
            if let GroupChunk::PointLights(rows) = chunk {
                for light in collect(rows, "group.point_lights")? {
                    model.nodes.push(s::Node {
                        name: Cow::Owned(format!("Group {group_index} point light")),
                        translation: light.position,
                        ..Default::default()
                    });
                    nodes.push(NodeSource::Point {
                        group: group_index,
                        light,
                    });
                }
            }
        }
    }
    Ok(nodes)
}
pub(super) fn decorate<D: Dialect>(
    doc: &mut RawDocument<'static>,
    model: &mut wr::ModelRecord<wr::V9>,
    source: &Bundle<'_, D>,
    nodes: &[NodeSource],
) -> Result<(), Error> {
    let mut rows = doc
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let doodad_names = source.root.chunks().find_map(|chunk| {
        if let RootChunk::DoodadNames(rows) = chunk {
            Some(rows.as_bytes())
        } else {
            None
        }
    });
    let doodad_ids = source
        .root
        .chunks()
        .find_map(|chunk| {
            if let RootChunk::DoodadFileIds(rows) = chunk {
                Some(collect(rows, "doodad.ids"))
            } else {
                None
            }
        })
        .transpose()?
        .unwrap_or_default();
    for (index, (node, source)) in rows.iter_mut().zip(nodes).enumerate() {
        match source {
            NodeSource::Light(light) => {
                node.kind = wr::NodeKind::Light;
                node.payload = wr::NodePayload::Light(Box::new(wr::LightPayload::<wr::V19> {
                    kind: wr::LightKind::from_raw(match light.kind {
                        nr::LightKind::Omni => 0,
                        nr::LightKind::Spot => 2,
                        nr::LightKind::Directional => 1,
                        nr::LightKind::Ambient => 3,
                        nr::LightKind::Unknown(value) => value,
                    }),
                    color: [light.color[2], light.color[1], light.color[0]]
                        .map(|v| f32::from(v) / 255.),
                    intensity: light.intensity,
                    attenuation_start: light.attenuation_start,
                    attenuation_end: light.attenuation_end,
                    ..Default::default()
                }));
                node.native = metadata::bag(
                    doc,
                    &[
                        (
                            "wmo.light.kind".into(),
                            i64::from(light.kind.raw()),
                            String::new(),
                        ),
                        (
                            "wmo.light.attenuation".into(),
                            i64::from(light.use_attenuation),
                            String::new(),
                        ),
                        (
                            "wmo.light.color".into(),
                            i64::from(u32::from_le_bytes(light.color)),
                            String::new(),
                        ),
                        (
                            "wmo.light.padding".into(),
                            i64::from(u16::from_le_bytes(light.padding)),
                            String::new(),
                        ),
                    ],
                )?;
            }
            NodeSource::Doodad(doodad) => {
                let identity = doodad.name_index();
                let asset = if let Some(names) = doodad_names {
                    wr::AssetKey {
                        group: 0,
                        id: u32::MAX,
                        path: metadata::string(doc, &text(names, identity, "doodad.path")?)?,
                    }
                } else {
                    wr::AssetKey {
                        group: 0,
                        id: *doodad_ids
                            .get(identity as usize)
                            .ok_or_else(|| Error::new("doodad.id", "missing file identity"))?,
                        path: Ref::default(),
                    }
                };
                node.kind = wr::NodeKind::Attachment;
                node.payload = wr::NodePayload::Attachment(Box::new(wr::AttachmentPayload {
                    asset,
                    model: u32::MAX,
                }));
                node.native = metadata::bag(
                    doc,
                    &[
                        (
                            "wmo.doodad.flags".into(),
                            i64::from(doodad.flags()),
                            String::new(),
                        ),
                        (
                            "wmo.doodad.color".into(),
                            i64::from(u32::from_le_bytes(doodad.color)),
                            String::new(),
                        ),
                        ("wmo.doodad.index".into(), index as i64, String::new()),
                    ],
                )?;
            }
            NodeSource::Point { group, light } => {
                node.kind = wr::NodeKind::Light;
                node.payload = wr::NodePayload::Light(Box::new(wr::LightPayload::<wr::V19> {
                    kind: wr::LightKind::Omni,
                    color: [
                        light.color_bgra[2],
                        light.color_bgra[1],
                        light.color_bgra[0],
                    ]
                    .map(|v| f32::from(v) / 255.),
                    intensity: light.intensity,
                    attenuation_start: light.attenuation_start,
                    attenuation_end: light.attenuation_end,
                    ..Default::default()
                }));
                node.native = metadata::bag(
                    doc,
                    &[
                        ("wmo.point.group".into(), *group as i64, String::new()),
                        (
                            "wmo.point.color".into(),
                            i64::from(u32::from_le_bytes(light.color_bgra)),
                            String::new(),
                        ),
                        (
                            "wmo.point.reserved_header".into(),
                            i64::from(light.reserved_header),
                            String::new(),
                        ),
                        (
                            "wmo.point.reserved_float".into(),
                            i64::from(light.reserved_float.to_bits()),
                            String::new(),
                        ),
                        (
                            "wmo.point.reserved_tail".into(),
                            0,
                            metadata::words(light.reserved_tail),
                        ),
                    ],
                )?;
            }
        }
    }
    if !model.nodes.nodes.is_empty() {
        doc.replace(doc.section_id(model.nodes.nodes.index).unwrap(), &rows)?;
    }
    Ok(())
}
pub(super) struct Output {
    pub(super) lights: Vec<nr::Light>,
    pub(super) doodads: Vec<nr::DoodadDef>,
    pub(super) names: Option<Vec<u8>>,
    pub(super) ids: Option<Vec<u32>>,
    pub(super) points: BTreeMap<usize, Vec<nr::PointLight>>,
}
fn color(value: [f32; 3], alpha: u8, context: &mut Context, path: &str) -> Result<[u8; 4], Error> {
    let mut output = [0, 0, 0, alpha];
    for (index, value) in value.into_iter().enumerate() {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(Error::new(
                path,
                "light color component outside normalized range",
            ));
        }
        let byte = (value * 255.).round() as u8;
        context.approximate(
            format!("{path}.color[{index}]"),
            (value - f32::from(byte) / 255.).abs(),
            "Light color quantized to one byte",
        )?;
        output[2 - index] = byte;
    }
    Ok(output)
}
pub(super) fn export(
    raw: &RawDocument<'_>,
    nodes: &[wr::Node<wr::V19>],
    context: &mut Context,
) -> Result<Output, Error> {
    let mut output = Output {
        lights: Vec::new(),
        doodads: Vec::new(),
        names: None,
        ids: None,
        points: BTreeMap::new(),
    };
    let uses_paths=nodes.iter().any(|node|matches!(&node.payload,wr::NodePayload::Attachment(data)if !data.asset.path.is_empty()));
    let mut names = vec![0];
    let mut ids = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        let path = format!("nodes[{index}]");
        if node.parent != u32::MAX {
            context.unsupported(
                format!("{path}.parent"),
                "WMO environment nodes are placed directly in model space",
            )?;
        }
        let fields = metadata::read(raw, node.native.clone())?;
        match &node.payload {
            wr::NodePayload::Helper(_) => {}
            wr::NodePayload::Bone(_) => {
                context.unsupported(path, "WMO cannot represent skeletal bones")?
            }
            wr::NodePayload::Light(data) => {
                let encoded = metadata::number(
                    &fields,
                    "wmo.light.color",
                    metadata::number(&fields, "wmo.point.color", u32::MAX)?,
                )?
                .to_le_bytes();
                let color = color(data.color, encoded[3], context, &path)?;
                if let Some((group, _)) = fields.get("wmo.point.group") {
                    let group = usize::try_from(*group).map_err(|error| failure(&path, error))?;
                    let tail = fields
                        .get("wmo.point.reserved_tail")
                        .map(|(_, text)| metadata::parse_words(text))
                        .transpose()?
                        .unwrap_or_else(|| vec![0; 2]);
                    output
                        .points
                        .entry(group)
                        .or_default()
                        .push(nr::PointLight {
                            reserved_header: metadata::number(
                                &fields,
                                "wmo.point.reserved_header",
                                0,
                            )?,
                            color_bgra: color,
                            position: node.local.translation,
                            intensity: data.intensity,
                            attenuation_start: data.attenuation_start,
                            attenuation_end: data.attenuation_end,
                            reserved_float: f32::from_bits(metadata::number(
                                &fields,
                                "wmo.point.reserved_float",
                                0,
                            )?),
                            reserved_tail: tail.as_slice().try_into().map_err(|_| {
                                Error::new(&path, "point-light tail needs two words")
                            })?,
                        });
                } else {
                    let kind = match data.kind {
                        wr::LightKind::Omni => nr::LightKind::Omni,
                        wr::LightKind::Directional => nr::LightKind::Directional,
                        wr::LightKind::Spot => nr::LightKind::Spot,
                        wr::LightKind::Ambient => nr::LightKind::Ambient,
                        value => nr::LightKind::from_raw(value.raw()),
                    };
                    output.lights.push(nr::Light {
                        kind,
                        use_attenuation: u8::try_from(metadata::number(
                            &fields,
                            "wmo.light.attenuation",
                            1,
                        )?)
                        .map_err(|error| failure(&path, error))?,
                        padding: u16::try_from(metadata::number(&fields, "wmo.light.padding", 0)?)
                            .map_err(|error| failure(&path, error))?
                            .to_le_bytes(),
                        color,
                        position: node.local.translation,
                        intensity: data.intensity,
                        rotation: node.local.rotation,
                        attenuation_start: data.attenuation_start,
                        attenuation_end: data.attenuation_end,
                    });
                }
            }
            wr::NodePayload::Attachment(data) => {
                if data.model != u32::MAX {
                    context.unsupported(
                        format!("{path}.model"),
                        "WMO doodads must reference an external model",
                    )?;
                }
                if node.local.scale[0] != node.local.scale[1]
                    || node.local.scale[0] != node.local.scale[2]
                {
                    context.unsupported(
                        format!("{path}.scale"),
                        "WMO doodads store one uniform scale component",
                    )?;
                }
                let identity = if uses_paths {
                    let value = raw.string(data.asset.path)?.unwrap_or_default();
                    if value.is_empty() && data.asset.id != u32::MAX {
                        return Err(Error::new(
                            &path,
                            "doodad filename and FileDataID addressing require a resource mapping",
                        ));
                    }
                    let offset =
                        u32::try_from(names.len()).map_err(|error| failure(&path, error))?;
                    names.extend_from_slice(value.as_bytes());
                    names.push(0);
                    offset
                } else {
                    if data.asset.id == u32::MAX {
                        return Err(Error::new(
                            &path,
                            "doodad resource has neither filename nor FileDataID",
                        ));
                    }
                    let index = ids
                        .iter()
                        .position(|id| *id == data.asset.id)
                        .unwrap_or_else(|| {
                            ids.push(data.asset.id);
                            ids.len() - 1
                        });
                    index as u32
                };
                if identity > 0xffffff {
                    return Err(Error::new(
                        &path,
                        "doodad resource identity exceeds 24 bits",
                    ));
                }
                let flags = metadata::number(&fields, "wmo.doodad.flags", 0)?;
                if flags > 255 {
                    return Err(Error::new(&path, "doodad flags exceed one byte"));
                }
                output.doodads.push(nr::DoodadDef {
                    name_and_flags: nr::DoodadReference(identity | (flags << 24)),
                    position: node.local.translation,
                    rotation: node.local.rotation,
                    scale: node.local.scale[0],
                    color: metadata::number(&fields, "wmo.doodad.color", u32::MAX)?.to_le_bytes(),
                });
            }
            _ => context.unsupported(
                format!("{path}.payload"),
                "Native node payload has no WMO representation",
            )?,
        }
    }
    if !output.doodads.is_empty() {
        if uses_paths {
            output.names = Some(names);
        } else {
            output.ids = Some(ids);
        }
    }
    Ok(output)
}
