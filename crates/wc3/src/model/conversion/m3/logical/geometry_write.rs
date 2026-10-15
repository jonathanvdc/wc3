//! Native geometry assembly projects draw seams and relocates every emitted array.
use super::{
    animation::AnimationIds, packing::UvTransform, vertex_write::Layout, write::append, Graph,
};
use crate::model::conversion::{geometry::project, Context, Error};
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::{LookId, M3Geometry, MaterialId, MeshId, NodeId, Topology};
use native::{Envelope, Ref};
use std::collections::BTreeMap;

pub(super) struct Geometry {
    pub regions: BTreeMap<MeshId, Vec<u32>>,
    pub vertices: Vec<u8>,
    pub flags: nr::VertexFlags,
    pub divisions: Ref<nr::targets::MeshDivision>,
    pub bone_lookup: Ref<nr::targets::U16>,
}
#[derive(Default)]
pub(super) struct Division {
    pub(super) faces: Vec<nr::U16>,
    pub(super) regions: Vec<nr::Region<nr::V5>>,
    pub(super) batches: Vec<nr::Batch<nr::V1>>,
    pub(super) bounds: Vec<nr::MeshSection<nr::V1>>,
    pub(super) instances: Option<u32>,
}
fn narrow(value: u32, path: &str) -> Result<u16, Error> {
    u16::try_from(value).map_err(|_| Error::new(path, "native u16 capacity exceeded"))
}
pub(super) struct References<'a> {
    pub materials: &'a BTreeMap<MaterialId, u32>,
    pub bones: &'a BTreeMap<NodeId, u32>,
    pub animations: &'a AnimationIds,
}
pub(super) fn write<'view, 'source: 'view, E: Envelope>(
    model: &'view Graph<'source>,
    document: &mut native::Document<'_, E>,
    look: Option<LookId>,
    references: References<'_>,
    context: &mut Context,
) -> Result<Geometry, Error> {
    let References {
        materials,
        bones,
        animations: ids,
    } = references;
    let layout = Layout {
        uv_mask: model.meshes().fold(0u8, |mask, (_, mesh)| {
            mask | mesh
                .texture_coordinates()
                .iter()
                .enumerate()
                .fold(0u8, |mask, (channel, values)| {
                    mask | if values.is_empty() { 0 } else { 1 << channel }
                })
        }),
        color: model.meshes().any(|(_, m)| m.colors().is_some()),
    };
    let first_lod = model
        .instances()
        .map(|(_, value)| value.lod.unwrap_or(0))
        .min();
    let mut region_map = BTreeMap::<MeshId, Vec<u32>>::new();
    let mut vertices = Vec::new();
    let mut palette = Vec::new();
    let mut divisions = BTreeMap::<u32, Division>::new();
    let bounds_indices = super::definition_bounds::write(model, &mut divisions, bones, ids)?;
    for (instance_id, instance) in model.instances() {
        if instance.selection_group != 0 {
            context.unsupported(
                "m3.geometry.selection",
                "native M3 has no common selection-group field",
            )?;
        }
        let node = model
            .node(instance.node)
            .ok_or_else(|| Error::new("m3.geometry.node", "placement node is absent"))?;
        if node.parent.is_some() || node.transform != Default::default() {
            return Err(Error::new(
                "m3.geometry.placement",
                "native geometry requires identity placement; transform baking is a semantic translation",
            ));
        }
        let source = model
            .mesh(instance.mesh)
            .ok_or_else(|| Error::new("m3.geometry.mesh", "mesh is absent"))?;
        if !matches!(source.geometry, M3Geometry::Display(_)) {
            return Err(Error::new(
                "m3.geometry.kind",
                "native display divisions require display geometry",
            ));
        }
        let division = divisions.entry(instance.lod.unwrap_or(0)).or_default();
        let instance_count = instance
            .controls
            .as_ref()
            .map_or(1, |control| control.instance_count);
        if division
            .instances
            .is_some_and(|count| count != instance_count)
        {
            return Err(Error::new(
                "m3.geometry.instances",
                "one native division cannot encode conflicting instance counts",
            ));
        }
        division.instances = Some(instance_count);
        let collect_regions = Some(instance.lod.unwrap_or(0)) == first_lod
            && !region_map.contains_key(&instance.mesh);
        for projected in project(model, instance_id, look, context)? {
            let mesh = &projected.mesh;
            let Topology::Triangles(triangles) = mesh.topology() else {
                return Err(Error::new(
                    "m3.geometry.topology",
                    "native M3 encodes triangle geometry only",
                ));
            };
            let region_index = narrow(division.regions.len() as u32, "m3.region.count")?;
            let first_vertex = u32::try_from(vertices.len() / layout.stride())
                .map_err(|_| Error::new("m3.vertices", "native vertex count capacity exceeded"))?;
            let vertex_count = u32::try_from(mesh.positions().len())
                .map_err(|_| Error::new("m3.vertices", "native vertex count capacity exceeded"))?;
            if vertex_count > 65536 {
                return Err(Error::new(
                    "m3.region.vertices",
                    "a native region has u16 local face indices; partitioning requires a semantic translation",
                ));
            }
            let first_index = u32::try_from(division.faces.len())
                .map_err(|_| Error::new("m3.faces", "native face count capacity exceeded"))?;
            for triangle in triangles.iter() {
                for index in triangle {
                    division.faces.push(nr::U16 {
                        value: narrow(index, "m3.faces.index")?,
                    });
                }
            }
            let index_count = u32::try_from(division.faces.len())
                .map_err(|_| Error::new("m3.faces", "native face count capacity exceeded"))?
                - first_index;
            let first_bone_lookup = narrow(palette.len() as u32, "m3.skin.lookup")?;
            let bone_lookup_count = if let Some(skin) = projected.skin {
                let skin = model
                    .skin(skin)
                    .ok_or_else(|| Error::new("m3.skin", "skin is absent"))?;
                for joint in &skin.joints {
                    palette.push(nr::U16 {
                        value: narrow(
                            *bones.get(joint).ok_or_else(|| {
                                Error::new("m3.skin.joint", "joint has no native bone")
                            })?,
                            "m3.skin.joint",
                        )?,
                    });
                }
                narrow(skin.joints.len() as u32, "m3.skin.palette")?
            } else {
                0
            };
            let uv = UvTransform::for_mesh(
                source
                    .controls
                    .as_ref()
                    .and_then(|controls| controls.uv_encoding),
                mesh.texture_coordinates(),
            )?;
            for index in 0..mesh.positions().len() {
                vertices.extend(layout.row(mesh, index, uv, context)?);
            }
            let control = source.controls.as_ref();
            let root_bone = control
                .and_then(|c| c.root_bone)
                .map(|node| {
                    bones
                        .get(&node)
                        .copied()
                        .ok_or_else(|| Error::new("m3.region.root", "root has no native bone"))
                })
                .transpose()?
                .map(|index| narrow(index, "m3.region.root"))
                .transpose()?
                .unwrap_or(u16::MAX);
            if collect_regions {
                region_map
                    .entry(instance.mesh)
                    .or_default()
                    .push(u32::from(region_index));
            }
            division.regions.push(nr::Region::<nr::V5> {
                index: control.map_or(u32::from(region_index), |c| c.source_id),
                unknown: control.map_or(0, |c| c.unknown),
                first_vertex,
                vertex_count,
                first_index,
                index_count,
                bone_count: bone_lookup_count,
                first_bone_lookup,
                bone_lookup_count,
                padding: 0,
                bone_weight_pairs: 4,
                unknown_byte: control.map_or(1, |c| c.unknown_byte),
                root_bone,
                flags: nr::RegionFlags(control.map_or(0, |c| c.flags.bits())),
                uv_scale: uv.scale,
                uv_offset: uv.offset,
            });
            let draw = instance
                .draws
                .iter()
                .find(|draw| draw.id() == projected.source_draw)
                .ok_or_else(|| {
                    Error::new("m3.geometry.draw", "projected draw lost its source control")
                })?;
            let batch = draw.controls.as_ref();
            let bounds_index = if let Some(id) = batch.and_then(|b| b.bounds) {
                let (detail, index) = *bounds_indices
                    .get(&id)
                    .ok_or_else(|| Error::new("m3.batch.bounds", "bounds definition is absent"))?;
                if detail != instance.lod.unwrap_or(0) {
                    return Err(Error::new(
                        "m3.batch.bounds",
                        "draw and bounds definition belong to different detail groups",
                    ));
                }
                index
            } else {
                0
            };
            let visibility_bone = batch
                .and_then(|b| b.batching_bone)
                .map(|node| {
                    bones.get(&node).copied().ok_or_else(|| {
                        Error::new("m3.batch.batching", "batch gate has no native bone")
                    })
                })
                .transpose()?
                .map(|index| narrow(index, "m3.batch.batching"))
                .transpose()?
                .unwrap_or(u16::MAX);
            division.batches.push(nr::Batch::<nr::V1> {
                flags: nr::BatchFlags(batch.map_or(0, |b| b.flags.bits())),
                priority_plane: batch.map_or(0, |b| b.priority),
                region_index,
                bounds_index,
                color_index: batch.map_or(0, |b| b.color_control),
                material_index: narrow(
                    *materials.get(&projected.material).ok_or_else(|| {
                        Error::new("m3.batch.material", "material has no native table entry")
                    })?,
                    "m3.batch.material",
                )?,
                visibility_bone,
            });
        }
    }
    super::undrawn_geometry::write(
        model,
        &layout,
        bones,
        super::undrawn_geometry::Buffers {
            divisions: &mut divisions,
            vertices: &mut vertices,
            palette: &mut palette,
            regions: &mut region_map,
        },
        context,
    )?;
    let mut rows = Vec::new();
    for (detail, division) in divisions {
        while rows.len() < detail as usize {
            rows.push(nr::MeshDivision::<nr::V2>::default());
        }
        rows.push(nr::MeshDivision::<nr::V2> {
            faces: append(document, &division.faces)?.cast(),
            regions: append(document, &division.regions)?.cast(),
            batches: append(document, &division.batches)?.cast(),
            msec: append(document, &division.bounds)?.cast(),
            instances: division.instances.unwrap_or(1),
        });
    }
    Ok(Geometry {
        regions: region_map,
        vertices,
        flags: nr::VertexFlags(layout.flags()),
        divisions: append(document, &rows)?.cast(),
        bone_lookup: append(document, &palette)?.cast(),
    })
}
