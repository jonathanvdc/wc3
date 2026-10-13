//! Native Warcraft III import orchestration.
use super::*;
/// Imports editable geometry, native records, resources, and animation channels.
pub fn import<D: ModelDialect>(
    source: &native::Model<'_, D>,
    options: &ImportOptions,
) -> Result<Conversion<Asset>, Error>
where
    D::Extension: Chunk,
{
    let mut context = Context::import(options);
    for (index, chunk) in source.chunks.iter().enumerate() {
        if matches!(chunk, ModelChunk::Unknown(_) | ModelChunk::Extension(_)) {
            match options.unknown {
                UnknownPolicy::Reject => {
                    return Err(Error::new(
                        format!("chunks[{index}]"),
                        "unknown/application chunk requires an explicit archive or drop policy",
                    ));
                }
                UnknownPolicy::Archive => context.issue(
                    format!("chunks[{index}]"),
                    IssueKind::Archived,
                    "Opaque source chunk retained in inactive provenance",
                ),
                UnknownPolicy::Drop => context.issue(
                    format!("chunks[{index}]"),
                    IssueKind::Dropped,
                    "Opaque source chunk omitted",
                ),
            }
        }
    }
    let mut model = s::Model {
        profile: if D::Version::NUMBER >= 900 {
            ProfileId::Wc3Reforged
        } else {
            ProfileId::Wc3Classic
        },
        ..Default::default()
    };
    if let Some(info) = source.model_info() {
        model.name = Cow::Owned(text(&info.name));
        model.bounds = wem::records::Extent {
            minimum: info.minimum_extent,
            maximum: info.maximum_extent,
            sphere_radius: info.bounds_radius,
        };
    }
    let nodes = source.nodes();
    let mut ids = BTreeMap::new();
    for (index, node) in nodes.iter().enumerate() {
        if ids.insert(node.object_id, index as u32).is_some() {
            return Err(Error::new(
                format!("nodes[{index}].object_id"),
                "duplicate object identity",
            ));
        }
    }
    let pivots = source.pivot_points();
    for (index, node) in nodes.iter().enumerate() {
        model.nodes.push(s::Node {
            name: Cow::Owned(text(node.name)),
            parent: if node.parent_id == u32::MAX {
                None
            } else {
                Some(*ids.get(&node.parent_id).ok_or_else(|| {
                    Error::new(format!("nodes[{index}].parent"), "missing parent identity")
                })?)
            },
            pivot: pivots
                .get(node.object_id as usize)
                .copied()
                .unwrap_or_default(),
            flags: common_flags(node.flags),
            bone: node.flags.bone()
                || source
                    .bones()
                    .any(|bone| bone.node.object_id == node.object_id),
            ..Default::default()
        });
    }
    for (index, mesh) in source.geosets().enumerate() {
        if mesh.primitive_types().iter().any(|kind| kind != 4) || mesh.face_indices().len() % 3 != 0
        {
            return Err(Error::new(
                format!("meshes[{index}].triangles"),
                "MDLX primitive stream is not a triangle list",
            ));
        }
        let faces: Vec<_> = mesh.face_indices().iter().map(u32::from).collect();
        let influences = influences(mesh)
            .map_err(|error| Error::new(format!("meshes[{index}].skin"), error))?
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|(node, weight)| {
                        Ok(s::Influence {
                            node: *ids.get(&node).ok_or_else(|| {
                                Error::new(
                                    format!("meshes[{index}].skin"),
                                    "missing influence node",
                                )
                            })?,
                            weight,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()
            })
            .collect::<Result<Vec<_>, Error>>()?;
        model.meshes.push(s::Mesh {
            name: Cow::Owned(
                mesh.try_name()
                    .map(|name| name.into_owned())
                    .unwrap_or_default(),
            ),
            positions: Cow::Owned(mesh.vertices().iter().collect()),
            normals: Cow::Owned(mesh.normals().iter().collect()),
            tangents: Cow::Owned(
                mesh.try_tangents()
                    .ok()
                    .flatten()
                    .map(|values| values.iter().collect())
                    .unwrap_or_default(),
            ),
            uv_sets: mesh
                .uv_sets()
                .iter()
                .map(|values| Cow::Owned(values.iter().collect()))
                .collect(),
            triangles: Cow::Owned(
                faces
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .map(|face| [face[0], face[1], face[2]])
                    .collect(),
            ),
            influences,
            material_slot: mesh.material_id,
            selection_group: mesh.selection_group.min(u32::from(u16::MAX)),
            lod: mesh.try_level_of_detail().unwrap_or(0),
            bounds: extent(mesh.extent),
            ..Default::default()
        });
    }
    for texture in source.textures() {
        model.textures.push(s::Texture {
            path: Cow::Owned(text(&texture.path)),
            replaceable: texture.replaceable_id,
            flags: wr::TextureFlags(texture.flags.bits()),
            ..Default::default()
        });
    }
    for (index, material) in source.materials().enumerate() {
        let layer = material.layers.first();
        model.materials.push(s::Material {
            name: Cow::Owned(format!("Material {index}")),
            texture: layer.and_then(|layer| layer.texture_id.value()).copied(),
            blend: layer
                .map(|layer| filter(layer.filter_mode))
                .unwrap_or_default(),
            flags: common_material_flags(material),
        });
    }
    let globals = source.global_sequences();
    let sequences: Vec<_> = source.sequences().collect();
    if sequences.is_empty()
        && nodes.iter().any(|node| {
            node.translation.is_some() || node.rotation.is_some() || node.scaling.is_some()
        })
    {
        model.clips.push(s::Clip {
            name: Cow::Borrowed("Timeline"),
            duration: 0.,
            ..Default::default()
        });
    }
    for sequence in sequences {
        model.clips.push(s::Clip {
            name: Cow::Owned(text(&sequence.name)),
            duration: (sequence.interval[1] - sequence.interval[0]) as f32 / 1000.,
            id: sequence.interval[0],
            flags: wr::ClipFlags::default(),
            looping: !sequence.flags.non_looping(),
            tracks: Vec::new(),
            integer_tracks: Vec::new(),
        });
    }
    for clip in &mut model.clips {
        let start = clip.id as f32 / 1000.;
        for (index, node) in nodes.iter().enumerate() {
            if let Some(track) = node.translation {
                clip.tracks.push(semantic_track(
                    read_track(track),
                    s::Target::Translation(index as u32),
                    start,
                    &globals,
                ));
            }
            if let Some(track) = node.rotation {
                clip.tracks.push(semantic_track(
                    read_track(track),
                    s::Target::Rotation(index as u32),
                    start,
                    &globals,
                ));
            }
            if let Some(track) = node.scaling {
                clip.tracks.push(semantic_track(
                    read_track(track),
                    s::Target::Scale(index as u32),
                    start,
                    &globals,
                ));
            }
        }
    }
    for camera in source.cameras() {
        let owner = model.nodes.len() as u32;
        model.nodes.push(s::Node {
            name: Cow::Owned(text(&camera.name)),
            translation: camera.position,
            ..Default::default()
        });
        for clip in &mut model.clips {
            if let Some(track) = &camera.translation {
                clip.tracks.push(semantic_track(
                    read_track(track),
                    s::Target::Translation(owner),
                    clip.id as f32 / 1000.,
                    &globals,
                ));
            }
        }
    }
    if let Ok(entries) = source.try_face_fx() {
        for entry in entries {
            model.nodes.push(s::Node {
                name: Cow::Owned(text(&entry.name)),
                ..Default::default()
            });
        }
    }
    import_properties(source, &mut model, &globals, &mut context)?;
    let document = decorate_metadata(decorate_native(s::build(&model)?, source)?, source)?;
    let mut asset = Asset::new(document);
    if options.unknown == UnknownPolicy::Archive {
        for chunk in &source.chunks {
            let (tag, bytes) = match chunk {
                ModelChunk::Unknown(value) => (value.raw().tag, value.raw().data.to_vec()),
                ModelChunk::Extension(value) => {
                    let mut bytes = Vec::new();
                    value
                        .encode_payload_to(&mut native::Encoder::new(&mut bytes))
                        .map_err(|error| failure("extension", error))?;
                    (value.tag(), bytes)
                }
                _ => continue,
            };
            asset.opaque.push(OpaquePayload {
                format: Format::Mdlx,
                file: FileKey::Root,
                tag,
                revision: Some(D::Version::NUMBER),
                count: None,
                bytes: Cow::Owned(bytes),
            });
        }
    }

    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}
