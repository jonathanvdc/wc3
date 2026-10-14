use super::super::codec::{append, lookup, mapped, read_error};
use crate::model::conversion::Error;
use crate::model::semantic::MaterialSlotId;
use crate::model::semantic::{
    profiles::wem::physics::soft::{SoftPart, SurfacePoint},
    MeshId, NodeId, Values,
};
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::BTreeMap;
pub(super) fn string<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::Character>,
) -> Result<&'a str, Error> {
    Ok(document
        .string(reference)
        .map_err(read_error)?
        .unwrap_or_default())
}
pub(super) fn write_string(
    document: &mut RawDocument<'_>,
    value: &str,
) -> Result<Ref<records::targets::Character>, Error> {
    if value.as_bytes().contains(&0) {
        return Err(Error::new("wem.soft.name", "name contains an embedded NUL"));
    }
    let bytes: Vec<_> = value
        .bytes()
        .map(|value| records::Character { value })
        .collect();
    Ok(append(document, &bytes)?.cast())
}
pub(super) fn optional<T: Copy>(values: &[T], index: u32, field: &str) -> Result<Option<T>, Error> {
    if index == u32::MAX {
        Ok(None)
    } else {
        lookup(values, index, field).map(Some)
    }
}
pub(super) fn node(index: u32, nodes: &[NodeId]) -> Result<Option<NodeId>, Error> {
    optional(nodes, index, "soft node")
}
pub(super) fn node_index(
    value: Option<NodeId>,
    nodes: &BTreeMap<NodeId, u32>,
) -> Result<u32, Error> {
    value
        .map(|value| mapped(nodes, value, "soft node"))
        .transpose()
        .map(|value| value.unwrap_or(u32::MAX))
}
pub(super) fn point(
    value: &records::SoftPlace,
    meshes: &[MeshId],
) -> Result<Option<SurfacePoint>, Error> {
    let Some(mesh) = optional(meshes, value.mesh, "soft attachment mesh")? else {
        return Ok(None);
    };
    Ok(Some(SurfacePoint {
        mesh,
        face: value.face,
        weights: value.weights,
        radius: value.radius,
    }))
}
pub(super) fn write_point(
    value: Option<&SurfacePoint>,
    meshes: &BTreeMap<MeshId, u32>,
) -> Result<records::SoftPlace, Error> {
    match value {
        Some(value) => Ok(records::SoftPlace {
            mesh: mapped(meshes, value.mesh, "soft attachment mesh")?,
            face: value.face,
            weights: value.weights,
            radius: value.radius,
        }),
        None => Ok(records::SoftPlace {
            mesh: u32::MAX,
            face: u32::MAX,
            weights: [1., 0., 0.],
            radius: 0.,
        }),
    }
}
pub(super) fn reals<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::Real>,
) -> Result<Values<'a, f32>, Error> {
    let rows = document
        .resolve::<records::Real>(reference.cast())
        .map_err(read_error)?;
    for row in rows.iter() {
        row.map_err(read_error)?;
    }
    Ok(Values::view(rows.len(), move |index| {
        rows.get(index)
            .expect("checked soft scalar")
            .expect("validated soft scalar")
            .value
    }))
}
pub(super) fn mesh_ids<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::U32>,
    meshes: &[MeshId],
) -> Result<Values<'a, MeshId>, Error> {
    let rows = document
        .resolve::<records::U32>(reference.cast())
        .map_err(read_error)?;
    for row in rows.iter() {
        lookup(meshes, row.map_err(read_error)?.value, "soft mesh")?;
    }
    let meshes = meshes.to_vec();
    Ok(Values::view(rows.len(), move |index| {
        meshes[rows
            .get(index)
            .expect("checked soft mesh")
            .expect("validated soft mesh")
            .value as usize]
    }))
}
pub(super) fn node_ids<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::U32>,
    nodes: &[NodeId],
) -> Result<Values<'a, NodeId>, Error> {
    let rows = document
        .resolve::<records::U32>(reference.cast())
        .map_err(read_error)?;
    for row in rows.iter() {
        lookup(nodes, row.map_err(read_error)?.value, "soft chain node")?;
    }
    let nodes = nodes.to_vec();
    Ok(Values::view(rows.len(), move |index| {
        nodes[rows
            .get(index)
            .expect("checked soft chain node")
            .expect("validated soft chain node")
            .value as usize]
    }))
}
pub(super) fn parts<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::SoftPart>,
    meshes: &[MeshId],
    slots: &[MaterialSlotId],
) -> Result<Values<'a, SoftPart>, Error> {
    let rows = document
        .resolve::<records::SoftPart>(reference.cast())
        .map_err(read_error)?;
    for row in rows.iter() {
        let row = row.map_err(read_error)?;
        lookup(meshes, row.mesh, "soft part mesh")?;
        optional(slots, row.material, "soft part material")?;
    }
    let meshes = meshes.to_vec();
    let slots = slots.to_vec();
    Ok(Values::view(rows.len(), move |index| {
        let row = rows
            .get(index)
            .expect("checked soft part")
            .expect("validated soft part");
        SoftPart {
            mesh: meshes[row.mesh as usize],
            material: if row.material == u32::MAX {
                None
            } else {
                Some(slots[row.material as usize])
            },
        }
    }))
}
pub(super) fn part(
    value: &SoftPart,
    meshes: &BTreeMap<MeshId, u32>,
    slots: &BTreeMap<MaterialSlotId, u32>,
) -> Result<records::SoftPart, Error> {
    Ok(records::SoftPart {
        mesh: mapped(meshes, value.mesh, "soft part mesh")?,
        material: value
            .material
            .map(|value| mapped(slots, value, "soft part material"))
            .transpose()?
            .unwrap_or(u32::MAX),
    })
}
pub(super) fn real_records(
    document: &mut RawDocument<'_>,
    values: impl IntoIterator<Item = f32>,
) -> Result<Ref<records::targets::Real>, Error> {
    let rows: Vec<_> = values
        .into_iter()
        .map(|value| records::Real { value })
        .collect();
    Ok(append(document, &rows)?.cast())
}
