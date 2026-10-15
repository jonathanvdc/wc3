//! Direct geometry views over native packed fields and face arrays.
use super::{
    hierarchy::Skeleton,
    materials::read_error,
    mesh_bounds,
    topology::{divisions, regions},
    vertices::Packed,
};

use crate::model::conversion::m3::logical::GraphBuilder;
use crate::model::conversion::Error;
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::profiles::m3::{self as surface, UvEncoding};
use crate::model::semantic::ExplicitBinding;
use crate::model::semantic::{
    Bounds, Draw, Influence, M3Geometry, MaterialId, Mesh, MeshId, MeshInstance, Node, Skin, Values,
};
use native::Envelope;
use std::collections::BTreeSet;

pub(super) struct Geometry {
    pub regions: Vec<Vec<MeshId>>,
}
pub(super) fn read<'a, E: Envelope>(
    source: &'a native::Model<'_, E>,
    builder: &mut GraphBuilder<'a>,
    skeleton: &Skeleton<'a>,
    materials: &[MaterialId],
) -> Result<Geometry, Error> {
    let root = source.root_record().map_err(read_error)?;
    let document = source.document();
    let (division_ref, palette_ref, bounds) =
        native::visit_m3_root!(root, |r| (r.divisions, r.bone_lookup, r.bounds));
    builder.set_bounds(Some(Bounds {
        minimum: bounds.minimum,
        maximum: bounds.maximum,
        sphere_radius: bounds.radius,
    }));
    let packed = Packed::read(source)?;
    let palette = document
        .resolve::<nr::U16>(palette_ref.cast())
        .map_err(read_error)?
        .iter()
        .map(|r| r.map(|r| r.value))
        .collect::<Result<Vec<_>, _>>()
        .map_err(read_error)?;
    let mut placement = None;
    let mut result = Geometry {
        regions: Vec::new(),
    };
    for (lod, division) in divisions(document, division_ref)
        .map_err(read_error)?
        .into_iter()
        .enumerate()
    {
        let extent_features =
            mesh_bounds::read(document, division.msec, skeleton, lod as u32, builder)?;
        let faces = document
            .resolve::<nr::U16>(division.faces.cast())
            .map_err(read_error)?;
        for face in faces.iter() {
            face.map_err(read_error)?;
        }
        let mut meshes = Vec::new();
        for (index, region) in regions(document, division.regions, faces.len())
            .map_err(read_error)?
            .into_iter()
            .enumerate()
        {
            let first = region.first_vertex as usize;
            let count = region.vertex_count as usize;
            let face_start = region.first_index as usize;
            let face_count = region.index_count as usize;
            if first
                .checked_add(count)
                .is_none_or(|end| end > packed.len())
                || face_start
                    .checked_add(face_count)
                    .is_none_or(|end| end > faces.len())
                || !face_count.is_multiple_of(3)
            {
                return Err(Error::new(
                    "m3.region",
                    "vertex or triangular face range is invalid",
                ));
            }
            for at in face_start..face_start + face_count {
                if u32::from(faces.get(at).unwrap().unwrap().value) >= region.vertex_count {
                    return Err(Error::new(
                        "m3.region.faces",
                        "face index exceeds region vertex range",
                    ));
                }
            }
            let face_bytes = document
                .referenced_section(division.faces)
                .map_err(read_error)?
                .map_or(&[][..], |section| section.bytes());
            let triangles = Values::view(face_count / 3, move |i| {
                [0, 1, 2].map(|lane| {
                    let offset = (face_start + i * 3 + lane) * 2;
                    u32::from(u16::from_le_bytes(
                        face_bytes[offset..offset + 2].try_into().unwrap(),
                    ))
                })
            });
            let start = region.first_bone_lookup as usize;
            let end = start + region.bone_lookup_count as usize;
            let joint_indices = palette
                .get(start..end)
                .ok_or_else(|| Error::new("m3.region.skin", "bone palette exceeds lookup table"))?;
            let mut joints = Vec::new();
            let mut inverse = Vec::new();
            for index in joint_indices {
                joints.push(
                    *skeleton
                        .nodes
                        .get(*index as usize)
                        .ok_or_else(|| Error::new("m3.region.skin", "joint exceeds skeleton"))?,
                );
                inverse.push(skeleton.inverse_bind.get(*index as usize).ok_or_else(|| {
                    Error::new("m3.region.skin", "inverse bind matrix is missing")
                })?);
            }
            let mut influences = Values::view(count, |_| {
                [Influence {
                    joint: 0,
                    weight: 0.,
                }; 4]
            });
            for vertex in first..first + count {
                let (weights, indices) = packed.skin(vertex);
                let sum: u16 = weights.iter().map(|w| u16::from(*w)).sum();
                if sum != 0 && sum != 255 {
                    return Err(Error::new(
                        "m3.region.skin.weights",
                        "nonzero native weights must sum to 255; arbitrary-total normalization is not established",
                    ));
                }
                if weights
                    .iter()
                    .zip(indices)
                    .any(|(weight, index)| *weight != 0 && index as usize >= joints.len())
                {
                    return Err(Error::new(
                        "m3.region.skin",
                        "weighted vertex joint exceeds palette",
                    ));
                }
            }
            let skin = if joints.is_empty() {
                None
            } else {
                influences = packed.influences(first, count, |weights, joints| {
                    let total: f32 = weights.iter().map(|w| f32::from(*w)).sum();
                    [0, 1, 2, 3].map(|lane| Influence {
                        joint: u32::from(joints[lane]),
                        weight: if total == 0. {
                            0.
                        } else {
                            f32::from(weights[lane]) / total
                        },
                    })
                });
                Some(builder.add_skin(Skin {
                    joints,
                    binding: ExplicitBinding::new(Values::owned(inverse))?,
                })?)
            };
            let geometry = M3Geometry::new(
                packed.positions(first, count),
                packed.normals(first, count),
                triangles,
                influences,
            )?
            .with_tangents(packed.tangents(first, count))?
            .with_colors(packed.colors(first, count))?
            .with_texture_coordinates(packed.uv_sets(
                first,
                count,
                region.uv_scale / 32768.,
                region.uv_offset,
            ))?;
            let mut mesh = Mesh::new(geometry).named(format!("LOD {lod} region {index}"));
            mesh.controls = Some(surface::MeshControls {
                uv_encoding: Some(UvEncoding {
                    scale: region.uv_scale,
                    offset: region.uv_offset,
                }),
                skin,
                undrawn: None,
                source_id: region.id,
                flags: surface::values::RegionFlags(region.flags.bits()),
                root_bone: if skeleton.nodes.is_empty() || region.root_bone == u16::MAX {
                    None
                } else {
                    Some(
                        *skeleton
                            .nodes
                            .get(region.root_bone as usize)
                            .ok_or_else(|| {
                                Error::new("m3.region.root", "root bone exceeds skeleton")
                            })?,
                    )
                },
                unknown: region.unknown,
                unknown_byte: region.unknown_byte,
            });
            mesh.bounds = bounds_of(mesh.positions());
            let id = builder.add_mesh(mesh)?;
            meshes.push((id, skin, face_count as u32 / 3));
        }
        let mut drawn = BTreeSet::new();
        macro_rules! batches {
            ($v:ty) => {
                for batch in document
                    .resolve::<nr::Batch<$v>>(division.batches.cast())
                    .map_err(read_error)?
                    .iter()
                {
                    let batch = batch.map_err(read_error)?;
                    let (mesh, _skin, count) = *meshes
                        .get(batch.region_index as usize)
                        .ok_or_else(|| Error::new("m3.batch.region", "region exceeds division"))?;
                    drawn.insert(mesh);
                    let material =
                        *materials
                            .get(batch.material_index as usize)
                            .ok_or_else(|| {
                                Error::new("m3.batch.material", "material exceeds logical table")
                            })?;
                    let batching_bone = if batch.visibility_bone == u16::MAX {
                        None
                    } else {
                        Some(
                            *skeleton
                                .nodes
                                .get(batch.visibility_bone as usize)
                                .ok_or_else(|| {
                                    Error::new("m3.batch.batching", "batch gate exceeds skeleton")
                                })?,
                        )
                    };
                    let placement = match placement {
                        Some(node) => node,
                        None => {
                            let node = builder.add_node(Node::named(""))?;
                            placement = Some(node);
                            node
                        }
                    };
                    let subset = builder.add_primary_subset(mesh, 0, count)?;
                    builder.add_instance_with_materials(
                        MeshInstance {
                            controls: Some(surface::InstanceControls {
                                instance_count: division.instances,
                            }),
                            node: placement,
                            mesh,
                            skin: None,
                            draws: vec![Draw::new(
                                subset,
                                Some(surface::BatchControls {
                                    flags: surface::values::BatchFlags(batch.flags.bits()),
                                    priority: batch.priority_plane,
                                    batching_bone,
                                    bounds: if extent_features.is_empty() {
                                        None
                                    } else {
                                        Some(
                                            *extent_features
                                                .get(batch.bounds_index as usize)
                                                .ok_or_else(|| {
                                                    Error::new(
                                                        "m3.batch.bounds",
                                                        "bounds control exceeds division",
                                                    )
                                                })?,
                                        )
                                    },
                                    color_control: batch.color_index,
                                }),
                            )],
                            lod: Some(lod as u32),
                            selection_group: 0,
                        },
                        vec![material],
                    )?;
                }
            };
        }
        if !division.batches.is_empty() {
            let revision = document
                .referenced_section(division.batches)
                .map_err(read_error)?
                .unwrap()
                .revision();
            match revision {
                0 => batches!(nr::V0),
                1 => batches!(nr::V1),
                _ => return Err(Error::new("m3.batch", "unsupported batch revision")),
            }
        }
        for (id, _, _) in &meshes {
            if !drawn.contains(id) {
                let mut mesh = builder
                    .model()
                    .mesh(*id)
                    .expect("checked region definition")
                    .clone();
                mesh.controls.as_mut().expect("region controls").undrawn =
                    Some(surface::UndrawnRegion {
                        detail_level: lod as u32,
                        instance_count: division.instances,
                    });
                builder.set_mesh(*id, mesh)?;
            }
        }
        result
            .regions
            .push(meshes.into_iter().map(|(id, _, _)| id).collect());
    }
    Ok(result)
}

fn bounds_of(positions: &Values<'_, [f32; 3]>) -> Bounds {
    let Some(first) = positions.get(0) else {
        return Bounds::default();
    };
    let mut minimum = first;
    let mut maximum = first;
    for point in positions.iter() {
        for axis in 0..3 {
            minimum[axis] = minimum[axis].min(point[axis]);
            maximum[axis] = maximum[axis].max(point[axis]);
        }
    }
    let center = [0, 1, 2].map(|axis| (f64::from(minimum[axis]) + f64::from(maximum[axis])) * 0.5);
    let radius = positions
        .iter()
        .map(|point| {
            [0, 1, 2]
                .into_iter()
                .map(|axis| (f64::from(point[axis]) - center[axis]).powi(2))
                .sum::<f64>()
                .sqrt()
        })
        .fold(0., f64::max) as f32;
    Bounds {
        minimum,
        maximum,
        sphere_radius: radius,
    }
}
