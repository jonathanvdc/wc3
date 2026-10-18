//! Normalized logical draw geometry encoded into native Warcraft III geosets.
use super::encode_animation::Encoder;
use crate::model::conversion::{Context, Error};
use crate::model::formats::mdlx::{self as native, Dialect};
use crate::model::{JointInfluences, LookId, MaterialId, MeshId, NodeId, Topology};
use native::geometry::SkinWeight;
use std::collections::BTreeMap;
use zerovec::{ule::AsULE, ZeroVec};

pub(super) fn write<'a, 'data, T: Dialect>(
    target: &mut native::Document<'a, T>,
    encoder: &mut Encoder<'data, 'a, '_>,
    nodes: &BTreeMap<NodeId, u32>,
    materials: &BTreeMap<MaterialId, u32>,
    look: Option<LookId>,
) -> Result<BTreeMap<MeshId, Vec<u32>>, Error> {
    let model = encoder.model;
    let mut output = Vec::new();
    let mut mapping = BTreeMap::<MeshId, Vec<u32>>::new();
    for (instance_id, instance) in model.instances() {
        for draw in &instance.draws {
            let mesh = model.mesh(instance.mesh).expect("validated mesh");
            let draw = mesh.resolve_draw(draw)?;
            if mesh.positions().len() > 65536 {
                return Err(Error::new(
                    "mesh.vertices",
                    "Warcraft III geosets use 16-bit vertex indices",
                ));
            }
            let all_triangles = triangles(mesh.topology());
            let triangles = &all_triangles[draw.first as usize..(draw.first + draw.count) as usize];
            let faces = triangles
                .iter()
                .flatten()
                .map(|index| {
                    u16::try_from(*index)
                        .map_err(|error| Error::new("mesh.triangles", error.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let positions = packed(mesh.positions());
            let normals = mesh.normals().map(packed).unwrap_or_else(|| {
                ZeroVec::alloc_from_slice(&vec![[0.; 3]; mesh.positions().len()])
            });
            let mut geoset = native::Geoset::<T::Version>::new(positions, normals, faces)
                .map_err(|error| Error::new("mesh", error.to_string()))?;
            let mut remaining = draw.count as usize;
            let mut skip = draw.first as usize;
            let mut counts = Vec::new();
            for size in group_sizes(mesh.topology()) {
                let omitted = skip.min(size);
                skip -= omitted;
                let taken = (size - omitted).min(remaining);
                if taken != 0 {
                    counts.push((taken * 3) as u32);
                    remaining -= taken;
                }
            }
            geoset
                .set_triangle_group_counts(&counts)
                .map_err(|error| Error::new("mesh.triangle_groups", error.to_string()))?;
            geoset.material_id = materials[&model
                .material_for_draw(instance_id, draw.id, look)
                .expect("validated material look")];
            geoset.selection_group = instance.selection_group;
            geoset.extent = native::GeosetExtent {
                minimum: mesh.bounds.minimum,
                maximum: mesh.bounds.maximum,
                bounds_radius: mesh.bounds.sphere_radius,
            };
            if let Err(error) = geoset.try_set_name(&mesh.name) {
                if !mesh.name.is_empty() {
                    encoder
                        .context
                        .unsupported("mesh.name", error.to_string())?;
                }
            }
            if let Some(lod) = instance.lod {
                if let Err(error) = geoset.try_set_level_of_detail(lod) {
                    if lod != 0 {
                        encoder.context.unsupported("mesh.lod", error.to_string())?;
                    }
                }
            }
            geoset.set_uv_sets(mesh.texture_coordinates().iter().map(packed).collect());
            if let Some(tangents) = mesh.tangents() {
                if let Err(error) = geoset.try_set_tangents(Some(packed(tangents))) {
                    encoder
                        .context
                        .unsupported("mesh.tangents", error.to_string())?;
                }
            }
            if let Some(metadata) = model.mesh(instance.mesh).map(|mesh| &mesh.controls) {
                geoset.set_unselectable(metadata.unselectable);
                let mut extents = Vec::new();
                for (clip, _) in model.clips() {
                    if let Some((_, extent)) =
                        metadata.sequence_bounds.iter().find(|(id, _)| *id == clip)
                    {
                        extents.push(super::extents::write(*extent));
                    }
                }
                extents.extend(
                    metadata
                        .unassigned_bounds
                        .iter()
                        .copied()
                        .map(super::extents::write),
                );
                geoset.sequence_extents = extents.iter().copied().collect();
            }
            let mut groups = Vec::<Vec<u32>>::new();
            let mut vertices = Vec::new();
            let mut weighted = Vec::new();
            let mut exact_groups =
                !matches!(mesh.geometry.vertex_influences(), JointInfluences::Four(_));
            for index in 0..mesh.influences().len() {
                let row = mesh
                    .influences()
                    .joint_weights(index)
                    .expect("checked influence index");
                let joints = instance.skin.and_then(|id| model.skin(id)).ok_or_else(|| {
                    Error::new("mesh.skin", "influence array has no skin binding")
                })?;
                let group = row
                    .iter()
                    .map(|influence| {
                        joints
                            .joints
                            .get(influence.joint as usize)
                            .map(|id| nodes[id])
                            .ok_or_else(|| Error::new("mesh.skin", "joint identity does not exist"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                exact_groups &= row
                    .iter()
                    .all(|influence| (influence.weight - 1. / row.len() as f32).abs() <= 1e-6);
                let group_id = groups
                    .iter()
                    .position(|old| *old == group)
                    .unwrap_or_else(|| {
                        groups.push(group.clone());
                        groups.len() - 1
                    });
                vertices.push(group_id);
                weighted.push(
                    group
                        .iter()
                        .copied()
                        .zip(row.iter().map(|influence| influence.weight))
                        .collect::<Vec<_>>(),
                );
            }
            if mesh.influences().is_empty() {
                groups.push(vec![nodes[&instance.node]]);
                vertices.resize(mesh.positions().len(), 0);
            }
            if exact_groups && groups.len() <= 256 {
                geoset
                    .set_matrix_groups(groups)
                    .map_err(|error| Error::new("mesh.skin", error.to_string()))?;
                geoset
                    .set_vertex_groups(
                        &vertices
                            .iter()
                            .map(|index| *index as u8)
                            .collect::<Vec<_>>(),
                    )
                    .map_err(|error| Error::new("mesh.skin", error.to_string()))?;
            } else {
                let rows = weighted
                    .into_iter()
                    .map(|row| {
                        if row.len() > 4 {
                            encoder.context.unsupported(
                                "mesh.skin",
                                "weighted Warcraft III vertices support four influences",
                            )?;
                        }
                        let mut bones = [0u16; 4];
                        let mut weights = [0u16; 4];
                        for (slot, (bone, weight)) in row.into_iter().take(4).enumerate() {
                            bones[slot] = u16::try_from(bone)
                                .map_err(|error| Error::new("mesh.skin", error.to_string()))?;
                            weights[slot] = quantize(weight, encoder.context)?;
                        }
                        <T::Version as native::GeosetLayout>::SkinWeights::from_influences(
                            bones, weights,
                        )
                        .ok_or_else(|| {
                            Error::new("mesh.skin", "influence exceeds the selected skin layout")
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                if let Err(error) = geoset.try_set_skin_weights(Some(rows.into_iter().collect())) {
                    encoder
                        .context
                        .unsupported("mesh.skin", error.to_string())?;
                }
                geoset
                    .set_matrix_groups(vec![vec![nodes[&instance.node]]])
                    .map_err(|error| Error::new("mesh.skin", error.to_string()))?;
                geoset
                    .set_vertex_groups(&vec![0; mesh.positions().len()])
                    .map_err(|error| Error::new("mesh.skin", error.to_string()))?;
            }
            mapping
                .entry(instance.mesh)
                .or_default()
                .push(output.len() as u32);
            output.push(geoset);
        }
    }
    target.set_geosets(output);
    Ok(mapping)
}
fn quantize(weight: f32, context: &mut Context) -> Result<u16, Error> {
    let rounded = (weight * 255.).round().clamp(0., 255.);
    let error = (weight - rounded / 255.).abs();
    if error > f32::EPSILON * 4. {
        context.approximate(
            "mesh.skin.weights",
            error,
            "skin weight quantized to Warcraft III bytes",
        )?;
    }
    Ok(rounded as u16)
}

fn packed<'a, T>(values: &'a ZeroVec<'_, T>) -> ZeroVec<'a, T>
where
    T: AsULE + Clone + Send + Sync,
    T::ULE: Send + Sync,
{
    values.as_slice().as_zerovec()
}

fn triangles(topology: &Topology<'_>) -> Vec<[u32; 3]> {
    match topology {
        Topology::Triangles(values) => values.to_vec(),
        Topology::Mixed(groups) => groups.iter().flat_map(triangles).collect(),
        _ => unreachable!("MDLX geometry vocabulary only permits triangles"),
    }
}

fn group_sizes(topology: &Topology<'_>) -> Vec<usize> {
    match topology {
        Topology::Triangles(values) => vec![values.len()],
        Topology::Mixed(groups) => groups.iter().flat_map(group_sizes).collect(),
        _ => unreachable!("checked MDLX topology"),
    }
}
