use super::*;
use std::collections::BTreeSet;
use std::slice::from_ref;
pub(super) fn build_impl(
    model: &Model<'_>,
    options: Options,
) -> Result<Document<'static, V3>, WriteError> {
    validate_model(model).map_err(|_| WriteError::invalid("semantic model"))?;
    let (mut doc, original) = editing::prepare(model, options)?;
    let mut meshes = Vec::new();
    if model
        .native
        .as_ref()
        .is_some_and(|n| !n.mesh_projections.is_empty())
    {
        meshes = projection::rebuild(model, &mut doc)?;
    } else {
        for (mesh_index, mesh) in model.meshes.iter().enumerate() {
            let mut layers = Vec::new();
            for layer in [
                attribute(&mut doc, "position", &mesh.positions, r::AttrType::F32X3)?,
                attribute(&mut doc, "normal", &mesh.normals, r::AttrType::F32X3)?,
                attribute(&mut doc, "tangent", &mesh.tangents, r::AttrType::F32X4)?,
                attribute(&mut doc, "color", &mesh.colors, r::AttrType::F32X4)?,
            ]
            .into_iter()
            .flatten()
            {
                layers.push(layer);
            }
            for (i, uv) in mesh.uv_sets.iter().enumerate() {
                if let Some(layer) = attribute(&mut doc, &format!("uv{i}"), uv, r::AttrType::F32X2)?
                {
                    layers.push(layer);
                }
            }
            let corner: Vec<_> = mesh.triangles.iter().flatten().copied().collect();
            let valences = vec![3; mesh.triangles.len()];
            let mut offsets = vec![0];
            let mut influences = Vec::new();
            for vertex in &mesh.influences {
                for influence in vertex {
                    influences.push(r::Influence {
                        bone: influence.node,
                        weight: influence.weight,
                    });
                }
                offsets.push(
                    u32::try_from(influences.len())
                        .map_err(|_| WriteError::invalid("influence count"))?,
                );
            }
            let skin = if mesh.influences.is_empty() {
                r::SkinBinding::default()
            } else {
                r::SkinBinding {
                    offsets: words(&mut doc, &offsets)?,
                    influences: put(&mut doc, &influences)?.cast(),
                }
            };
            let section = r::MeshSection {
                name: text(&mut doc, &mesh.name)?,
                material_slot: editing::material_slot(model, mesh_index, mesh.material_slot)?,
                selection_group: u16::try_from(mesh.selection_group)
                    .map_err(|_| WriteError::invalid("selection group"))?,
                bounds: mesh.bounds.clone(),
                ..Default::default()
            };
            let sections = put(&mut doc, &[section])?.cast();
            meshes.push(r::Mesh::<r::V2> {
                name: text(&mut doc, &mesh.name)?,
                lod_level: mesh.lod,
                face_set: r::FaceSet {
                    vertex_count: u32::try_from(mesh.positions.len())
                        .map_err(|_| WriteError::invalid("vertex count"))?,
                    face_valence: words(&mut doc, &valences)?,
                    corner_vertex: words(&mut doc, &corner)?,
                },
                attributes: r::AttributeSet {
                    domain_counts: [
                        mesh.positions.len() as u32,
                        corner.len() as u32,
                        0,
                        valences.len() as u32,
                        1,
                    ],
                    layers: put(&mut doc, &layers)?.cast(),
                },
                skin,
                sections,
                bounds: mesh.bounds.clone(),
                ..Default::default()
            });
        }
    }
    let mut nodes = Vec::new();
    for (node_index, node) in model.nodes.iter().enumerate() {
        let node_metadata = match node.inverse_bind {
            Some(value) => {
                let hex = value
                    .iter()
                    .flatten()
                    .flat_map(|v| v.to_le_bytes())
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                metadata(&mut doc, &[("conversion.inverse_bind", 64, hex)])?
            }
            None => r::NativeBag::default(),
        };
        let payload = if node.bone {
            r::NodePayload::Bone(Box::default())
        } else {
            r::NodePayload::Helper(Box::default())
        };
        let node_name = text(&mut doc, &node.name)?;
        nodes.push(editing::node(
            &mut doc,
            r::Node::<r::V19> {
                name: node_name,
                parent: node.parent.unwrap_or(u32::MAX),
                kind: if node.bone {
                    r::NodeKind::BONE
                } else {
                    r::NodeKind::HELPER
                },
                flags: r::NodeFlags(node.flags),
                pivot: node.pivot,
                local: r::Transform {
                    translation: node.translation,
                    rotation: node.rotation,
                    scale: node.scale,
                },
                native: node_metadata,
                payload,
                ..Default::default()
            },
            original.as_ref().and_then(|o| o.nodes.get(node_index)),
        )?);
    }
    let mut textures = Vec::new();
    for texture in &model.textures {
        let path = text(&mut doc, &texture.path)?;
        let key = match texture.id {
            Some(id) if texture.group != 0 => r::TextureKey::SnoId(r::TextureSnoId {
                group: texture.group,
                id,
            }),
            Some(id) => r::TextureKey::FileDataId(r::TextureFileDataId { value: id }),
            None => r::TextureKey::Path(r::TexturePath { value: path }),
        };
        textures.push(r::TextureRecord {
            key,
            path,
            flags: texture.flags,
            replaceable_id: texture.replaceable,
            ..Default::default()
        });
    }
    let mut materials = Vec::new();
    for (material_index, material) in model.materials.iter().enumerate() {
        let input = r::TextureInput {
            texture: material.texture.unwrap_or(u32::MAX),
            weight: 1.,
            ..Default::default()
        };
        let layer = r::CompositeLayer {
            input,
            ..Default::default()
        };
        let body = r::CompositeBody::<r::V2> {
            layers: put(&mut doc, &[layer])?.cast(),
            diffuse_factor: [1.; 4],
            ..Default::default()
        };
        let body = put(&mut doc, &[body])?.cast();
        let material_name = text(&mut doc, &material.name)?;
        materials.push(editing::material(
            &doc,
            r::MaterialRecord {
                name: material_name,
                common: r::CommonMaterial {
                    blend: r::BlendMode(material.blend),
                    flags: r::MaterialFlags(material.flags),
                    depth: r::DepthState {
                        test: true.into(),
                        write: true.into(),
                        bias: 0.,
                    },
                    cull: r::CullMode::BACK,
                    body: r::MaterialBody::Composite(body),
                    ..Default::default()
                },
                ..Default::default()
            },
            original
                .as_ref()
                .and_then(|o| o.materials.get(material_index)),
        ));
    }
    let mut clip_rows = Vec::new();
    let mut channels: Vec<r::AnimChannel> = Vec::new();
    let reserved_ids: BTreeSet<_> = model
        .clips
        .iter()
        .flat_map(|c| {
            c.tracks
                .iter()
                .map(|t| t.target)
                .chain(c.integer_tracks.iter().map(IntegerTrack::target))
        })
        .filter_map(|t| match t {
            Target::NativeProperty { id, .. } => Some(id),
            _ => None,
        })
        .collect();
    let mut next_channel = 0u32;
    for (clip_index, clip) in model.clips.iter().enumerate() {
        let mut tracks = Vec::new();
        for track in &clip.tracks {
            let (node, channel, width) = match track.target {
                Target::Translation(n) => (n, r::Channel::TRANSLATION, 3),
                Target::Rotation(n) => (n, r::Channel::ROTATION, 4),
                Target::Scale(n) => (n, r::Channel::SCALE, 3),
                Target::NativeProperty { owner, .. } => (
                    owner,
                    r::Channel(255),
                    track.keys.first().map_or(1, |k| k.value.len()),
                ),
                Target::Property { owner, .. } => (
                    owner,
                    r::Channel::EMITTER_PROPERTY,
                    track.keys.first().map_or(1, |k| k.value.len()),
                ),
            };
            let id = if let Target::NativeProperty { id, .. } = track.target {
                id
            } else {
                while reserved_ids.contains(&next_channel) {
                    next_channel = next_channel
                        .checked_add(1)
                        .ok_or_else(|| WriteError::invalid("channel identity overflow"))?;
                }
                let id = next_channel;
                next_channel = next_channel
                    .checked_add(1)
                    .ok_or_else(|| WriteError::invalid("channel identity overflow"))?;
                id
            };
            let kind = match width {
                1 => r::AttrType::F32,
                2 => r::AttrType::F32X2,
                3 => r::AttrType::F32X3,
                4 => r::AttrType::F32X4,
                _ => return Err(WriteError::invalid("animation component count")),
            };
            let init: Vec<_> = track
                .keys
                .first()
                .into_iter()
                .flat_map(|k| k.value.iter().flat_map(|v| v.to_le_bytes()))
                .collect();
            let mut channel_record = r::AnimChannel {
                id,
                target: r::TrackTarget {
                    kind: r::TrackTargetKind::NODE,
                    node,
                    channel,
                    sub: match track.target {
                        Target::Property { channel, .. } => channel,
                        Target::NativeProperty { id, .. } => id,
                        _ => 0,
                    },
                    ..Default::default()
                },
                value_type: kind,
                init_value: bytes(&mut doc, &init)?,
            };
            if let Target::NativeProperty { .. } = track.target {
                if let Some(old) = original.as_ref().and_then(|o| {
                    o.channels.iter().find(|c| {
                        c.id == id && c.target.channel == r::Channel(255) && c.value_type == kind
                    })
                }) {
                    channel_record.target = old.target.clone();
                    channel_record.target.node = node;
                    channel_record.target.channel = channel;
                    channel_record.target.sub = id;
                    channel_record.init_value = old.init_value;
                }
            }
            if let Some(existing) = channels.iter().find(|c| c.id == id) {
                if existing.target != channel_record.target || existing.value_type != kind {
                    return Err(WriteError::invalid("conflicting native animation identity"));
                }
            } else {
                channels.push(channel_record);
            }
            let interp = match track.interpolation {
                Interpolation::Step => r::Interpolation::STEP,
                Interpolation::Linear => r::Interpolation::LINEAR,
                Interpolation::Hermite => r::Interpolation::HERMITE,
                Interpolation::Bezier => r::Interpolation::BEZIER,
            };
            let mut values = Vec::new();
            for key in &track.keys {
                for v in &key.value {
                    values.extend_from_slice(&v.to_le_bytes());
                }
                if matches!(
                    track.interpolation,
                    Interpolation::Hermite | Interpolation::Bezier
                ) {
                    for tangent in [&key.incoming, &key.outgoing] {
                        let tangent = tangent
                            .as_ref()
                            .ok_or_else(|| WriteError::invalid("missing spline tangent"))?;
                        for v in tangent {
                            values.extend_from_slice(&v.to_le_bytes());
                        }
                    }
                }
            }
            let sub = r::SubTrack::<r::V2> {
                channel: id,
                interp,
                times: floats(
                    &mut doc,
                    &track.keys.iter().map(|k| k.time).collect::<Vec<_>>(),
                )?,
                values: bytes(&mut doc, &values)?,
                tcb: Ref::default(),
            };
            let mut extra = Vec::new();
            if let Some(duration) = track.global_duration {
                extra.push((
                    "conversion.global_duration",
                    duration.to_bits() as i64,
                    String::new(),
                ));
            }
            let container = r::SubTrackContainer {
                name: text(&mut doc, &track.layer.name)?,
                priority: track.layer.priority,
                concurrent: track.layer.concurrent.into(),
                sub_tracks: put(&mut doc, &[sub])?.cast(),
                native: metadata(&mut doc, &extra)?,
            };
            tracks.push(editing::container(
                &mut doc,
                container,
                &track.layer,
                original.as_ref(),
                clip_index,
            )?);
        }
        for track in &clip.integer_tracks {
            let container = integer::encode(
                track,
                &mut doc,
                &mut channels,
                &mut next_channel,
                &reserved_ids,
                original.as_ref(),
            )?;
            tracks.push(editing::container(
                &mut doc,
                container,
                track.layer(),
                original.as_ref(),
                clip_index,
            )?);
        }
        let extra = metadata(
            &mut doc,
            &[("conversion.clip_id", clip.id as i64, String::new())],
        )?;
        clip_rows.push(r::Clip::<r::V7> {
            name: text(&mut doc, &clip.name)?,
            model: 0,
            duration: clip.duration,
            flags: r::ClipFlags(clip.flags),
            containers: put(&mut doc, &tracks)?.cast(),
            native: extra,
            ..Default::default()
        });
    }
    let look = r::Look {
        name: text(&mut doc, "Default")?,
        ..Default::default()
    };
    let look_ref = put(&mut doc, &[look])?.cast();
    let mut bindings = Vec::new();
    let mut slots = Vec::new();
    for (i, material) in model.materials.iter().enumerate() {
        bindings.push(r::SlotBinding {
            by_look: words(&mut doc, &[i as u32])?,
        });
        slots.push(text(&mut doc, &material.name)?);
    }
    let profile = r::ProfileMaterialSet {
        profile: model.profile,
        looks: r::LookTable { looks: look_ref },
        materials: put(&mut doc, &materials)?.cast(),
        slot_bindings: put(&mut doc, &bindings)?.cast(),
        ..Default::default()
    };
    let root_model = r::ModelRecord::<r::V9> {
        name: text(&mut doc, &model.name)?,
        bounds: model.bounds.clone(),
        meshes: put(&mut doc, &meshes)?.cast(),
        nodes: r::NodeTree {
            nodes: put(&mut doc, &nodes)?.cast(),
            ..Default::default()
        },
        material_slots: InlineList(slots),
        anim_channels: r::AnimChannelTable {
            channels: put(&mut doc, &channels)?.cast(),
        },
        profile_sets: put(&mut doc, from_ref(&profile))?.cast(),
        anim_set: u32::MAX,
        t_pose: u32::MAX,
        ..Default::default()
    };
    let texture_ref = put(&mut doc, &textures)?.cast();
    if let Some(original) = original {
        editing::finish(
            &mut doc,
            root_model,
            profile,
            texture_ref,
            clip_rows,
            original,
        )?;
        doc.prune_unreachable()?;
        return Document::from_raw(doc)
            .map_err(|_| WriteError::invalid("edited native document graph"));
    }
    let root = r::DocumentRecord::<r::V1> {
        profiles: InlineList(vec![model.profile]),
        default_profile: model.profile,
        unit_scale: model.unit_scale,
        space: model.space,
        name: text(&mut doc, &model.name)?,
        bounds: model.bounds.clone(),
        models: put(&mut doc, &[root_model])?.cast(),
        textures: texture_ref,
        clips: put(&mut doc, &clip_rows)?.cast(),
        ..Default::default()
    };
    let root_ref = put(&mut doc, &[root])?;
    doc.set_root(root_ref);
    Document::from_raw(doc).map_err(|_| WriteError::invalid("semantic document graph"))
}
