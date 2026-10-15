//! Native static geometry rebuilt from independent authored streams.
use crate::model::conversion::{Context, Error};
use crate::model::semantic::profiles::wmo::Lighting;
use crate::model::semantic::profiles::Wmo;
use crate::model::semantic::MaterialId;
use crate::model::wmo::group::GroupChunk;
use crate::model::wmo::records::{BatchMetadata, HeaderTailMetadata, TriangleFlags2};
use crate::model::wmo::{Dialect, Group};
pub(in crate::model::conversion::wmo::graph) fn write<'a, D: Dialect>(
    model: &crate::model::semantic::Model<'_, Wmo>,
    instance_id: crate::model::semantic::InstanceId,
    lod: u32,
    names: (u32, u32),
    materials: &std::collections::BTreeMap<MaterialId, u16>,
    look: Option<crate::model::semantic::LookId>,
    context: &mut Context,
) -> Result<Group<'a, D>, Error> {
    use crate::model::semantic::Topology;
    use crate::model::wmo::records::{Batch, BatchFlags, GroupFlags2, Triangle2};
    let instance = model
        .instance(instance_id)
        .ok_or_else(|| Error::new("world.instance", "group placement does not exist"))?;
    let mesh = model
        .mesh(instance.mesh)
        .expect("validated geometry handle");
    if !mesh.geometry.lightmap_coordinates().is_empty()
        || mesh
            .geometry
            .triangle_controls()
            .iter()
            .any(|face| face.lightmap_selector.is_some() || face.alignment != 0)
        || instance.draws.iter().any(|draw| {
            draw.controls.as_ref().is_some_and(|controls| {
                controls.lightmap_selector.is_some() || controls.alignment != 0
            })
        })
    {
        context.unsupported("world.group.baked_light", "selected modern group layout cannot encode historical baked-light coordinates or selectors")?;
    }
    let region = &instance.controls;
    if !region.lightmap_images.is_empty()
        || !region.lightmap_rectangles.is_empty()
        || region.auxiliary_geometry.is_some()
        || region.interior_windows.is_some()
        || region.exterior_windows.is_some()
    {
        context.unsupported("world.group.historical_lighting", "selected modern group layout cannot encode historical baked-light resources or lighting windows")?;
    }
    let node = model.node(instance.node).expect("validated placement node");
    if node.transform != crate::model::semantic::Transform::default() || node.parent.is_some() {
        context.unsupported(
            "world.group.transform",
            "group transforms need static geometry baking before native encoding",
        )?;
    }
    let mut output = Group::<D>::new();
    let header = output.header_mut();
    header.name_offset = names.0;
    header.descriptive_name_offset = names.1;
    header.bounds_minimum = mesh.bounds.minimum;
    header.bounds_maximum = mesh.bounds.maximum;
    header.unique_id = region.area_id;
    header.reserved_batch_count = region.batch_control;
    if region.supplemental_controls.len() > 1
        || region
            .supplemental_controls
            .first()
            .is_some_and(|value| !header.tail.set_control(*value))
    {
        context.unsupported(
            "world.group.controls",
            "selected header has no matching supplemental region controls",
        )?;
    }
    header.liquid_type =
        super::super::liquids::category(region, model.metadata().options.database_liquids)?;
    header.flags2 = GroupFlags2(region.options.supplemental.bits());
    let mut flags = super::super::group_flags::write(region.options);
    flags.set_lod(lod != 0);
    flags.set_has_vertex_colors(!mesh.geometry.color_sets().is_empty());
    flags.set_has_liquid(!region.liquids.is_empty() || region.implicit_liquid.is_some());
    header.flags = flags;
    let resolve = |id: Option<crate::model::semantic::profiles::wmo::GroupId>| {
        id.map_or(Ok(-1), |id| {
            i16::try_from(id.index).map_err(|_| {
                Error::new(
                    "world.group.split",
                    "group index exceeds split-header capacity",
                )
            })
        })
    };
    if !header.tail.set_split_links(
        resolve(region.parent_or_first_child)?,
        resolve(region.next_child)?,
    ) {
        context.unsupported(
            "world.group.split",
            "selected group-header layout has no split links",
        )?;
    }
    let mut spherical_fogs = Vec::new();
    let mut box_fogs = Vec::new();
    for id in &region.fogs {
        let world = model.system(id.system).expect("validated fog system");
        let fog = &world.fogs[id.index as usize];
        let is_box = matches!(
            fog.volume,
            crate::model::semantic::profiles::wmo::Volume::Planes { .. }
        );
        let index = world.fogs[..id.index as usize]
            .iter()
            .filter(|fog| {
                matches!(
                    fog.volume,
                    crate::model::semantic::profiles::wmo::Volume::Planes { .. }
                ) == is_box
            })
            .count();
        let index = u16::try_from(index)
            .map_err(|_| Error::new("world.fog", "fog identity exceeds native capacity"))?;
        if is_box {
            box_fogs.push(index)
        } else {
            spherical_fogs.push(index)
        }
    }
    header.fog_indices = [u8::MAX; 4];
    for (slot, index) in spherical_fogs.iter().take(4).enumerate() {
        header.fog_indices[slot] = u8::try_from(*index).unwrap_or(u8::MAX);
    }
    append_group(
        &mut output,
        GroupChunk::Positions(super::encode::array(&mesh.positions().to_vec())?),
    )?;
    if let Some(normals) = mesh.normals() {
        append_group(
            &mut output,
            GroupChunk::Normals(super::encode::array(&normals.to_vec())?),
        )?;
    }
    for set in mesh.texture_coordinates() {
        append_group(
            &mut output,
            GroupChunk::TextureCoordinates(super::encode::array(&set.to_vec())?),
        )?;
    }
    for set in mesh.geometry.color_sets() {
        let colors = set
            .iter()
            .map(|rgba| {
                let mut values = [0u8; 4];
                for (target, source) in values.iter_mut().zip(rgba) {
                    let rounded = (source.clamp(0., 1.) * 255.).round() as u8;
                    let error = (source - f32::from(rounded) / 255.).abs();
                    if error != 0. {
                        context.approximate(
                            "world.colors",
                            error,
                            "native vertex colors use eight-bit normalized components",
                        )?;
                    }
                    *target = rounded;
                }
                Ok([values[2], values[1], values[0], values[3]])
            })
            .collect::<Result<Vec<_>, Error>>()?;
        append_group(
            &mut output,
            GroupChunk::VertexColors(super::encode::array(&colors)?),
        )?;
    }
    let Topology::Triangles(triangles) = mesh.topology() else {
        return Err(Error::new(
            "world.topology",
            "world geometry requires triangle connectivity",
        ));
    };
    let indices = triangles.iter().flatten().collect::<Vec<_>>();
    if indices.iter().all(|index| *index <= u16::MAX.into()) {
        append_group(
            &mut output,
            GroupChunk::Indices(super::encode::array(
                &indices
                    .iter()
                    .map(|index| *index as u16)
                    .collect::<Vec<_>>(),
            )?),
        )?;
    } else {
        append_group(
            &mut output,
            GroupChunk::Indices32(super::encode::array(&indices)?),
        )?;
    }
    let mut faces = Vec::new();
    for control in mesh.geometry.triangle_controls().iter() {
        let mut flags = TriangleFlags2(control.unclassified.bits());
        flags.set_transition(control.transition);
        flags.set_no_camera_collision(control.no_camera_collision);
        flags.set_detail(control.detail);
        flags.set_collision(control.collision);
        flags.set_hint(control.hint);
        flags.set_render(control.render);
        flags.set_cull_objects(control.cull_objects);
        flags.set_collide_hit(control.collide_hit);
        flags.set_ground_type_entry(control.ground_type_selector.is_some());
        faces.push(Triangle2 {
            flags,
            material: control
                .ground_type_selector
                .unwrap_or_else(|| control.material.map_or(u16::MAX, |id| materials[&id])),
        });
    }
    if faces.is_empty() {
        faces.resize(triangles.len(), Triangle2::default());
        for draw in &instance.draws {
            let draw = mesh.resolve_draw(draw)?;
            let material = materials[&model.material_for_draw(instance_id, draw.id, look)?];
            for face in &mut faces[draw.first as usize..(draw.first + draw.count) as usize] {
                face.flags.set_render(true);
                face.material = material;
            }
        }
    }
    if faces
        .iter()
        .all(|face| face.flags.bits() <= 255 && (face.material <= 254 || face.material == u16::MAX))
    {
        let rows = faces
            .iter()
            .map(|face| crate::model::wmo::records::Triangle {
                flags: crate::model::wmo::records::TriangleFlags(face.flags.bits() as u8),
                material: if face.material == u16::MAX {
                    u8::MAX
                } else {
                    face.material as u8
                },
            })
            .collect::<Vec<_>>();
        append_group(
            &mut output,
            GroupChunk::Triangles(super::encode::array(&rows)?),
        )?;
    } else {
        append_group(
            &mut output,
            GroupChunk::Triangles2(super::encode::array(&faces)?),
        )?;
    }
    let query_faces = mesh
        .geometry
        .triangle_controls()
        .iter()
        .enumerate()
        .filter_map(|(index, face)| face.spatial_queries.then_some(index))
        .map(|index| {
            u16::try_from(index).map_err(|_| {
                Error::new(
                    "world.query",
                    "spatial-query face index exceeds native capacity",
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    output
        .header_mut()
        .flags
        .set_has_bsp(!query_faces.is_empty());
    if !query_faces.is_empty() {
        let mut flags = crate::model::wmo::records::BspFlags::default();
        flags.set_leaf(true);
        let node = crate::model::wmo::records::BspNode {
            flags,
            negative_child: -1,
            positive_child: -1,
            face_count: u16::try_from(query_faces.len()).map_err(|_| {
                Error::new("world.query", "spatial-query leaf exceeds face capacity")
            })?,
            first_face: 0,
            plane_distance: 0.,
        };
        append_group(
            &mut output,
            GroupChunk::BspNodes(super::encode::array(&[node])?),
        )?;
        append_group(
            &mut output,
            GroupChunk::BspFaces(super::encode::array(&query_faces)?),
        )?;
    }
    let mut batches = Vec::new();
    let mut strip_windows = Vec::new();
    let mut tangent_starts = Vec::new();
    let mut shadow_batches = Vec::new();
    for draw in &instance.draws {
        let draw = mesh.resolve_draw(draw)?;
        let material = materials[&model.material_for_draw(instance_id, draw.id, look)?];
        if let Some(crate::model::semantic::profiles::wmo::Pass::Shadow {
            controls,
            supplemental,
        }) = draw.controls.as_ref().map(|value| &value.pass)
        {
            let mut remaining = draw.count;
            let mut first = draw.first;
            while remaining != 0 {
                let count = remaining.min(i16::MAX as u32 / 3);
                let mut flags = BatchFlags(
                    draw.controls
                        .as_ref()
                        .expect("shadow controls")
                        .unclassified
                        .bits(),
                );
                flags.set_large_material(material > u16::from(u8::MAX));
                shadow_batches.push(crate::model::wmo::records::ShadowBatch {
                    reserved_prefix: std::array::from_fn(|index| {
                        controls[index / 2].to_le_bytes()[index % 2]
                    }),
                    reserved_middle: std::array::from_fn(|index| {
                        supplemental[index / 2].to_le_bytes()[index % 2]
                    }),
                    start: i32::try_from(u64::from(first) * 3).map_err(|_| {
                        Error::new("world.shadow", "shadow start exceeds native capacity")
                    })?,
                    count: (count * 3) as i16,
                    large_material: material,
                    material: material as u8,
                    flags,
                });
                first += count;
                remaining -= count;
            }
            continue;
        }
        let mut first = draw.first as usize;
        let mut remaining = draw.count as usize;
        let mut first_record = true;
        while remaining != 0 || first_record {
            first_record = false;
            let count = remaining.min(usize::from(u16::MAX) / 3);
            let vertices = triangles
                .iter()
                .skip(first)
                .take(count)
                .flatten()
                .collect::<Vec<_>>();
            let minimum = vertices.iter().copied().min().unwrap_or(0);
            let maximum = vertices.iter().copied().max().unwrap_or(0);
            let mut batch = Batch::<D> {
                start_index: (first * 3) as u32,
                index_count: (count * 3) as u16,
                minimum_vertex: minimum as u16,
                maximum_vertex: maximum as u16,
                material: material as u8,
                flags: BatchFlags(
                    draw.controls
                        .as_ref()
                        .map_or(0, |value| value.unclassified.bits()),
                ),
                ..Default::default()
            };
            if let Some(controls) = &draw.controls {
                batch.prefix.set_supplemental(controls.supplemental);
                if let Some(bounds) = controls.bounds {
                    let corners = [bounds.minimum, bounds.maximum]
                        .map(|corner| corner.map(|value| value.round() as i16));
                    let error = [bounds.minimum, bounds.maximum]
                        .iter()
                        .flatten()
                        .zip(corners.iter().flatten())
                        .map(|(source, value)| (*source - f32::from(*value)).abs())
                        .fold(0f32, f32::max);
                    context.approximate(
                        "world.draw.bounds",
                        error,
                        "draw culling bounds quantized to signed sixteen-bit coordinates",
                    )?;
                    if !batch.prefix.set_culling_bounds(corners) {
                        context.unsupported(
                            "world.draw.bounds",
                            "target batch prefix has no historical culling bounds",
                        )?;
                    }
                }
                match controls.lighting {
                    Lighting::Transition => output.header_mut().transition_batch_count += 1,
                    Lighting::Interior => output.header_mut().interior_batch_count += 1,
                    Lighting::Exterior => output.header_mut().exterior_batch_count += 1,
                    Lighting::Unknown(_) => context.unsupported(
                        "world.draw.lighting",
                        "unknown illumination class has no native batch-count role",
                    )?,
                }
            } else {
                output.header_mut().exterior_batch_count += 1;
            }
            let tangent = draw.subset.tangents;
            if remaining != draw.count as usize
                && (draw.subset.strip.is_some() || draw.subset.tangents.is_some())
            {
                return Err(Error::new(
                    "world.draw.streams",
                    "splitting an authored strip or tangent association requires explicit draw windows",
                ));
            }
            strip_windows.push(draw.subset.strip.map_or(
                crate::model::wmo::records::BatchOverride::default(),
                |window| crate::model::wmo::records::BatchOverride {
                    start_index: window.selection.range.first(),
                    index_count: window.selection.range.count() as u16,
                    padding: window.control,
                },
            ));
            tangent_starts.push(tangent.map_or(Ok(u16::MAX), |window| {
                u16::try_from(window.first_tangent)
                    .map_err(|error| Error::new("world.tangents", error.to_string()))
            })?);
            let (minimum, maximum) = draw
                .subset
                .vertex_window
                .map_or((minimum, maximum), |[first, last]| (first, last));
            let (minimum, maximum) = if let Some(window) = tangent {
                (
                    window.first_vertex,
                    window.first_vertex + window.vertex_count - 1,
                )
            } else {
                (minimum, maximum)
            };
            batch
                .prefix
                .set_selection(&mut batch.flags, material, (minimum, maximum))
                .map_err(|error| Error::new("world.batch", error.to_string()))?;
            batches.push(batch);
            first += count;
            remaining -= count;
        }
    }
    append_group(
        &mut output,
        GroupChunk::Batches(super::encode::array(&batches)?),
    )?;
    if instance.draws.iter().any(|draw| {
        mesh.subset(draw.subset)
            .expect("validated draw subset")
            .strip
            .is_some()
    }) {
        if instance
            .draws
            .iter()
            .filter_map(|draw| {
                mesh.subset(draw.subset)
                    .expect("validated draw subset")
                    .strip
            })
            .any(|window| window.selection.range.count() > u32::from(u16::MAX))
        {
            return Err(Error::new(
                "world.strips",
                "strip window exceeds native count capacity",
            ));
        }
        append_group(
            &mut output,
            GroupChunk::BatchOverrides(super::encode::array(&strip_windows)?),
        )?;
    }
    if !mesh.geometry.strip_indices().is_empty() {
        let strips = mesh
            .geometry
            .strip_indices()
            .iter()
            .map(|index| {
                u16::try_from(index).map_err(|error| Error::new("world.strips", error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        append_group(
            &mut output,
            GroupChunk::StripIndices(super::encode::array(&strips)?),
        )?;
    }
    if !mesh.geometry.tangent_vectors().is_empty()
        || tangent_starts.iter().any(|start| *start != u16::MAX)
    {
        append_group(
            &mut output,
            GroupChunk::Tangents(crate::model::wmo::group::Tangents {
                first_tangent: super::encode::array(&tangent_starts)?,
                tangents: super::encode::array(&mesh.geometry.tangent_vectors().to_vec())?,
            }),
        )?;
    }
    if !shadow_batches.is_empty() {
        append_group(
            &mut output,
            GroupChunk::ShadowBatches(super::encode::array(&shadow_batches)?),
        )?;
    }
    if spherical_fogs.len() > 4 || spherical_fogs.iter().any(|index| *index >= 255) {
        append_group(
            &mut output,
            GroupChunk::FogReferences(super::encode::array(&spherical_fogs)?),
        )?;
    }
    if !box_fogs.is_empty() {
        append_group(
            &mut output,
            GroupChunk::FogBoxReferences(super::encode::array(&box_fogs)?),
        )?;
    }
    let ground = mesh
        .geometry
        .triangle_controls()
        .iter()
        .map(|control| control.ground_type)
        .collect::<Vec<_>>();
    let query_first = ground.iter().position(Option::is_some);
    if let Some(first) = query_first {
        let end = ground
            .iter()
            .rposition(Option::is_some)
            .expect("ground range present")
            + 1;
        let values = ground[first..end]
            .iter()
            .map(|value| {
                value.ok_or_else(|| {
                    Error::new(
                        "world.ground_types",
                        "native query table requires a contiguous classification range",
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        append_group(
            &mut output,
            GroupChunk::GroundTypes(super::encode::array(&values)?),
        )?;
    }
    if query_first.is_some() || region.detail_doodad_key.is_some() {
        append_group(
            &mut output,
            GroupChunk::QueryGeometry(crate::model::wmo::group::QueryGeometry {
                first_face: query_first.unwrap_or(0) as u32,
                extension: std::borrow::Cow::Owned(region.detail_doodad_key.into_iter().collect()),
            }),
        )?;
    }
    if let Some(color) = region.ambient_override {
        append_group(&mut output, GroupChunk::AmbientOverride(super::rgba(color)))?;
    }
    if !region.terrain_cut_planes.is_empty() {
        let planes = region
            .terrain_cut_planes
            .iter()
            .map(|plane| crate::model::wmo::records::Plane {
                normal: [plane[0], plane[1], plane[2]],
                distance: plane[3],
            })
            .collect::<Vec<_>>();
        append_group(
            &mut output,
            GroupChunk::TerrainCutPlanes(super::encode::array(&planes)?),
        )?;
    }
    for liquid in &region.liquids {
        append_group(
            &mut output,
            GroupChunk::Liquid(super::liquids::write(
                liquid,
                materials,
                model.metadata().options.database_liquids,
            )?),
        )?;
    }
    Ok(output)
}
fn append_group<'a, D: Dialect>(
    group: &mut Group<'a, D>,
    chunk: GroupChunk<'a, D>,
) -> Result<(), Error> {
    group.chunks_mut().push(chunk);
    Ok(())
}
