use super::projection::read_mesh;
use super::*;
pub(super) fn read_impl<'a>(
    document: &'a Document<'_, V3>,
    index: u32,
    profile: r::ProfileId,
    look: Option<u32>,
) -> Result<Model<'a>, ReadError> {
    let root = document.root::<r::DocumentRecord<r::V1>>()?;
    if !root.profiles.0.contains(&profile) {
        return Err(ReadError::invalid(0, "undeclared selected profile"));
    }
    let doc = document.raw();
    let section = doc
        .referenced_section(root.models)?
        .ok_or_else(|| ReadError::invalid(0, "missing models"))?;
    if index >= root.models.entries {
        return Err(ReadError::invalid(0, "model selection"));
    }
    macro_rules! model_header {
        ($v:ty) => {{
            let record = doc
                .resolve::<r::ModelRecord<$v>>(root.models.cast())?
                .get(index as usize)
                .ok_or_else(|| ReadError::invalid(0, "model selection"))??;
            (
                record.name,
                record.bounds,
                record.meshes,
                record.nodes.nodes,
                record.profile_sets,
                record.anim_channels.channels,
            )
        }};
    }
    let (name, bounds, meshes, nodes, profiles, channels) = match section.revision() {
        1 => model_header!(r::V1),
        2 => model_header!(r::V2),
        3 => model_header!(r::V3),
        4 => model_header!(r::V4),
        5 => model_header!(r::V5),
        6 => model_header!(r::V6),
        7 => model_header!(r::V7),
        8 => model_header!(r::V8),
        9 => model_header!(r::V9),
        _ => return Err(ReadError::unsupported(0, "model revision")),
    };
    let mut model = Model {
        name: text_view(doc, name)?,
        space: root.space,
        unit_scale: root.unit_scale,
        bounds,
        profile,
        ..Default::default()
    };
    let mut mesh_projections = Vec::new();
    if meshes.entries > 0 {
        let section = doc
            .referenced_section(meshes)?
            .ok_or_else(|| ReadError::invalid(0, "mesh table"))?;
        match section.revision() {
            2 => {
                for (index, mesh) in rows::<r::Mesh<r::V2>>(doc, meshes.cast())?
                    .into_iter()
                    .enumerate()
                {
                    let (values, projections) = read_mesh(doc, mesh, index as u32)?;
                    model.meshes.extend(values);
                    mesh_projections.extend(projections);
                }
            }
            1 => {
                for (index, mesh) in rows::<r::Mesh<r::V1>>(doc, meshes.cast())?
                    .into_iter()
                    .enumerate()
                {
                    let (values, projections) = read_mesh(
                        doc,
                        r::Mesh::<r::V2> {
                            name: mesh.name,
                            lod_level: mesh.lod_level,
                            face_set: mesh.face_set,
                            attributes: mesh.attributes,
                            skin: mesh.skin,
                            sections: mesh.sections,
                            repair_log: mesh.repair_log,
                            bounds: mesh.bounds,
                            ..Default::default()
                        },
                        index as u32,
                    )?;
                    model.meshes.extend(values);
                    mesh_projections.extend(projections);
                }
            }
            _ => return Err(ReadError::unsupported(0, "mesh revision")),
        }
    }
    macro_rules! node_rows {
        ($v:ty) => {
            for node in rows::<r::Node<$v>>(doc, nodes.cast())? {
                model.nodes.push(Node {
                    name: text_view(doc, node.name)?,
                    parent: (node.parent != u32::MAX).then_some(node.parent),
                    pivot: node.pivot,
                    translation: node.local.translation,
                    rotation: node.local.rotation,
                    scale: node.local.scale,
                    flags: node.flags.0,
                    bone: node.kind == r::NodeKind::BONE,
                    ..Default::default()
                });
            }
        };
    }
    if nodes.entries > 0 {
        let section = doc
            .referenced_section(nodes)?
            .ok_or_else(|| ReadError::invalid(0, "node table"))?;
        match section.revision() {
            1 => node_rows!(r::V1),
            2 => node_rows!(r::V2),
            3 => node_rows!(r::V3),
            4 => node_rows!(r::V4),
            5 => node_rows!(r::V5),
            6 => node_rows!(r::V6),
            7 => node_rows!(r::V7),
            8 => node_rows!(r::V8),
            9 => node_rows!(r::V9),
            10 => node_rows!(r::V10),
            11 => node_rows!(r::V11),
            12 => node_rows!(r::V12),
            13 => node_rows!(r::V13),
            14 => node_rows!(r::V14),
            15 => node_rows!(r::V15),
            16 => node_rows!(r::V16),
            17 => node_rows!(r::V17),
            18 => node_rows!(r::V18),
            19 => {
                for node in rows::<r::Node<r::V19>>(doc, nodes.cast())? {
                    let extra = metadata_read(doc, node.native)?;
                    let matrix = extra
                        .get("conversion.inverse_bind")
                        .map(|(_, hex)| decode_matrix(hex))
                        .transpose()?;
                    model.nodes.push(Node {
                        name: text_view(doc, node.name)?,
                        parent: (node.parent != u32::MAX).then_some(node.parent),
                        pivot: node.pivot,
                        translation: node.local.translation,
                        rotation: node.local.rotation,
                        scale: node.local.scale,
                        inverse_bind: matrix,
                        flags: node.flags.0,
                        bone: node.kind == r::NodeKind::BONE,
                    });
                }
            }
            _ => return Err(ReadError::unsupported(0, "node revision")),
        }
    }
    for texture in rows::<r::TextureRecord>(doc, root.textures.cast())? {
        let (id, group) = match texture.key {
            r::TextureKey::FileDataId(key) => (Some(key.value), 0),
            r::TextureKey::SnoId(key) => (Some(key.id), key.group),
            _ => (None, 0),
        };
        model.textures.push(Texture {
            path: text_view(doc, texture.path)?,
            id,
            group,
            replaceable: texture.replaceable_id,
            flags: texture.flags,
        });
    }
    let mut slot_materials = Vec::new();
    let mesh_slots = model.meshes.iter().map(|m| m.material_slot).collect();
    let sets = rows::<r::ProfileMaterialSet>(doc, profiles.cast())?;
    if let Some(set) = sets.into_iter().find(|s| s.profile == profile) {
        let look = look.unwrap_or(set.default_look);
        if look >= set.looks.looks.entries {
            return Err(ReadError::invalid(0, "selected material look"));
        }
        for binding in rows::<r::SlotBinding>(doc, set.slot_bindings.cast())? {
            let indices = rows::<r::U32>(doc, binding.by_look.cast())?;
            let material = indices
                .get(look as usize)
                .ok_or_else(|| ReadError::invalid(0, "material look binding"))?
                .value;
            if material != u32::MAX && material >= set.materials.entries {
                return Err(ReadError::invalid(0, "bound material index"));
            }
            slot_materials.push(material);
        }
        for mesh in &mut model.meshes {
            if mesh.material_slot != u32::MAX {
                mesh.material_slot = *slot_materials
                    .get(mesh.material_slot as usize)
                    .ok_or_else(|| ReadError::invalid(0, "mesh material slot binding"))?;
            }
        }
        for material in rows::<r::MaterialRecord>(doc, set.materials.cast())?.into_iter() {
            let texture = if let r::MaterialBody::Composite(reference) = material.common.body {
                let section = doc.referenced_section(reference)?;
                match section.map(|s| s.revision()) {
                    Some(2) => rows::<r::CompositeBody<r::V2>>(doc, reference.cast())?
                        .first()
                        .map(|b| b.layers),
                    Some(1) => rows::<r::CompositeBody<r::V1>>(doc, reference.cast())?
                        .first()
                        .map(|b| b.layers),
                    None => None,
                    _ => return Err(ReadError::unsupported(0, "composite material revision")),
                }
                .map(|layers| rows::<r::CompositeLayer>(doc, layers.cast()))
                .transpose()?
                .and_then(|layers| layers.first().map(|r| r.input.texture))
                .filter(|&id| id != u32::MAX)
            } else {
                None
            };
            model.materials.push(Material {
                name: text_view(doc, material.name)?,
                texture,
                blend: material.common.blend.0,
                flags: material.common.flags.0,
            });
        }
    }
    let channels = rows::<r::AnimChannel>(doc, channels.cast())?;
    macro_rules! clip_rows {
        ($v:ty) => {
            rows::<r::Clip<$v>>(doc, root.clips.cast())?
                .into_iter()
                .map(|c| (c.name, c.model, c.duration, c.flags, c.containers, c.native))
                .collect::<Vec<_>>()
        };
    }
    if root.clips.entries > 0 {
        let section = doc
            .referenced_section(root.clips)?
            .ok_or_else(|| ReadError::invalid(0, "clip table"))?;
        let clips = match section.revision() {
            1 => clip_rows!(r::V1),
            2 => clip_rows!(r::V2),
            3 => clip_rows!(r::V3),
            4 => clip_rows!(r::V4),
            5 => clip_rows!(r::V5),
            6 => clip_rows!(r::V6),
            7 => clip_rows!(r::V7),
            _ => return Err(ReadError::unsupported(0, "clip revision")),
        };
        for (name, owner, duration, flags, containers, native) in clips {
            if owner != index {
                continue;
            }
            let meta = metadata_read(doc, native)?;
            let mut clip = Clip {
                name: text_view(doc, name)?,
                duration,
                flags: flags.0,
                id: meta.get("conversion.clip_id").map_or(0, |v| v.0 as u32),
                ..Default::default()
            };
            for (container_index, container) in
                rows::<r::SubTrackContainer>(doc, containers.cast())?
                    .into_iter()
                    .enumerate()
            {
                let extra = metadata_read(doc, container.native)?;
                let section = doc.referenced_section(container.sub_tracks)?;
                let tracks = match section.map(|s| s.revision()) {
                    Some(2) => rows::<r::SubTrack<r::V2>>(doc, container.sub_tracks.cast())?
                        .into_iter()
                        .map(|t| (t.channel, t.interp, t.times, t.values))
                        .collect::<Vec<_>>(),
                    Some(1) => rows::<r::SubTrack<r::V1>>(doc, container.sub_tracks.cast())?
                        .into_iter()
                        .map(|t| (t.channel, t.interp, t.times, t.values))
                        .collect(),
                    None => Vec::new(),
                    _ => return Err(ReadError::unsupported(0, "subtrack revision")),
                };
                for (id, interp, times, values) in tracks {
                    let channel = channels
                        .iter()
                        .find(|c| c.id == id)
                        .ok_or_else(|| ReadError::invalid(0, "animation channel identity"))?;
                    let layer = TrackLayer {
                        name: text_view(doc, container.name)?.into_owned(),
                        priority: container.priority,
                        concurrent: container.concurrent.get(),
                        source: Some(container_index as u32),
                    };
                    let target = match channel.target.channel {
                        r::Channel(255) => Target::NativeProperty {
                            owner: channel.target.node,
                            id: channel.target.sub,
                        },
                        r::Channel::TRANSLATION => Target::Translation(channel.target.node),
                        r::Channel::ROTATION => Target::Rotation(channel.target.node),
                        r::Channel::SCALE => Target::Scale(channel.target.node),
                        r::Channel::EMITTER_PROPERTY => Target::Property {
                            owner: channel.target.node,
                            channel: channel.target.sub,
                        },
                        code => Target::Property {
                            owner: channel.target.node,
                            channel: u32::from(code.0),
                        },
                    };
                    let interp = match interp {
                        r::Interpolation::STEP => Interpolation::Step,
                        r::Interpolation::LINEAR | r::Interpolation::SLERP => Interpolation::Linear,
                        r::Interpolation::HERMITE => Interpolation::Hermite,
                        r::Interpolation::BEZIER => Interpolation::Bezier,
                        _ => return Err(ReadError::unsupported(0, "animation interpolation")),
                    };
                    if matches!(
                        channel.value_type,
                        r::AttrType::U32
                            | r::AttrType::I32
                            | r::AttrType::U16
                            | r::AttrType::U8X4
                            | r::AttrType::BOOL
                            | r::AttrType::U32X4
                    ) {
                        let times = rows::<r::Real>(doc, times.cast())?;
                        clip.integer_tracks.push(integer::decode(
                            channel.value_type,
                            target,
                            interp,
                            layer,
                            extra
                                .get("conversion.global_duration")
                                .map(|v| f32::from_bits(v.0 as u32)),
                            &times,
                            rawbytes(doc, values)?,
                        )?);
                        continue;
                    }
                    let width = match channel.value_type {
                        r::AttrType::F32 => 1,
                        r::AttrType::F32X2 => 2,
                        r::AttrType::F32X3 => 3,
                        r::AttrType::F32X4 | r::AttrType::QUAT => 4,
                        _ => return Err(ReadError::unsupported(0, "animation value type")),
                    };
                    let times = rows::<r::Real>(doc, times.cast())?;
                    let data = rawbytes(doc, values)?;
                    let factor = if matches!(interp, Interpolation::Hermite | Interpolation::Bezier)
                    {
                        3
                    } else {
                        1
                    };
                    let stride = width * 4 * factor;
                    if data.len() != times.len() * stride {
                        return Err(ReadError::invalid(0, "animation key width"));
                    }
                    let mut keys = Vec::new();
                    for (time, key) in times.into_iter().zip(data.chunks_exact(stride)) {
                        let components: Vec<_> = key
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .map(|b| f32::from_le_bytes(*b))
                            .collect();
                        keys.push(Key {
                            time: time.value,
                            value: components[..width].to_vec(),
                            incoming: (factor == 3).then(|| components[width..width * 2].to_vec()),
                            outgoing: (factor == 3).then(|| components[width * 2..].to_vec()),
                        });
                    }
                    clip.tracks.push(Track {
                        layer,
                        target,
                        interpolation: interp,
                        global_duration: extra
                            .get("conversion.global_duration")
                            .map(|v| f32::from_bits(v.0 as u32)),
                        keys,
                    });
                }
            }
            model.clips.push(clip);
        }
    }
    validate_model(&model)?;
    model.native = Some(NativeModel {
        document: document.as_borrowed(),
        model: index,
        slot_materials,
        mesh_slots,
        mesh_projections,
    });
    Ok(model)
}
