//! Skin profiles, triangle windows, bone palettes, and packed vertex attributes.
use super::*;

pub(super) fn section_start<V: ModelLayout>(section: &SkinSection<V>) -> usize {
    (usize::from(section.level) << 16) | usize::from(section.index_start)
}

pub(super) fn profile_vertices<V: ModelLayout>(
    core: &Core<'_, V>,
    profile: &SkinProfile<'_, V>,
) -> Result<Vec<Vertex>, native::ReadError> {
    let base = if core.flags.global_skin_vertex_offsets() {
        profile.lod_vertex_base as usize
    } else {
        0
    };
    profile
        .vertices
        .iter()
        .map(|index| {
            core.vertices.get(base + usize::from(index)).ok_or_else(|| {
                native::ReadError::invalid(0, "skin references a missing model vertex")
            })
        })
        .collect()
}

pub(super) fn import_profile<V: ModelLayout>(
    core: &Core<'_, V>,
    profile: &SkinProfile<'_, V>,
    profile_index: usize,
    lod: bool,
    model: &mut sm::Model<'static>,
    native_materials: &mut Vec<wr::M2Material>,
    context: &mut Context,
) -> Result<(), Error> {
    let vertices = profile_vertices(core, profile).map_err(failure)?;
    for (section_index, section) in profile.sections.iter().enumerate() {
        let start = section_start(section);
        let end = start
            .checked_add(usize::from(section.index_count))
            .ok_or_else(|| Error::new("m2.section.indices", "triangle range overflow"))?;
        if end > profile.indices.len() || !section.index_count.is_multiple_of(3) {
            return Err(Error::new(
                format!("profiles[{profile_index}].sections[{section_index}]"),
                "invalid triangle range",
            ));
        }
        let first = usize::from(section.vertex_start);
        let vertex_end = first + usize::from(section.vertex_count);
        if vertex_end > vertices.len() {
            return Err(Error::new(
                "m2.section.vertices",
                "section references missing profile vertices",
            ));
        }
        let selected = &vertices[first..vertex_end];
        let triangles: Vec<_> = (start..end)
            .step_by(3)
            .map(|at| {
                let mut triangle = [0; 3];
                for (corner, index) in triangle.iter_mut().enumerate() {
                    let value = usize::from(profile.indices.get(at + corner).unwrap());
                    if value < first || value >= vertex_end {
                        return Err(Error::new(
                            "m2.section.triangles",
                            "triangle escapes its section vertex window",
                        ));
                    }
                    *index = (value - first) as u32;
                }
                Ok(triangle)
            })
            .collect::<Result<_, Error>>()?;
        let batches: Vec<_> = profile
            .batches
            .iter()
            .filter(|batch| usize::from(batch.skin_section_index) == section_index)
            .collect();
        let mut mesh = sm::Mesh {
            name: Cow::Owned(format!("skin-{profile_index}-section-{section_index}")),
            positions: Cow::Owned(selected.iter().map(|v| v.position).collect()),
            normals: Cow::Owned(selected.iter().map(|v| v.normal).collect()),
            uv_sets: (0..2)
                .map(|set| {
                    Cow::Owned(
                        selected
                            .iter()
                            .map(|v| v.texture_coordinates[set])
                            .collect(),
                    )
                })
                .collect(),
            triangles: Cow::Owned(triangles),
            influences: selected
                .iter()
                .map(|v| {
                    v.bone_weights
                        .into_iter()
                        .zip(v.bone_indices)
                        .filter(|(weight, _)| *weight != 0)
                        .map(|(weight, node)| sm::Influence {
                            node: u32::from(node),
                            weight: f32::from(weight) / 255.,
                        })
                        .collect()
                })
                .collect(),
            selection_group: u32::from(section.id),
            lod: if lod {
                profile_index as u32 + 1
            } else {
                profile_index as u32
            },
            bounds: model.bounds.clone(),
            material_slot: u32::MAX,
            ..Default::default()
        };
        if batches.is_empty() {
            model.meshes.push(mesh);
            continue;
        }
        if batches.len() > 1 {
            context.issue(
                format!("profiles[{profile_index}].sections[{section_index}]"),
                IssueKind::Normalized,
                "material passes represented as coincident draw meshes",
            );
        }
        for batch in batches {
            let material = core
                .materials
                .get(usize::from(batch.material_index))
                .ok_or_else(|| Error::new("m2.batch.material", "missing render-state material"))?;
            let mut units = Vec::new();
            for slot in 0..usize::from(batch.texture_count) {
                let texture = core
                    .texture_combos
                    .get(usize::from(batch.texture_combo_index) + slot)
                    .ok_or_else(|| {
                        Error::new("m2.batch.texture", "missing texture-combination entry")
                    })?;
                let texture_record = core
                    .textures
                    .get(usize::from(texture))
                    .ok_or_else(|| Error::new("m2.batch.texture", "missing texture resource"))?;
                if texture_record.kind.raw() > u32::from(u8::MAX) {
                    return Err(Error::new(
                        "m2.texture.kind",
                        "native WEM material cannot encode this replacement category",
                    ));
                }
                units.push(wr::M2TextureUnit {
                    texture,
                    uv_set: core
                        .texture_transform_bone_map
                        .get(usize::from(batch.texture_coordinate_combo_index) + slot)
                        .unwrap_or(0),
                    transform: core
                        .texture_transform_combos
                        .get(usize::from(batch.texture_transform_combo_index) + slot)
                        .unwrap_or(u16::MAX),
                    weight: core
                        .texture_weight_combos
                        .get(usize::from(batch.texture_weight_combo_index) + slot)
                        .unwrap_or(u16::MAX),
                    texture_type: wr::M2TextureSource8::from_raw(texture_record.kind.raw() as u8),
                    replaceable_id: texture_record.kind.raw() as u8,
                });
            }
            let record = wr::M2Material {
                source_version: V::VERSION,
                render_flags: wr::M2RenderFlags16::from_bits_retain(material.flags.bits()),
                blending_mode: wr::M2BlendingMode16::from_raw(material.blending_mode.raw()),
                shader_id: batch.shader_id,
                priority_plane: i16::from(batch.priority_plane),
                material_layer: batch.material_layer,
                color_index: batch.color_index,
                batch_flags: wr::M2BatchFlags8::from_bits_retain(batch.flags.bits()),
                units: wem::InlineList(units),
            };
            let texture = record.units.0.first().map(|unit| u32::from(unit.texture));
            native_materials.push(record);
            mesh.material_slot = model.materials.len() as u32;
            model.materials.push(sm::Material {
                name: Cow::Owned(format!("material-{}", model.materials.len())),
                texture,
                blend: common_blend(material.blending_mode),
                flags: common_material_flags(material.flags),
            });
            model.meshes.push(mesh.clone());
        }
    }
    Ok(())
}

