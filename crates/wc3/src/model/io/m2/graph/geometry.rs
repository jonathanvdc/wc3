//! Borrowed section geometry and logical shader passes.
use super::*;
use crate::model::io::materialize;
use crate::model::materials::bindings::{Coordinates, ImageSelection};
use crate::model::materials::m2::FrameSampling;
use crate::model::materials::m2::{material_flags, rasterization};
use crate::model::materials::state::PassState;
use semantic::{Draw, Influence, M2Geometry, Mesh, MeshInstance, Skin, Topology};
use zerovec::ZeroVec;
#[path = "geometry_coordinates.rs"]
mod coordinates;
pub(super) fn import<'a, 'data: 'a, D: Dialect>(
    source: &'a Bundle<'data, D>,
    nodes: &[semantic::NodeId],
    textures: &[semantic::Texture],
    clips: &[semantic::ClipId],
    clocks: &[semantic::ClockId],
    builder: &mut GraphBuilder<'a>,
    context: &mut Context,
) -> Result<(), Error>
where
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
    <D::Layout as native::records::SkinSectionLayout>::Tail: SectionSorting,
{
    let core = source.model.core();
    let mut profiles: Vec<_> = core
        .embedded_skins()
        .iter()
        .enumerate()
        .map(|(i, _)| (i, false))
        .collect();
    for dependency in source.model.dependencies(&source.key) {
        if let native::DependencyKind::Skin { index, lod } = dependency.kind {
            if !profiles.contains(&(index, lod)) {
                profiles.push((index, lod))
            }
        }
    }
    let skin = if nodes.is_empty() {
        None
    } else {
        Some(builder.add_skin(Skin::identity(nodes.to_vec()))?)
    };
    let tables = sequence_tables(source, &source.key)?;
    for (index, lod) in profiles {
        let source_profile = source.profile(index, lod).map_err(failure)?;
        let profile = match source_profile {
            ProfileRef::Embedded(p) => p,
            ProfileRef::Skin(s) => &s.profile,
            ProfileRef::Geometry(_) => {
                context.unsupported(
                    "m2.geometry",
                    "geometry companion has no verified logical schema",
                )?;
                continue;
            }
        };
        let base = if core.flags.global_skin_vertex_offsets() {
            profile.lod_vertex_base as usize
        } else {
            0
        };
        for (section_index, section) in profile.sections.iter().enumerate() {
            let first = section.vertex_start as usize;
            let count = section.vertex_count as usize;
            let start = section_start(section);
            let faces = section.index_count as usize / 3;
            if first + count > profile.vertices.len()
                || start + faces * 3 > profile.indices.len()
                || !section.index_count.is_multiple_of(3)
            {
                return Err(Error::new(
                    "m2.section",
                    "section window exceeds geometry arrays",
                ));
            }
            for i in first..first + count {
                if core
                    .vertices
                    .get(base + profile.vertices.get(i).unwrap() as usize)
                    .is_none()
                {
                    return Err(Error::new("m2.section", "missing model vertex"));
                }
            }
            let model_vertices = &core.vertices;
            let section_vertices = &profile.vertices;
            let section_indices = &profile.indices;
            let vertex = move |i| {
                model_vertices
                    .get(base + section_vertices.get(first + i).unwrap() as usize)
                    .unwrap()
            };
            let geometry = M2Geometry::new(
                materialize(count, move |i| vertex(i).position),
                materialize(count, move |i| vertex(i).normal),
                std::array::from_fn(|set| {
                    materialize(count, move |i| vertex(i).texture_coordinates[set])
                }),
                materialize(faces, move |i| {
                    std::array::from_fn(|corner| {
                        u32::from(section_indices.get(start + i * 3 + corner).unwrap())
                            .wrapping_sub(first as u32)
                    })
                }),
                materialize(count, move |i| {
                    let v = vertex(i);
                    std::array::from_fn(|slot| Influence {
                        joint: u32::from(v.bone_indices[slot]),
                        weight: f32::from(v.bone_weights[slot]) / 255.,
                    })
                }),
            )?;
            let mut mesh = Mesh::new(geometry);
            mesh.name = Cow::Owned(format!("skin-{index}-section-{section_index}"));
            mesh.bounds = bounds(core.bounding);
            let mesh = builder.add_mesh(mesh.with_controls(profile::GeometryControls {
                center_bone: if nodes.is_empty() && section.center_bone_index == 0 {
                    None
                } else {
                    Some(
                        *nodes
                            .get(section.center_bone_index as usize)
                            .ok_or_else(|| {
                                Error::new(
                                    "m2.mesh.center_bone",
                                    "sorting center selects an absent bone",
                                )
                            })?,
                    )
                },
                center: section.center_position,
                sorting: section.sort_center.logical(),
            }))?;
            let node = builder.add_named_node(format!("mesh-{index}-{section_index}"))?;
            let mut draws = Vec::new();
            let mut draw_materials = Vec::new();
            let mut batches: Vec<_> = profile
                .batches
                .iter()
                .filter(|batch| batch.skin_section_index as usize == section_index)
                .collect();
            batches.sort_by_key(|batch| batch.material_layer);
            let mut groups: Vec<(u16, Vec<_>)> = Vec::new();
            for batch in batches {
                if let Some((_, steps)) = groups
                    .last_mut()
                    .filter(|(word, _)| *word == batch.geoset_index)
                {
                    steps.push(batch);
                } else {
                    groups.push((batch.geoset_index, vec![batch]));
                }
            }
            let subset = builder.add_primary_subset(mesh, 0, faces as u32)?;
            for (auxiliary_section, batches) in groups {
                let material = builder.add_material(semantic::Material {
                    name: Cow::Borrowed(""),
                    surface: profile::Material::default(),
                })?;
                let mut steps = Vec::new();
                for batch in batches.into_iter() {
                    let native_material = core
                        .materials
                        .get(batch.material_index as usize)
                        .ok_or_else(|| Error::new("m2.material", "missing render state"))?;
                    let declaration = profile::MaterialDeclaration {
                        batch_flags: profile::values::BatchFlags::from_bits_retain(
                            batch.flags.bits(),
                        ),
                        color: builder.add_value([1.; 3]),
                        opacity: builder.add_value(1.),
                        state: PassState {
                            blend: profile::values::BlendMode::from_raw(
                                native_material.blending_mode.raw(),
                            ),
                            rasterization: rasterization(
                                profile::values::MaterialFlags::from_bits_retain(
                                    native_material.flags.bits(),
                                ),
                            ),
                            ordering: batch.priority_plane,
                        },
                        ..builder.construct()
                    };
                    let mut textures_for_step = Vec::new();
                    let routes = coordinates::routes(batch.shader_id, batch.texture_count as usize);
                    for slot in 0..batch.texture_count as usize {
                        let texture = core
                            .texture_combos
                            .get(batch.texture_combo_index as usize + slot)
                            .ok_or_else(|| {
                                Error::new("m2.material.texture", "missing texture combination")
                            })? as usize;
                        let record = core
                            .textures
                            .get(texture)
                            .ok_or_else(|| Error::new("m2.material.texture", "missing texture"))?;
                        textures_for_step.push(profile::TextureUnit {
                            image: ImageSelection {
                                source: profile::values::TextureSource::from_raw(record.kind.raw()),
                                texture: Some(*textures.get(texture).ok_or_else(|| {
                                    Error::new("m2.material.texture", "missing logical texture")
                                })?),
                            },
                            coordinates: Coordinates {
                                source: routes.map_or_else(
                                    || {
                                        core.texture_transform_bone_map
                                            .get(
                                                batch.texture_coordinate_combo_index as usize
                                                    + slot,
                                            )
                                            .unwrap_or(0)
                                            as i16
                                    },
                                    |routes| routes[slot].source,
                                ),
                                transform: builder.construct(),
                            },
                            contribution: builder.add_value(1.),
                            sampling: FrameSampling {
                                frame: builder.add_value(0),
                            },
                        });
                    }
                    let mut surface = profile::MaterialStep::from_native(
                        batch.shader_id,
                        declaration,
                        textures_for_step,
                        Default::default(),
                    );
                    if batch.color_index >= 0 {
                        if let Some(color) = core.colors.get(batch.color_index as usize) {
                            let rgb = surface.declaration().color;
                            let alpha = surface.declaration().opacity;
                            for (seq, clip) in animation::sequences(clips) {
                                let (start, _) = tables
                                    .sequences
                                    .get(seq)
                                    .map_or((0, 0), |s| s.timing.window());
                                if let Some((c, g)) =
                                    curve(source, &color.color, &source.key, seq, start, |v| v)?
                                {
                                    animation::insert(
                                        builder,
                                        clip,
                                        DynamicTrack::Vector3(bind(c, g, rgb, clocks)?),
                                    )?
                                }
                                if let Some((c, g)) =
                                    curve(source, &color.alpha, &source.key, seq, start, |v| {
                                        f32::from(v) / 32767.
                                    })?
                                {
                                    animation::insert(
                                        builder,
                                        clip,
                                        DynamicTrack::Scalar(bind(c, g, alpha, clocks)?),
                                    )?
                                }
                            }
                        }
                    }
                    for (slot, unit) in surface.inputs_mut().into_iter().enumerate() {
                        if let Some(weight) = core
                            .texture_weight_combos
                            .get(batch.texture_weight_combo_index as usize + slot)
                            .and_then(|i| core.texture_weights.get(i as usize))
                        {
                            let prop = unit.contribution;
                            for (seq, clip) in animation::sequences(clips) {
                                let (start, _) = tables
                                    .sequences
                                    .get(seq)
                                    .map_or((0, 0), |s| s.timing.window());
                                if let Some((c, g)) =
                                    curve(source, &weight.weight, &source.key, seq, start, |v| {
                                        f32::from(v) / 32767.
                                    })?
                                {
                                    animation::insert(
                                        builder,
                                        clip,
                                        DynamicTrack::Scalar(bind(c, g, prop, clocks)?),
                                    )?
                                }
                            }
                        }
                        let transform_slot = if let Some(routes) = routes {
                            let route = routes[slot];
                            if route.transform.is_none()
                                || routes[..slot]
                                    .iter()
                                    .any(|previous| previous.transform == route.transform)
                            {
                                continue;
                            }
                            route.transform.unwrap()
                        } else {
                            slot
                        };
                        if let Some(transform) = core
                            .texture_transform_combos
                            .get(batch.texture_transform_combo_index as usize + transform_slot)
                            .and_then(|i| core.texture_transforms.get(i as usize))
                        {
                            let tr = unit.coordinates.transform.translation;
                            let ro = unit.coordinates.transform.rotation;
                            let sc = unit.coordinates.transform.scale;
                            for (seq, clip) in animation::sequences(clips) {
                                let (start, _) = tables
                                    .sequences
                                    .get(seq)
                                    .map_or((0, 0), |s| s.timing.window());
                                for (prop, track) in
                                    [(tr, &transform.translation), (sc, &transform.scaling)]
                                {
                                    if let Some((c, g)) =
                                        curve(source, track, &source.key, seq, start, |v| v)?
                                    {
                                        animation::insert(
                                            builder,
                                            clip,
                                            DynamicTrack::Vector3(bind(c, g, prop, clocks)?),
                                        )?
                                    }
                                }
                                if let Some((c, g)) = curve(
                                    source,
                                    &transform.rotation,
                                    &source.key,
                                    seq,
                                    start,
                                    semantic::Quaternion,
                                )? {
                                    animation::insert(
                                        builder,
                                        clip,
                                        DynamicTrack::Rotation(bind(c, g, ro, clocks)?),
                                    )?
                                }
                            }
                        }
                    }
                    if let Some(routes) = routes {
                        for slot in 0..routes.len() {
                            if let Some(owner) = routes[..slot]
                                .iter()
                                .position(|route| route.transform == routes[slot].transform)
                                .filter(|_| routes[slot].transform.is_some())
                            {
                                let transform =
                                    surface.inputs()[owner].coordinates.transform.clone();
                                surface.inputs_mut()[slot].coordinates.transform = transform;
                            }
                        }
                    }
                    steps.push(surface);
                }
                builder.set_material_surface(material, profile::Material { steps })?;
                draw_materials.push(material);
                draws.push(Draw::new(
                    subset,
                    Some(profile::DrawControls {
                        auxiliary_section,
                        shadow: None,
                    }),
                ));
            }
            builder.add_instance_with_materials(
                MeshInstance {
                    controls: NoControls,
                    node,
                    mesh,
                    skin,
                    draws,
                    lod: Some(index as u32 + u32::from(lod)),
                    selection_group: section.id as u32,
                },
                draw_materials,
            )?;
        }
    }
    Ok(())
}
pub(super) fn export<D: Dialect>(
    model: &Graph<'_>,
    nodes: &[semantic::NodeId],
    textures: &[semantic::Texture],
    controls: &BTreeMap<(semantic::MaterialId, usize), super::materials::Controls>,
    look: Option<semantic::LookId>,
    bundle: &mut Bundle<'static, D>,
    context: &mut Context,
) -> Result<(), Error>
where
    D::Layout: ExportLayout,
    <D::Layout as native::records::SkinSectionLayout>::Tail: SectionSorting,
{
    let mut profiles: BTreeMap<u32, SkinProfile<'static, D::Layout>> = BTreeMap::new();
    let mut vertices = Vec::new();
    let mut materials = Vec::new();
    let mut bone_combos = Vec::new();
    let mut texture_combos = Vec::new();
    let mut weights = Vec::new();
    let mut transforms = Vec::new();
    let mut uv_sets = Vec::new();
    for (instance_id, instance) in model.instances() {
        let mesh = model
            .mesh(instance.mesh)
            .ok_or_else(|| Error::new("m2.mesh", "missing geometry"))?;
        let placement = model
            .node(instance.node)
            .ok_or_else(|| Error::new("m2.mesh", "missing placement"))?;
        if placement.transform.defaults(model.values()).translation != [0.; 3]
            || placement.transform.defaults(model.values()).rotation
                != semantic::Quaternion::default()
            || placement.transform.defaults(model.values()).scale != [1.; 3]
        {
            context.unsupported(
                "m2.mesh.placement",
                "a nonidentity mesh placement requires vertex and normal rebasing",
            )?
        }
        let triangles = if let Topology::Triangles(t) = mesh.topology() {
            t
        } else {
            return Err(Error::new(
                "m2.mesh.topology",
                "M2 requires triangle geometry",
            ));
        };
        let count = mesh.positions().len();
        if count > u16::MAX as usize || triangles.len() * 3 > u16::MAX as usize {
            return Err(Error::new("m2.mesh", "section exceeds u16 limits"));
        }
        let profile = profiles.entry(instance.lod.unwrap_or(0)).or_default();
        let first = profile.vertices.len();
        let core_start = vertices.len();
        let index_start = profile.indices.len();
        let mut palette = Vec::new();
        let mut bone_maps = Vec::new();
        let skin = instance.skin.and_then(|id| model.skin(id));
        if let Some(skin) = skin {
            for joint in &skin.joints {
                if model
                    .node(*joint)
                    .is_none_or(|node| node.controls.is_none())
                    || !nodes.contains(joint)
                {
                    return Err(Error::new(
                        "m2.mesh.skin",
                        "skin palette requires exported declared bones",
                    ));
                }
            }
        }
        for (i, position) in mesh.positions().iter().enumerate() {
            let mut v = Vertex {
                position,
                normal: mesh.normals().and_then(|n| n.get(i)).unwrap_or([0.; 3]),
                texture_coordinates: std::array::from_fn(|set| {
                    mesh.texture_coordinates()
                        .get(set)
                        .and_then(|uv| uv.get(i))
                        .unwrap_or([0.; 2])
                }),
                ..Default::default()
            };
            let row = mesh.influences().joint_weights(i).unwrap_or_default();
            if row.len() > 4 {
                return Err(Error::new(
                    "m2.mesh.influences",
                    "M2 supports at most four influences",
                ));
            }
            let mut mapping = [0; 4];
            let mut remainder = Vec::new();
            let mut total = 0u32;
            for (slot, influence) in row.iter().enumerate() {
                if influence.weight == 0. {
                    continue;
                }
                let joint_id = skin
                    .and_then(|s| s.joints.get(influence.joint as usize))
                    .ok_or_else(|| Error::new("m2.mesh.skin", "missing authored joint"))?;
                if model
                    .node(*joint_id)
                    .is_none_or(|node| node.controls.is_none())
                {
                    return Err(Error::new(
                        "m2.mesh.skin",
                        "influence requires a declared bone",
                    ));
                }
                let joint = nodes
                    .iter()
                    .position(|node| node == joint_id)
                    .ok_or_else(|| Error::new("m2.mesh.skin", "missing exported joint"))?;
                v.bone_indices[slot] = u8::try_from(joint)
                    .map_err(|_| Error::new("m2.mesh.skin", "joint exceeds byte width"))?;
                let scaled = influence.weight * 255.;
                v.bone_weights[slot] = scaled.floor() as u8;
                total += u32::from(v.bone_weights[slot]);
                remainder.push((slot, scaled - scaled.floor()));
                let index = palette
                    .iter()
                    .position(|b| *b == joint as u16)
                    .unwrap_or_else(|| {
                        palette.push(joint as u16);
                        palette.len() - 1
                    });
                mapping[slot] = u8::try_from(index)
                    .map_err(|_| Error::new("m2.mesh.skin", "palette exceeds 256 bones"))?
            }
            remainder.sort_by(|a, b| b.1.total_cmp(&a.1));
            for (slot, _) in remainder.iter().take(255u32.saturating_sub(total) as usize) {
                v.bone_weights[*slot] += 1
            }
            for (slot, influence) in row.iter().enumerate() {
                if influence.weight == 0. {
                    continue;
                }
                context.approximate(
                    "m2.mesh.skin",
                    (influence.weight - f32::from(v.bone_weights[slot]) / 255.).abs(),
                    "weights quantized to bytes summing to 255",
                )?
            }
            vertices.push(v);
            bone_maps.push(mapping)
        }
        let mut lookup: Vec<_> = profile.vertices.iter().collect();
        for i in core_start..core_start + count {
            lookup.push(
                u16::try_from(i).map_err(|_| {
                    Error::new("m2.mesh.vertices", "global vertex index exceeds u16")
                })?,
            )
        }
        profile.vertices = ZeroVec::alloc_from_slice(&lookup);
        let mut indices: Vec<_> = profile.indices.iter().collect();
        for t in triangles.iter() {
            for i in t {
                indices.push(
                    u16::try_from(first + i as usize)
                        .map_err(|_| Error::new("m2.mesh.indices", "profile index exceeds u16"))?,
                )
            }
        }
        profile.indices = ZeroVec::alloc_from_slice(&indices);
        let mut remapping: Vec<_> = profile.bones.iter().collect();
        remapping.extend(bone_maps);
        profile.bones = ZeroVec::alloc_from_slice(&remapping);
        let palette_start = bone_combos.len();
        bone_combos.extend(palette);
        let section_index = profile.sections.len();
        let geoset = mesh.controls.as_ref();
        profile.sections.push(SkinSection {
            id: u16::try_from(instance.selection_group)
                .map_err(|_| Error::new("m2.mesh.selection", "selection group exceeds u16"))?,
            level: (index_start >> 16) as u16,
            vertex_start: first as u16,
            vertex_count: count as u16,
            index_start: index_start as u16,
            index_count: (triangles.len() * 3) as u16,
            bone_count: (bone_combos.len() - palette_start) as u16,
            bone_combo_index: palette_start as u16,
            bone_influences: mesh
                .influences()
                .iter()
                .map(|r| r.weights().filter(|weight| *weight != 0.).count())
                .max()
                .unwrap_or(0) as u16,
            center_position: geoset.map_or_else(
                || std::array::from_fn(|a| (mesh.bounds.minimum[a] + mesh.bounds.maximum[a]) * 0.5),
                |g| g.center,
            ),
            center_bone_index: geoset
                .and_then(|g| g.center_bone)
                .map(|id| {
                    if model.node(id).is_none_or(|node| node.controls.is_none()) {
                        return Err(Error::new(
                            "m2.mesh.center_bone",
                            "sorting center requires a declared bone",
                        ));
                    }
                    let index = nodes.iter().position(|node| *node == id).ok_or_else(|| {
                        Error::new("m2.mesh.center_bone", "sorting-center bone is not exported")
                    })?;
                    u16::try_from(index).map_err(|_| {
                        Error::new(
                            "m2.mesh.center_bone",
                            "sorting-center index exceeds native width",
                        )
                    })
                })
                .transpose()?
                .unwrap_or(0),
            sort_center: SectionSorting::make(geoset.and_then(|g| g.sorting), context)?,
        });
        let mut section_layer = 0usize;
        for draw in &instance.draws {
            let draw = mesh.resolve_draw(draw)?;
            let material_id = model.material_for_draw(instance_id, draw.id, look)?;
            if draw.first != 0 || draw.count as usize != triangles.len() {
                context.unsupported(
                    "m2.mesh.draw",
                    "partial draw ranges require splitting the native section",
                )?
            }
            let material = model
                .material(material_id)
                .ok_or_else(|| Error::new("m2.material", "missing surface"))?;
            for (step_index, surface) in material.surface.steps.iter().enumerate() {
                let declaration = surface.declaration();
                let inputs = surface.inputs();
                let shader = surface.native_shader();
                let routes = coordinates::routes(shader, inputs.len());
                if let Some(routes) = routes {
                    for (slot, route) in routes.iter().enumerate() {
                        if inputs[slot].coordinates.source != route.source {
                            context.unsupported(
                                "m2.material.coordinates",
                                "shader coordinate route differs from the native programme",
                            )?;
                        }
                        let conflicting_owner = routes[..slot]
                            .iter()
                            .position(|previous| previous.transform == route.transform)
                            .filter(|owner| {
                                route.transform.is_some()
                                    && inputs[slot].coordinates.transform
                                        != inputs[*owner].coordinates.transform
                            });
                        if conflicting_owner.is_some() {
                            context.unsupported("m2.material.transform", "native shader repeats one transform; independent input transforms require changing the shader")?;
                        }
                    }
                }
                let material_index = materials.len();
                materials.push(native::records::Material {
                    flags: MaterialFlags::from_bits_retain(
                        material_flags(&declaration.state.rasterization).bits(),
                    ),
                    blending_mode: BlendMode::from_raw(declaration.state.blend.raw()),
                });
                let combo = texture_combos.len();
                for (slot, unit) in inputs.iter().enumerate() {
                    let id = unit.image.texture.ok_or_else(|| {
                        Error::new("m2.material.texture", "shader unit has no texture resource")
                    })?;
                    texture_combos.push(
                        u16::try_from(texture_index(model, textures, id).ok_or_else(|| {
                            Error::new("m2.material.texture", "missing resource")
                        })?)
                        .map_err(|_| Error::new("m2.material.texture", "index exceeds u16"))?,
                    );
                    uv_sets.push(unit.coordinates.source as u16);
                    let indices = controls
                        .get(&(material_id, step_index))
                        .and_then(|c| c.units.get(slot))
                        .copied()
                        .unwrap_or((u16::MAX, u16::MAX));
                    weights.push(indices.0);
                    let transform = routes.map_or(indices.1, |routes| {
                        routes
                            .iter()
                            .position(|route| route.transform == Some(slot))
                            .and_then(|owner| {
                                controls
                                    .get(&(material_id, step_index))
                                    .and_then(|c| c.units.get(owner))
                            })
                            .map_or(u16::MAX, |indices| indices.1)
                    });
                    transforms.push(transform)
                }
                profile.batches.push(native::records::Batch {
                    flags: BatchFlags::from_bits_retain(declaration.batch_flags.bits()),
                    priority_plane: declaration.state.ordering,
                    shader_id: shader,
                    skin_section_index: section_index as u16,
                    color_index: controls
                        .get(&(material_id, step_index))
                        .map_or(-1, |c| c.color),
                    material_index: material_index as u16,
                    material_layer: u16::try_from(section_layer)
                        .map_err(|_| Error::new("m2.material.steps", "too many material steps"))?,
                    geoset_index: draw.controls.as_ref().map_or(0, |v| v.auxiliary_section),
                    texture_count: inputs.len() as u16,
                    texture_combo_index: combo as u16,
                    texture_coordinate_combo_index: combo as u16,
                    texture_weight_combo_index: combo as u16,
                    texture_transform_combo_index: combo as u16,
                });
                section_layer += 1;
            }
        }
    }
    let core = bundle.model.core_mut();
    core.vertices = ZeroVec::alloc_from_slice(&vertices);
    core.materials = ZeroVec::alloc_from_slice(&materials);
    core.bone_combos = ZeroVec::alloc_from_slice(&bone_combos);
    core.texture_combos = ZeroVec::alloc_from_slice(&texture_combos);
    core.texture_weight_combos = ZeroVec::alloc_from_slice(&weights);
    core.texture_transform_combos = ZeroVec::alloc_from_slice(&transforms);
    core.texture_transform_bone_map = ZeroVec::alloc_from_slice(&uv_sets);
    D::Layout::install_profiles(bundle, profiles.into_values().collect());
    Ok(())
}
