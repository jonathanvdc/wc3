//! Geometry definitions with no draw commands retain their binding and detail placement.
use super::{geometry_write::Division, packing::UvTransform, vertex_write::Layout, Graph};
use crate::model::conversion::{Context, Error};
use crate::model::m3::records;
use crate::model::semantic::{M3Geometry, MeshId, NodeId, Topology};
use std::collections::BTreeMap;
pub(super) struct Buffers<'a> {
    pub divisions: &'a mut BTreeMap<u32, Division>,
    pub vertices: &'a mut Vec<u8>,
    pub palette: &'a mut Vec<records::U16>,
    pub regions: &'a mut BTreeMap<MeshId, Vec<u32>>,
}
fn count(value: usize, path: &str) -> Result<u32, Error> {
    u32::try_from(value).map_err(|_| Error::new(path, "native count capacity exceeded"))
}
fn small(value: u32, path: &str) -> Result<u16, Error> {
    u16::try_from(value).map_err(|_| Error::new(path, "native 16-bit capacity exceeded"))
}
pub(super) fn write(
    graph: &Graph<'_>,
    layout: &Layout,
    bones: &BTreeMap<NodeId, u32>,
    buffers: Buffers<'_>,
    context: &mut Context,
) -> Result<(), Error> {
    for (id, mesh) in graph.meshes() {
        let M3Geometry::Display(_) = &mesh.geometry else {
            continue;
        };
        if graph.instances().any(|(_, instance)| instance.mesh == id) {
            continue;
        }
        let controls = mesh.controls.as_ref();
        let placement = controls.and_then(|controls| controls.undrawn.as_ref());
        let detail = placement.map_or(0, |placement| placement.detail_level);
        let instances = placement.map_or(1, |placement| placement.instance_count);
        let division = buffers.divisions.entry(detail).or_default();
        if division.instances.is_some_and(|old| old != instances) {
            return Err(Error::new(
                "m3.geometry.instances",
                "definitions at one detail level have conflicting instancing controls",
            ));
        }
        division.instances = Some(instances);
        let Topology::Triangles(triangles) = mesh.topology() else {
            return Err(Error::new(
                "m3.geometry.topology",
                "native regions require triangle connectivity",
            ));
        };
        let first_vertex = count(buffers.vertices.len() / layout.stride(), "m3.vertices")?;
        let vertex_count = count(mesh.positions().len(), "m3.vertices")?;
        if vertex_count > 65536 {
            return Err(Error::new(
                "m3.region.vertices",
                "an undrawn region exceeds the 16-bit local vertex-index range",
            ));
        }
        let first_index = count(division.faces.len(), "m3.faces")?;
        for triangle in triangles.iter() {
            for index in triangle {
                division.faces.push(records::U16 {
                    value: small(index, "m3.faces.index")?,
                });
            }
        }
        let index_count = count(division.faces.len(), "m3.faces")? - first_index;
        let first_lookup = small(
            count(buffers.palette.len(), "m3.skin.lookup")?,
            "m3.skin.lookup",
        )?;
        let binding = controls.and_then(|controls| controls.skin);
        let lookup_count = if let Some(id) = binding {
            let skin = graph
                .skin(id)
                .ok_or_else(|| Error::new("m3.skin", "definition binding does not exist"))?;
            for joint in &skin.joints {
                let bone = *bones.get(joint).ok_or_else(|| {
                    Error::new("m3.skin.joint", "binding joint has no native bone")
                })?;
                buffers.palette.push(records::U16 {
                    value: small(bone, "m3.skin.joint")?,
                });
            }
            small(
                count(skin.joints.len(), "m3.skin.palette")?,
                "m3.skin.palette",
            )?
        } else {
            0
        };
        let uv = UvTransform::for_mesh(
            mesh.controls
                .as_ref()
                .and_then(|controls| controls.uv_encoding),
            mesh.texture_coordinates(),
        )?;
        for index in 0..mesh.positions().len() {
            buffers
                .vertices
                .extend(layout.row(mesh, index, uv, context)?);
        }
        let root = controls
            .and_then(|controls| controls.root_bone)
            .map(|id| {
                bones
                    .get(&id)
                    .copied()
                    .ok_or_else(|| Error::new("m3.region.root", "region root has no native bone"))
            })
            .transpose()?
            .map(|id| small(id, "m3.region.root"))
            .transpose()?
            .unwrap_or(u16::MAX);
        let region = count(division.regions.len(), "m3.region.count")?;
        small(region, "m3.region.count")?;
        division.regions.push(records::Region::<records::V5> {
            index: controls.map_or(region, |controls| controls.source_id),
            unknown: controls.map_or(0, |controls| controls.unknown),
            first_vertex,
            vertex_count,
            first_index,
            index_count,
            bone_count: lookup_count,
            first_bone_lookup: first_lookup,
            bone_lookup_count: lookup_count,
            padding: 0,
            bone_weight_pairs: 4,
            unknown_byte: controls.map_or(1, |controls| controls.unknown_byte),
            root_bone: root,
            flags: records::RegionFlags(controls.map_or(0, |controls| controls.flags.bits())),
            uv_scale: uv.scale,
            uv_offset: uv.offset,
        });
        buffers.regions.entry(id).or_default().push(region);
    }
    Ok(())
}