pub(super) fn export_geometry<D: Dialect>(
    model: &sm::Model<'_>,
    bundle: &mut Bundle<'static, D>,
    context: &mut Context,
) -> Result<(), Error>
where
    D::Layout: ExportLayout,
{
    let mut profiles: BTreeMap<u32, SkinProfile<'static, D::Layout>> = BTreeMap::new();
    let core = bundle.model.core_mut();
    let mut vertices = Vec::new();
    let mut materials = Vec::new();
    let mut bone_combos = Vec::new();
    let mut textures = Vec::new();
    let mut weights = Vec::new();
    let mut transforms = Vec::new();
    let mut uv_sets = Vec::new();
    for (mesh_index, mesh) in model.meshes.iter().enumerate() {
        let path = format!("meshes[{mesh_index}]");
        let count = mesh.positions.len();
        if count > usize::from(u16::MAX) {
            return Err(Error::new(
                &path,
                "mesh vertex count exceeds native section u16 limits",
            ));
        }
        if !mesh.colors.is_empty() || !mesh.tangents.is_empty() {
            context.unsupported(
                &path,
                "M2 vertices have no stored vertex color or tangent attribute",
            )?;
        }
        if mesh.uv_sets.len() > 2 {
            context.unsupported(
                format!("{path}.uv_sets"),
                "M2 vertices carry only two UV sets",
            )?;
        }
        let profile = profiles.entry(mesh.lod).or_default();
        let first = profile.vertices.len();
        let index_start = profile.indices.len();
        let core_start = vertices.len();
        if first + count > usize::from(u16::MAX) || core_start + count > usize::from(u16::MAX) {
            return Err(Error::new(
                &path,
                "combined geometry exceeds a u16 vertex lookup; split the target asset explicitly",
            ));
        }
        if mesh.triangles.len() * 3 > usize::from(u16::MAX) {
            return Err(Error::new(
                &path,
                "section triangle-entry count exceeds u16",
            ));
        }
        let mut palette = Vec::<u16>::new();
        let mut local_bones = Vec::new();
        for (index, position) in mesh.positions.iter().enumerate() {
            let mut vertex = Vertex {
                position: *position,
                normal: mesh.normals.get(index).copied().unwrap_or([0.; 3]),
                texture_coordinates: [
                    mesh.uv_sets
                        .first()
                        .and_then(|set| set.get(index))
                        .copied()
                        .unwrap_or([0.; 2]),
                    mesh.uv_sets
                        .get(1)
                        .and_then(|set| set.get(index))
                        .copied()
                        .unwrap_or([0.; 2]),
                ],
                ..Default::default()
            };
            let influences = mesh.influences.get(index).map(Vec::as_slice).unwrap_or(&[]);
            if influences.len() > 4 {
                return Err(Error::new(
                    format!("{path}.influences[{index}]"),
                    "M2 vertex carries at most four bone influences",
                ));
            }
            let sum: f32 = influences.iter().map(|v| v.weight).sum();
            if !influences.is_empty() && (!sum.is_finite() || sum <= 0.) {
                return Err(Error::new(
                    &path,
                    "skin influence sum must be positive and finite",
                ));
            }
            let mut fractions = Vec::new();
            let mut total = 0u32;
            let mut remapping = [0; 4];
            for (slot, influence) in influences.iter().enumerate() {
                if influence.weight < 0. || !influence.weight.is_finite() {
                    return Err(Error::new(
                        &path,
                        "skin weights must be nonnegative finite values",
                    ));
                }
                if influence.node as usize >= model.nodes.len() {
                    return Err(Error::new(
                        &path,
                        "skin influence references a missing node",
                    ));
                }
                vertex.bone_indices[slot] = u8::try_from(influence.node)
                    .map_err(|_| Error::new(&path, "M2 vertex bone index exceeds u8"))?;
                let scaled = influence.weight / sum * 255.;
                vertex.bone_weights[slot] = scaled.floor() as u8;
                total += u32::from(vertex.bone_weights[slot]);
                fractions.push((slot, scaled - scaled.floor()));
                let node = influence.node as u16;
                let palette_index = palette.iter().position(|n| *n == node).unwrap_or_else(|| {
                    palette.push(node);
                    palette.len() - 1
                });
                remapping[slot] = u8::try_from(palette_index)
                    .map_err(|_| Error::new(&path, "section bone palette exceeds 256 entries"))?;
            }
            fractions.sort_by(|a, b| b.1.total_cmp(&a.1));
            for (slot, _) in fractions.iter().take(255u32.saturating_sub(total) as usize) {
                vertex.bone_weights[*slot] = vertex.bone_weights[*slot].saturating_add(1);
            }
            let error = influences
                .iter()
                .enumerate()
                .map(|(slot, influence)| {
                    (influence.weight - f32::from(vertex.bone_weights[slot]) / 255.).abs()
                })
                .fold(0., f32::max);
            context.approximate(
                format!("{path}.influences[{index}]"),
                error,
                "skin weights quantized to bytes summing to 255",
            )?;
            vertices.push(vertex);
            local_bones.push(remapping);
        }
        let indices: Vec<u16> = mesh
            .triangles
            .iter()
            .flat_map(|triangle| triangle.iter())
            .map(|index| {
                if *index as usize >= count {
                    return Err(Error::new(&path, "triangle references a missing vertex"));
                }
                u16::try_from(first + *index as usize)
                    .map_err(|_| Error::new(&path, "profile index exceeds u16"))
            })
            .collect::<Result<_, _>>()?;
        let mut lookup: Vec<_> = profile.vertices.iter().collect();
        lookup.extend((core_start..core_start + count).map(|index| index as u16));
        profile.vertices = ZeroVec::alloc_from_slice(&lookup);
        let mut triangles: Vec<_> = profile.indices.iter().collect();
        triangles.extend(indices);
        profile.indices = ZeroVec::alloc_from_slice(&triangles);
        let mut bones: Vec<_> = profile.bones.iter().collect();
        bones.extend(local_bones);
        profile.bones = ZeroVec::alloc_from_slice(&bones);
        let palette_start = bone_combos.len();
        bone_combos.extend(palette);
        let section_index = profile.sections.len();
        profile.sections.push(SkinSection::<D::Layout> {
            id: u16::try_from(mesh.selection_group)
                .map_err(|_| Error::new(&path, "selection group exceeds u16"))?,
            level: u16::try_from(index_start >> 16)
                .map_err(|_| Error::new(&path, "triangle start exceeds native high word"))?,
            vertex_start: first as u16,
            vertex_count: count as u16,
            index_start: index_start as u16,
            index_count: (mesh.triangles.len() * 3) as u16,
            bone_count: (bone_combos.len() - palette_start) as u16,
            bone_combo_index: u16::try_from(palette_start)
                .map_err(|_| Error::new(&path, "bone-combination table exceeds u16"))?,
            bone_influences: mesh.influences.iter().map(Vec::len).max().unwrap_or(0) as u16,
            center_position: from_fn(|axis| {
                (mesh.bounds.minimum[axis] + mesh.bounds.maximum[axis]) * 0.5
            }),
            ..Default::default()
        });
        if mesh.material_slot == u32::MAX {
            continue;
        }
        let material = model
            .materials
            .get(mesh.material_slot as usize)
            .ok_or_else(|| Error::new(&path, "material slot references a missing material"))?;
        let supported = wr::MaterialFlags::UNLIT.bits() | wr::MaterialFlags::UNFOGGED.bits();
        if material.flags.bits() & !supported != 0 {
            context.unsupported(
                format!("{path}.material.flags"),
                "common material flags have no verified native M2 equivalent",
            )?;
        }
        if foreign_native_material(model, mesh.material_slot)? {
            context.unsupported(
                format!("{path}.material.native"),
                "foreign shader controls require an explicit lossy M2 material export",
            )?;
        }
        let (mut native_material, native_authoritative) =
            if let Some(native) = active_material(model, mesh.material_slot)? {
                native
            } else {
                (
                    wr::M2Material {
                        render_flags: native_material_flags(material.flags),
                        units: wem::InlineList(
                            material
                                .texture
                                .map(|texture| wr::M2TextureUnit {
                                    texture: texture as u16,
                                    uv_set: 0,
                                    transform: u16::MAX,
                                    weight: u16::MAX,
                                    ..Default::default()
                                })
                                .into_iter()
                                .collect(),
                        ),
                        ..Default::default()
                    },
                    false,
                )
            };
        if !native_authoritative {
            let projection = common_blend(BlendMode::from_raw(native_material.blending_mode.raw()));
            if material.blend != projection {
                native_material.blending_mode = native_blend(material.blend, context)?;
            }
            if let Some(texture) = material.texture {
                let texture = u16::try_from(texture)
                    .map_err(|_| Error::new("materials.texture", "texture index exceeds u16"))?;
                if let Some(unit) = native_material.units.0.first_mut() {
                    unit.texture = texture;
                } else {
                    native_material.units.0.push(wr::M2TextureUnit {
                        texture,
                        transform: u16::MAX,
                        weight: u16::MAX,
                        ..Default::default()
                    });
                }
            }
        }
        let material_index = u16::try_from(materials.len())
            .map_err(|_| Error::new("materials", "too many native materials"))?;
        materials.push(native::records::Material {
            flags: MaterialFlags::from_bits_retain(native_material.render_flags.bits()),
            blending_mode: BlendMode::from_raw(native_material.blending_mode.raw()),
        });
        let combo = u16::try_from(textures.len())
            .map_err(|_| Error::new("materials", "texture combinations exceed u16"))?;
        for unit in &native_material.units.0 {
            if usize::from(unit.texture) >= model.textures.len() {
                return Err(Error::new("materials.texture", "missing texture resource"));
            }
            textures.push(unit.texture);
            uv_sets.push(unit.uv_set);
            weights.push(unit.weight);
            transforms.push(unit.transform);
        }
        profile.batches.push(native::records::Batch {
            flags: BatchFlags::from_bits_retain(native_material.batch_flags.bits()),
            priority_plane: i8::try_from(native_material.priority_plane)
                .map_err(|_| Error::new("materials.priority", "M2 priority exceeds i8"))?,
            shader_id: native_material.shader_id,
            skin_section_index: u16::try_from(section_index)
                .map_err(|_| Error::new(&path, "too many sections"))?,
            color_index: native_material.color_index,
            material_index,
            material_layer: native_material.material_layer,
            texture_count: u16::try_from(native_material.units.0.len())
                .map_err(|_| Error::new("materials.units", "too many texture units"))?,
            texture_combo_index: combo,
            texture_coordinate_combo_index: combo,
            texture_weight_combo_index: combo,
            texture_transform_combo_index: combo,
            ..Default::default()
        });
    }
    core.vertices = ZeroVec::alloc_from_slice(&vertices);
    core.materials = ZeroVec::alloc_from_slice(&materials);
    core.bone_combos = ZeroVec::alloc_from_slice(&bone_combos);
    core.texture_combos = ZeroVec::alloc_from_slice(&textures);
    core.texture_weight_combos = ZeroVec::alloc_from_slice(&weights);
    core.texture_transform_combos = ZeroVec::alloc_from_slice(&transforms);
    core.texture_transform_bone_map = ZeroVec::alloc_from_slice(&uv_sets);
    D::Layout::install_profiles(bundle, profiles.into_values().collect());
    Ok(())
}
