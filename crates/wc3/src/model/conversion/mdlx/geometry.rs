//! Borrowed vertex arrays and logical joint identities decoded without WEM.
use super::state::ImportState;
use crate::model::conversion::Error;
use crate::model::mdlx::geometry::SkinWeight;
use crate::model::mdlx::{self as native, ModelDialect, ModelVersion};
use crate::model::semantic::profiles::mdlx::{self as profile, Bone};
use crate::model::semantic::profiles::Mdlx;
use crate::model::semantic::{
    Bounds, ClipId, ClockId, Draw, Influence, JointInfluences, MaterialId, MdlxGeometry, Mesh,
    MeshInstance, ModelBuilder, Node, NodeId, Skin, Values,
};
use std::collections::BTreeMap;
use zerovec::ZeroVec;

pub(super) fn read<'a, D: ModelDialect>(
    source: &'a native::Model<'_, D>,
    builder: &mut ModelBuilder<'a, Mdlx>,
    hierarchy: (&BTreeMap<u32, NodeId>, &[NodeId]),
    materials: &[MaterialId],
    clips: &[(ClipId, f64)],
    globals: &[ClockId],
    profile: &mut ImportState<'a>,
) -> Result<(), Error> {
    let (identities, nodes) = hierarchy;
    let node_indices: BTreeMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(index, id)| (*id, index as u32))
        .collect();
    let mut meshes = Vec::new();
    let mut placements = Vec::new();
    for mesh in source.geosets() {
        if mesh.primitive_types().iter().any(|kind| kind != 4) || mesh.face_indices().len() % 3 != 0
        {
            return Err(Error::new(
                "mesh.topology",
                "Warcraft III geometry is not a triangle list",
            ));
        }
        let positions = mesh.vertices();
        let faces = mesh.face_indices();
        let triangles = Values::view(faces.len() / 3, move |index| {
            [
                u32::from(faces.get(index * 3).expect("checked triangle index")),
                u32::from(faces.get(index * 3 + 1).expect("checked triangle index")),
                u32::from(faces.get(index * 3 + 2).expect("checked triangle index")),
            ]
        });
        let mut offset = 0usize;
        let groups = mesh
            .primitive_counts()
            .iter()
            .map(|count| {
                if count % 3 != 0 {
                    return Err(Error::new(
                        "mesh.topology",
                        "triangle group contains incomplete triangle",
                    ));
                }
                let start = offset;
                offset += count as usize / 3;
                if offset > triangles.len() {
                    return Err(Error::new(
                        "mesh.topology",
                        "triangle group exceeds index array",
                    ));
                }
                let data = triangles.clone();
                Ok(Values::view(count as usize / 3, move |index| {
                    data.get(start + index).expect("checked triangle group")
                }))
            })
            .collect::<Result<Vec<_>, Error>>()?;
        if offset != triangles.len() {
            return Err(Error::new(
                "mesh.topology",
                "triangle groups do not cover index array",
            ));
        }
        let mut geometry = MdlxGeometry::triangle_groups(
            Values::from_zero_vec(
                ZeroVec::parse_bytes(positions.as_bytes()).expect("validated canonical array"),
            ),
            groups,
        )?;
        let mut value = Mesh::<Mdlx>::new(geometry.clone());
        value.name = mesh.try_name().unwrap_or_default();
        let normals = mesh.normals();
        geometry = geometry.with_normals(Values::from_zero_vec(
            ZeroVec::parse_bytes(normals.as_bytes()).expect("validated canonical array"),
        ))?;
        if let Ok(Some(tangents)) = mesh.try_tangents() {
            geometry = geometry.with_tangents(Values::from_zero_vec(
                ZeroVec::parse_bytes(tangents.as_bytes()).expect("validated canonical array"),
            ))?;
        }
        geometry = geometry.with_texture_coordinates(
            mesh.uv_sets()
                .iter()
                .map(|uv| {
                    Values::from_zero_vec(
                        ZeroVec::parse_bytes(uv.as_bytes()).expect("validated canonical array"),
                    )
                })
                .collect(),
        )?;
        value.bounds = Bounds {
            minimum: mesh.extent.minimum,
            maximum: mesh.extent.maximum,
            sphere_radius: mesh.extent.bounds_radius,
        };
        let rows = influences(mesh).map_err(|error| Error::new("mesh.skin", error))?;
        let rows = Values::owned(
            rows.into_iter()
                .map(|row| {
                    row.into_iter()
                        .map(|(id, weight)| {
                            let node = identities.get(&id).ok_or_else(|| {
                                Error::new("mesh.skin", "influence references an absent node")
                            })?;
                            Ok(Influence {
                                joint: node_indices[node],
                                weight,
                            })
                        })
                        .collect::<Result<Vec<_>, Error>>()
                })
                .collect::<Result<Vec<_>, Error>>()?,
        );
        geometry =
            geometry.with_influences(if let Ok(Some(weights)) = mesh.try_skin_weights() {
                let slots = weights
                    .iter()
                    .map(|source| {
                        let mut result = [Influence {
                            joint: 0,
                            weight: 0.,
                        }; 4];
                        for ((slot, bone), weight) in result
                            .iter_mut()
                            .zip(source.bone_indices())
                            .zip(source.weights())
                        {
                            let id = u32::from(bone);
                            slot.joint = identities
                                .get(&id)
                                .map(|node| node_indices[node])
                                .unwrap_or(id);
                            slot.weight = f32::from(weight) / 255.;
                            if weight != 0 && !identities.contains_key(&id) {
                                return Err(Error::new(
                                    "mesh.skin",
                                    "weighted joint does not exist",
                                ));
                            }
                        }
                        Ok(result)
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                JointInfluences::Four(Values::owned(slots))
            } else {
                JointInfluences::MatrixGroups(rows.map(|row: Vec<Influence>| {
                    row.into_iter().map(|influence| influence.joint).collect()
                }))
            })?;
        value.geometry = geometry;
        let primitive_count = faces.len() as u32 / 3;
        let mesh_id = builder.add_mesh(value)?;
        meshes.push(mesh_id);
        let skin = builder.add_skin(Skin::identity(nodes.to_vec()))?;
        let placement = builder.add_node(Node::named("Geometry"))?;
        placements.push(placement);
        let material = *materials
            .get(mesh.material_id as usize)
            .ok_or_else(|| Error::new("mesh.material", "material does not exist"))?;
        builder.add_instance(MeshInstance {
            controls: Default::default(),
            node: placement,
            mesh: mesh_id,
            skin: Some(skin),
            draws: vec![Draw {
                controls: Default::default(),
                first: 0,
                count: primitive_count,
                material,
            }],
            lod: mesh.try_level_of_detail().ok(),
            selection_group: mesh.selection_group,
        })?;
        profile.geometry.push((
            mesh_id,
            profile::Geometry {
                glider: false,
                unselectable: mesh.unselectable(),
                sequence_bounds: mesh
                    .sequence_extents
                    .iter()
                    .zip(clips)
                    .map(|(extent, (clip, _))| {
                        (
                            *clip,
                            [extent.minimum, extent.maximum],
                            extent.bounds_radius,
                        )
                    })
                    .collect(),
                unassigned_bounds: mesh
                    .sequence_extents
                    .iter()
                    .skip(clips.len())
                    .map(|extent| Bounds {
                        minimum: extent.minimum,
                        maximum: extent.maximum,
                        sphere_radius: extent.bounds_radius,
                    })
                    .collect(),
            },
        ));
    }
    let mut animation_ids = Vec::new();
    {
        let mut decoder = super::animation::Decoder { builder, globals };
        for source in source.geoset_animations() {
            let mesh =
                if source.geoset_id == u32::MAX {
                    None
                } else {
                    Some(*meshes.get(source.geoset_id as usize).ok_or_else(|| {
                        Error::new("geometry_animation.mesh", "mesh does not exist")
                    })?)
                };
            let placement = if mesh.is_some() {
                placements[source.geoset_id as usize]
            } else {
                decoder.builder.add_node(Node::named("GeometryAnimation"))?
            };
            let feature = decoder.builder.reserve_feature(placement);
            let owner = feature;
            let mut flags = source.flags;
            flags.set_drop_shadow(false);
            flags.set_color(false);
            {
                let payload = profile::Node::GeometryAnimation(profile::GeometryAnimation {
                    mesh,
                    opacity: decoder.bind(
                        &source.alpha,
                        owner,
                        profile::properties::geometry::OPACITY,
                        |value| value,
                    )?,
                    color: decoder.bind(
                        &source.color,
                        owner,
                        profile::properties::geometry::COLOR,
                        |value| value,
                    )?,
                    use_color: source.flags.color(),
                    drop_shadow: source.flags.drop_shadow(),
                    unclassified: flags.bits().into(),
                });
                decoder.builder.set_feature(feature, payload)?;
            }
            animation_ids.push(feature);
        }
    }
    for bone in source.bones() {
        profile.bones.push((
            *identities
                .get(&bone.node.object_id)
                .ok_or_else(|| Error::new("bone", "node does not exist"))?,
            Bone {
                mesh: if bone.geoset_id == u32::MAX {
                    None
                } else {
                    Some(
                        *meshes
                            .get(bone.geoset_id as usize)
                            .ok_or_else(|| Error::new("bone.mesh", "mesh does not exist"))?,
                    )
                },
                animation: if bone.geoset_animation_id == u32::MAX {
                    None
                } else {
                    Some(
                        *animation_ids
                            .get(bone.geoset_animation_id as usize)
                            .ok_or_else(|| {
                                Error::new("bone.animation", "geometry animation does not exist")
                            })?,
                    )
                },
            },
        ));
    }
    for glider in source.gliders() {
        profile.gliders.push(
            *meshes
                .get(glider.geoset_id as usize)
                .ok_or_else(|| Error::new("glider.mesh", "mesh does not exist"))?,
        );
    }
    Ok(())
}

fn influences<V: ModelVersion>(
    mesh: &native::Geoset<'_, V>,
) -> Result<Vec<Vec<(u32, f32)>>, String> {
    if let Ok(Some(rows)) = mesh.try_skin_weights() {
        return Ok(rows
            .iter()
            .map(|row| {
                row.bone_indices()
                    .into_iter()
                    .zip(row.weights())
                    .filter(|(_, weight)| *weight != 0)
                    .map(|(bone, weight)| (u32::from(bone), f32::from(weight) / 255.0))
                    .collect()
            })
            .collect());
    }
    let mut groups = Vec::new();
    let indices: Vec<_> = mesh.matrix_indices().iter().collect();
    let mut offset = 0usize;
    for size in mesh.matrix_group_sizes().iter() {
        let end = offset
            .checked_add(size as usize)
            .ok_or("matrix-group length overflow")?;
        let group = indices
            .get(offset..end)
            .ok_or("matrix-group exceeds matrix table")?;
        groups.push(
            group
                .iter()
                .copied()
                .map(|id| (id, 1.0 / size as f32))
                .collect::<Vec<_>>(),
        );
        offset = end;
    }
    if offset != indices.len() {
        return Err("matrix table has unclaimed entries".into());
    }
    mesh.vertex_groups()
        .iter()
        .map(|index| {
            groups
                .get(usize::from(index))
                .cloned()
                .ok_or_else(|| "vertex matrix-group index is out of range".into())
        })
        .collect()
}
