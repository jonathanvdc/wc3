//! Joins compatible region geometry without changing draw order or shared identities.
use crate::model::conversion::Error;
use crate::model::semantic::profiles::Wmo;
use crate::model::semantic::{
    Bounds, DrawAppearanceSchema, DrawSubsets, InstanceId, LookId, Model, ModelBuilder,
    PrimitiveRange, PrimitiveSelection, Resources, Values, WmoGeometry,
};
use std::collections::{BTreeMap, BTreeSet};

type Normalized<'a> = (Model<'a, Wmo>, Resources<'a, Wmo>);

pub(super) fn prepare<'data>(
    model: &Model<'data, Wmo>,
    resources: &Resources<'data, Wmo>,
    look: Option<LookId>,
) -> Result<Option<Normalized<'data>>, Error> {
    let mut merges = Vec::new();
    for (system_id, world) in model.systems() {
        for (group_index, group) in world.groups.iter().enumerate() {
            let mut lods = BTreeMap::<u32, Vec<InstanceId>>::new();
            for id in &group.instances {
                lods.entry(
                    model
                        .instance(*id)
                        .expect("validated placement")
                        .lod
                        .unwrap_or(0),
                )
                .or_default()
                .push(*id);
            }
            for instances in lods.into_values().filter(|instances| instances.len() > 1) {
                merges.push((system_id, group_index, instances));
            }
        }
    }
    if merges.is_empty() {
        return Ok(None);
    }
    let mut builder = ModelBuilder::from_parts(model.clone(), resources.clone());
    let mut removed = BTreeSet::new();
    let mut old_meshes = BTreeSet::new();
    let mut old_nodes = BTreeSet::new();
    for (system_id, group_index, instances) in merges {
        let first_id = instances[0];
        let first = model.instance(first_id).expect("validated placement");
        let first_mesh = model.mesh(first.mesh).expect("validated geometry");
        for id in &instances[1..] {
            let instance = model.instance(*id).expect("validated placement");
            let mesh = model.mesh(instance.mesh).expect("validated geometry");
            let mut controls = instance.controls.clone();
            controls.interior_windows = first.controls.interior_windows;
            controls.exterior_windows = first.controls.exterior_windows;
            if controls != first.controls
                || instance.selection_group != first.selection_group
                || mesh.name != first_mesh.name
            {
                return Err(Error::new(
                    "world.group.merge",
                    "geometry placements at one detail level must share region behavior, descriptions, and geometry names",
                ));
            }
            if mesh.normals().is_some() != first_mesh.normals().is_some()
                || mesh.texture_coordinates().len() != first_mesh.texture_coordinates().len()
                || mesh.geometry.color_sets().len() != first_mesh.geometry.color_sets().len()
                || mesh.geometry.lightmap_coordinates().is_empty()
                    != first_mesh.geometry.lightmap_coordinates().is_empty()
            {
                return Err(Error::new(
                    "world.group.merge.streams",
                    "joined geometry must use the same normal, texture, color, and baked-light stream vocabulary",
                ));
            }
        }
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut coordinates = vec![Vec::new(); first_mesh.texture_coordinates().len()];
        let mut colors = vec![Vec::new(); first_mesh.geometry.color_sets().len()];
        let mut lightmap_coordinates = Vec::new();
        let mut triangles = Vec::new();
        let mut faces = Vec::new();
        let mut strips = Vec::new();
        let mut tangents = Vec::new();
        let mut draws = Vec::new();
        let mut subsets = DrawSubsets::default();
        let mut vertex_base = 0u32;
        let mut face_base = 0u32;
        let mut strip_base = 0u32;
        let mut tangent_base = 0u32;
        let mut windows = super::lighting_windows::Windows::default();
        let mut minimum = first_mesh.bounds.minimum;
        let mut maximum = first_mesh.bounds.maximum;
        for id in &instances {
            let instance = model.instance(*id).expect("validated placement");
            let mesh = model.mesh(instance.mesh).expect("validated geometry");
            for axis in 0..3 {
                minimum[axis] = minimum[axis].min(mesh.bounds.minimum[axis]);
                maximum[axis] = maximum[axis].max(mesh.bounds.maximum[axis]);
            }
            let source_vertices = u32::try_from(mesh.positions().len()).map_err(error)?;
            let source_faces = u32::try_from(mesh.topology().primitive_count()).map_err(error)?;
            let next_vertex = vertex_base.checked_add(source_vertices).ok_or_else(|| {
                Error::new(
                    "world.group.merge",
                    "vertex count exceeds logical connectivity capacity",
                )
            })?;
            let next_face = face_base.checked_add(source_faces).ok_or_else(|| {
                Error::new(
                    "world.group.merge",
                    "face count exceeds logical draw capacity",
                )
            })?;
            positions.push(mesh.positions().clone());
            if let Some(values) = mesh.normals() {
                normals.push(values.clone());
            }
            for (target, source) in coordinates.iter_mut().zip(mesh.texture_coordinates()) {
                target.push(source.clone());
            }
            for (target, source) in colors.iter_mut().zip(mesh.geometry.color_sets()) {
                target.push(source.clone());
            }
            if !mesh.geometry.lightmap_coordinates().is_empty() {
                lightmap_coordinates.push(mesh.geometry.lightmap_coordinates().clone());
            }
            let crate::model::semantic::Topology::Triangles(connectivity) = mesh.topology() else {
                return Err(Error::new(
                    "world.group.merge",
                    "world geometry requires triangle connectivity",
                ));
            };
            triangles.push(
                connectivity
                    .clone()
                    .map(move |triangle| triangle.map(|vertex| vertex + vertex_base)),
            );
            faces.push(mesh.geometry.triangle_controls().clone());
            strips.push(
                mesh.geometry
                    .strip_indices()
                    .clone()
                    .map(move |vertex| vertex + vertex_base),
            );
            tangents.push(mesh.geometry.tangent_vectors().clone());
            windows.append(
                instance.controls.interior_windows,
                instance.controls.exterior_windows,
                vertex_base,
                u32::try_from(draws.len()).map_err(error)?,
            )?;
            for (ordinal, source) in instance.draws.iter().enumerate() {
                let source_draw = mesh.resolve_draw(source)?;
                let mut subset = source_draw.subset.clone();
                subset.selection = PrimitiveSelection {
                    connectivity: subsets.primary(),
                    range: PrimitiveRange::Primitives {
                        first: source_draw.first.checked_add(face_base).ok_or_else(|| {
                            Error::new("world.group.merge", "draw selection exceeds face capacity")
                        })?,
                        count: source_draw.count,
                    },
                };
                if let Some([first, last]) = &mut subset.vertex_window {
                    *first = first.checked_add(vertex_base).ok_or_else(|| {
                        Error::new("world.group.merge", "vertex window exceeds capacity")
                    })?;
                    *last = last.checked_add(vertex_base).ok_or_else(|| {
                        Error::new("world.group.merge", "vertex window exceeds capacity")
                    })?;
                }
                if let Some(strip) = &mut subset.strip {
                    strip.selection.connectivity = subsets.connectivity_id(1);
                    strip.selection.range = PrimitiveRange::StripIndices {
                        first: strip
                            .selection
                            .range
                            .first()
                            .checked_add(strip_base)
                            .ok_or_else(|| {
                                Error::new("world.group.merge", "strip window exceeds capacity")
                            })?,
                        count: strip.selection.range.count(),
                    };
                }
                if let Some(tangents) = &mut subset.tangents {
                    tangents.first_vertex = tangents
                        .first_vertex
                        .checked_add(vertex_base)
                        .ok_or_else(|| {
                            Error::new("world.group.merge", "tangent vertex exceeds capacity")
                        })?;
                    tangents.first_tangent = tangents
                        .first_tangent
                        .checked_add(tangent_base)
                        .ok_or_else(|| {
                            Error::new("world.group.merge", "tangent pool exceeds capacity")
                        })?;
                }
                let mut draw = source.clone();
                draw.subset = subsets.insert(subset);
                draw.material = model
                    .material_for_draw(*id, ordinal as u32, look)
                    .expect("validated material assignment");
                draws.push(draw);
            }
            vertex_base = next_vertex;
            face_base = next_face;
            strip_base = strip_base
                .checked_add(u32::try_from(mesh.geometry.strip_indices().len()).map_err(error)?)
                .ok_or_else(|| {
                    Error::new("world.group.merge", "strip pool exceeds logical capacity")
                })?;
            tangent_base = tangent_base
                .checked_add(u32::try_from(mesh.geometry.tangent_vectors().len()).map_err(error)?)
                .ok_or_else(|| {
                    Error::new("world.group.merge", "tangent pool exceeds logical capacity")
                })?;
        }
        let geometry = WmoGeometry::triangles(concat(positions)?, concat(triangles)?)?
            .with_normals(concat(normals)?)?
            .with_texture_coordinates(
                coordinates
                    .into_iter()
                    .map(concat)
                    .collect::<Result<_, _>>()?,
            )?
            .with_color_sets(colors.into_iter().map(concat).collect::<Result<_, _>>()?)?
            .with_triangle_controls(concat(faces)?)?
            .with_lightmap_coordinates(concat(lightmap_coordinates)?)?
            .with_strip_indices(concat(strips)?)?
            .with_tangent_vectors(concat(tangents)?)?;
        let mut mesh = first_mesh.clone();
        mesh.geometry = geometry;
        mesh.subsets = subsets;
        mesh.bounds = Bounds {
            minimum,
            maximum,
            sphere_radius: minimum
                .iter()
                .zip(maximum)
                .map(|(a, b)| ((b - a) / 2.).powi(2))
                .sum::<f32>()
                .sqrt(),
        };
        let mesh = builder.add_mesh(mesh)?;
        old_meshes.insert(first.mesh);
        let mut instance = first.clone();
        instance.mesh = mesh;
        instance.draws = draws;
        (
            instance.controls.interior_windows,
            instance.controls.exterior_windows,
        ) = windows.finish()?;
        builder.set_instance(first_id, instance)?;
        let mut world = builder
            .model()
            .system(system_id)
            .expect("validated world")
            .clone();
        world.groups[group_index]
            .instances
            .retain(|id| *id == first_id || !instances.contains(id));
        builder.set_system(system_id, world)?;
        for id in &instances[1..] {
            let instance = builder.remove_instance(*id).expect("existing placement");
            removed.insert(*id);
            old_meshes.insert(instance.mesh);
            old_nodes.insert(instance.node);
        }
    }
    let mut appearance = model.appearance().clone();
    for (id, look) in model.looks() {
        let mut updated = look.clone();
        updated.bindings.retain(|binding| {
            !removed.contains(&binding.instance)
                && !model.instance(binding.instance).is_some_and(|instance| {
                    builder
                        .model()
                        .instance(binding.instance)
                        .is_some_and(|normalized| normalized.draws.len() != instance.draws.len())
                })
        });
        appearance.remove_look(id);
        appearance.add_look(id, updated)?;
    }
    appearance.set_default_look(model.default_look())?;
    builder.set_appearance(appearance);
    for mesh in old_meshes {
        if builder
            .model()
            .instances()
            .all(|(_, instance)| instance.mesh != mesh)
        {
            builder.remove_mesh(mesh);
        }
    }
    for node in old_nodes {
        if builder
            .model()
            .instances()
            .all(|(_, instance)| instance.node != node)
            && builder.model().node_features(node).next().is_none()
        {
            builder.remove_node(node);
        }
    }
    Ok(Some(builder.finish_parts()?))
}
fn concat<'data, T: Clone + Send + Sync + 'data>(
    parts: Vec<Values<'data, T>>,
) -> Result<Values<'data, T>, Error> {
    let mut ends = Vec::with_capacity(parts.len());
    let mut length = 0usize;
    for part in &parts {
        length = length
            .checked_add(part.len())
            .ok_or_else(|| Error::new("world.group.merge", "stream length overflows"))?;
        ends.push(length);
    }
    Ok(Values::view(length, move |index| {
        let part = ends.partition_point(|end| *end <= index);
        let base = if part == 0 { 0 } else { ends[part - 1] };
        parts[part]
            .get(index - base)
            .expect("checked concatenated stream")
    }))
}
fn error(error: impl std::fmt::Display) -> Error {
    Error::new("world.group.merge", error.to_string())
}
