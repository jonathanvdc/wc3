//! Fracture authoring retains geometry identities and actual cutting controls.
use super::super::codec::{
    append, lookup, mapped, read_error, read_vectors, read_words, vectors, words,
};
use super::choices::*;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::physics::*;
use crate::model::semantic::{MaterialSlotId, MeshId, NodeId, Values};
use crate::model::wem::{records, Bool, RawDocument, Ref, Target};
use std::collections::BTreeMap;
pub(super) trait Slot<T: Copy + Default> {
    fn value(&self) -> T;
    fn value_or(&self, _fallback: T) -> T {
        self.value()
    }
}
impl<T: Copy + Default> Slot<T> for () {
    fn value(&self) -> T {
        T::default()
    }
    fn value_or(&self, fallback: T) -> T {
        fallback
    }
}
impl Slot<u8> for u8 {
    fn value(&self) -> u8 {
        *self
    }
}
impl Slot<u16> for u16 {
    fn value(&self) -> u16 {
        *self
    }
}
impl Slot<u32> for u32 {
    fn value(&self) -> u32 {
        *self
    }
}
impl Slot<f32> for f32 {
    fn value(&self) -> f32 {
        *self
    }
}
impl Slot<Bool> for Bool {
    fn value(&self) -> Bool {
        *self
    }
}
impl Slot<[f32; 3]> for [f32; 3] {
    fn value(&self) -> [f32; 3] {
        *self
    }
}
pub(super) trait NodeSlot {
    fn node_index(&self) -> u32;
}
impl NodeSlot for () {
    fn node_index(&self) -> u32 {
        u32::MAX
    }
}
impl NodeSlot for u32 {
    fn node_index(&self) -> u32 {
        *self
    }
}
pub(super) trait Reference<T: Target> {
    fn reference(&self) -> Ref<T>;
}
impl<T: Target> Reference<T> for () {
    fn reference(&self) -> Ref<T> {
        Ref::default()
    }
}
impl<T: Target> Reference<T> for Ref<T> {
    fn reference(&self) -> Ref<T> {
        *self
    }
}
fn optional<T: Copy>(values: &[T], index: u32, field: &str) -> Result<Option<T>, Error> {
    if index == u32::MAX {
        Ok(None)
    } else {
        lookup(values, index, field).map(Some)
    }
}
fn write_optional<T: Ord>(
    values: &BTreeMap<T, u32>,
    id: Option<T>,
    field: &str,
) -> Result<u32, Error> {
    id.map(|id| mapped(values, id, field))
        .transpose()
        .map(|id| id.unwrap_or(u32::MAX))
}
fn read_u16<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::U16>,
) -> Result<Values<'a, u16>, Error> {
    let rows = document
        .resolve::<records::U16>(reference.cast())
        .map_err(read_error)?;
    for value in rows.iter() {
        value.map_err(read_error)?;
    }
    Ok(Values::view(rows.len(), move |index| {
        rows.get(index).unwrap().unwrap().value
    }))
}
fn write_u16(
    document: &mut RawDocument<'_>,
    values: &Values<'_, u16>,
) -> Result<Ref<records::targets::U16>, Error> {
    Ok(append(
        document,
        &values
            .iter()
            .map(|value| records::U16 { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
fn read_byte<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::Byte>,
) -> Result<Values<'a, u8>, Error> {
    let rows = document
        .resolve::<records::Byte>(reference.cast())
        .map_err(read_error)?;
    for value in rows.iter() {
        value.map_err(read_error)?;
    }
    Ok(Values::view(rows.len(), move |index| {
        rows.get(index).unwrap().unwrap().value
    }))
}
fn write_byte(
    document: &mut RawDocument<'_>,
    values: &Values<'_, u8>,
) -> Result<Ref<records::targets::Byte>, Error> {
    Ok(append(
        document,
        &values
            .iter()
            .map(|value| records::Byte { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
fn read_real<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::Real>,
) -> Result<Values<'a, f32>, Error> {
    let rows = document
        .resolve::<records::Real>(reference.cast())
        .map_err(read_error)?;
    for value in rows.iter() {
        value.map_err(read_error)?;
    }
    Ok(Values::view(rows.len(), move |index| {
        rows.get(index).unwrap().unwrap().value
    }))
}
fn write_real(
    document: &mut RawDocument<'_>,
    values: &Values<'_, f32>,
) -> Result<Ref<records::targets::Real>, Error> {
    Ok(append(
        document,
        &values
            .iter()
            .map(|value| records::Real { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
pub(super) fn read<'a, D: records::FractureRecipeDialect>(
    document: &'a RawDocument<'_>,
    row: &records::FractureRecipe<D>,
    nodes: &[NodeId],
    meshes: &[MeshId],
    slots: &[MaterialSlotId],
) -> Result<FractureRecipe<'a>, Error>
where
    D::Method: Slot<u8>,
    D::Total: Slot<u16>,
    D::Pinned: Reference<records::targets::Byte>,
    D::SliceAlong: Reference<records::targets::Byte>,
    D::SliceDirections: Reference<records::targets::Vector3>,
    D::SliceCounts: Reference<records::targets::U16>,
    D::SliceJitters: Reference<records::targets::Real>,
    D::SliceTilts: Reference<records::targets::Real>,
    D::SliceShifts: Reference<records::targets::Real>,
    D::PlaneNormals: Reference<records::targets::Vector3>,
    D::PlaneOffsets: Reference<records::targets::Real>,
    D::Push: Slot<u8>,
    D::Release: Slot<u8>,
    D::Over: Slot<f32>,
    D::Loosen: Slot<f32>,
    D::Centre: Slot<[f32; 3]>,
    D::Radius: Slot<f32>,
    D::Strength: Slot<f32>,
    D::Length: Slot<f32>,
    D::At: Slot<f32>,
    D::FillingUvs: Slot<u8>,
    D::MapSize: Slot<u32>,
    D::FillingMade: Slot<Bool>,
    D::Before: Slot<u8>,
    D::WholeGate: NodeSlot,
    D::PiecesGate: NodeSlot,
    D::PlaneCentres: Reference<records::targets::Vector3>,
    D::PlaneAlongs: Reference<records::targets::Vector3>,
    D::PlaneWidths: Reference<records::targets::Real>,
    D::PlaneHeights: Reference<records::targets::Real>,
    D::CutSkeleton: Slot<Bool>,
    D::Bones: Reference<records::targets::U32>,
    D::CutRagdoll: Slot<Bool>,
{
    Ok(FractureRecipe {
        keep_whole: row.keep_whole.get(),
        sources: read_words(document, row.sources)?
            .iter()
            .map(|index| optional(meshes, index, "fracture mesh"))
            .collect::<Result<Vec<_>, _>>()?,
        made: read_words(document, row.made)?
            .iter()
            .map(|index| optional(meshes, index, "fracture mesh"))
            .collect::<Result<Vec<_>, _>>()?,
        pieces: read_u16(document, row.pieces)?,
        skin_nodes: read_words(document, row.skin_nodes)?
            .iter()
            .map(|index| {
                if index == 0 {
                    Ok(None)
                } else {
                    lookup(nodes, index - 1, "fracture skin node").map(Some)
                }
            })
            .collect::<Result<Vec<_>, _>>()?,
        seed: row.seed,
        near_blast: row.near_blast,
        even: row.even,
        grain: read_grain_axis(row.grain),
        stretch: row.stretch,
        smallest: row.smallest,
        open_parts: read_open_parts(row.open_parts),
        thickness: row.thickness,
        inside: optional(slots, row.inside, "fracture interior slot")?,
        uv_scale: row.uv_scale,
        hull_points: row.hull_points,
        split_hollow: row.split_hollow.get(),
        material: row.material,
        helper: optional(nodes, row.helper, "fracture node")?,
        field: optional(nodes, row.field, "fracture node")?,
        channel: row.channel,
        seeds: read_vectors(document, row.seeds)?,
        statics: read_vectors(document, row.statics)?,
        method: read_fracture_method(row.method.value()),
        total: row.total.value(),
        pinned: read_byte(document, row.pinned.reference())?,
        slice_along: read_byte(document, row.slice_along.reference())?.map(read_slice_along),
        slice_directions: read_vectors(document, row.slice_directions.reference())?,
        slice_counts: read_u16(document, row.slice_counts.reference())?,
        slice_jitters: read_real(document, row.slice_jitters.reference())?,
        slice_tilts: read_real(document, row.slice_tilts.reference())?,
        slice_shifts: read_real(document, row.slice_shifts.reference())?,
        plane_normals: read_vectors(document, row.plane_normals.reference())?,
        plane_offsets: read_real(document, row.plane_offsets.reference())?,
        push: read_push_mode(row.push.value()),
        release: read_release_order(row.release.value()),
        over: row.over.value(),
        loosen: row.loosen.value(),
        centre: row.centre.value(),
        radius: row.radius.value(),
        strength: row.strength.value_or(10.),
        length: row.length.value_or(0.1),
        at: row.at.value(),
        filling_uvs: read_interior_uvs(row.filling_uvs.value()),
        map_size: row.map_size.value_or(1024),
        filling_made: row.filling_made.value().get(),
        before: read_before_activation(row.before.value_or(2)),
        whole_gate: optional(nodes, row.whole_gate.node_index(), "fracture node")?,
        pieces_gate: optional(nodes, row.pieces_gate.node_index(), "fracture node")?,
        plane_centres: read_vectors(document, row.plane_centres.reference())?,
        plane_alongs: read_vectors(document, row.plane_alongs.reference())?,
        plane_widths: read_real(document, row.plane_widths.reference())?,
        plane_heights: read_real(document, row.plane_heights.reference())?,
        cut_skeleton: row.cut_skeleton.value().get(),
        bones: read_words(document, row.bones.reference())?
            .iter()
            .map(|index| lookup(nodes, index, "fracture bone"))
            .collect::<Result<Vec<_>, _>>()?,
        cut_ragdoll: row.cut_ragdoll.value().get(),
    })
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    row: &FractureRecipe<'_>,
    nodes: &BTreeMap<NodeId, u32>,
    meshes: &BTreeMap<MeshId, u32>,
    slots: &BTreeMap<MaterialSlotId, u32>,
) -> Result<records::FractureRecipe<records::V8>, Error> {
    Ok(records::FractureRecipe {
        keep_whole: Bool::from(row.keep_whole),
        sources: words(
            document,
            row.sources
                .iter()
                .copied()
                .map(|id| write_optional(meshes, id, "fracture mesh"))
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        made: words(
            document,
            row.made
                .iter()
                .copied()
                .map(|id| write_optional(meshes, id, "fracture mesh"))
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        pieces: write_u16(document, &row.pieces)?,
        skin_nodes: words(
            document,
            row.skin_nodes
                .iter()
                .copied()
                .map(|id| {
                    id.map(|id| {
                        mapped(nodes, id, "fracture skin node").and_then(|index| {
                            index
                                .checked_add(1)
                                .ok_or_else(|| read_error("node capacity exhausted"))
                        })
                    })
                    .transpose()
                    .map(|id| id.unwrap_or(0))
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        seed: row.seed,
        near_blast: row.near_blast,
        even: row.even,
        grain: write_grain_axis(row.grain),
        stretch: row.stretch,
        smallest: row.smallest,
        open_parts: write_open_parts(row.open_parts),
        thickness: row.thickness,
        inside: write_optional(slots, row.inside, "fracture interior slot")?,
        uv_scale: row.uv_scale,
        hull_points: row.hull_points,
        split_hollow: Bool::from(row.split_hollow),
        material: row.material,
        helper: write_optional(nodes, row.helper, "fracture node")?,
        field: write_optional(nodes, row.field, "fracture node")?,
        channel: row.channel,
        seeds: vectors(document, &row.seeds)?,
        statics: vectors(document, &row.statics)?,
        method: write_fracture_method(row.method),
        total: row.total,
        pinned: write_byte(document, &row.pinned)?,
        slice_along: write_byte(document, &row.slice_along.clone().map(write_slice_along))?,
        slice_directions: vectors(document, &row.slice_directions)?,
        slice_counts: write_u16(document, &row.slice_counts)?,
        slice_jitters: write_real(document, &row.slice_jitters)?,
        slice_tilts: write_real(document, &row.slice_tilts)?,
        slice_shifts: write_real(document, &row.slice_shifts)?,
        plane_normals: vectors(document, &row.plane_normals)?,
        plane_offsets: write_real(document, &row.plane_offsets)?,
        push: write_push_mode(row.push),
        release: write_release_order(row.release),
        over: row.over,
        loosen: row.loosen,
        centre: row.centre,
        radius: row.radius,
        strength: row.strength,
        length: row.length,
        at: row.at,
        filling_uvs: write_interior_uvs(row.filling_uvs),
        map_size: row.map_size,
        filling_made: Bool::from(row.filling_made),
        before: write_before_activation(row.before),
        whole_gate: write_optional(nodes, row.whole_gate, "fracture node")?,
        pieces_gate: write_optional(nodes, row.pieces_gate, "fracture node")?,
        plane_centres: vectors(document, &row.plane_centres)?,
        plane_alongs: vectors(document, &row.plane_alongs)?,
        plane_widths: write_real(document, &row.plane_widths)?,
        plane_heights: write_real(document, &row.plane_heights)?,
        cut_skeleton: Bool::from(row.cut_skeleton),
        bones: words(
            document,
            row.bones
                .iter()
                .copied()
                .map(|id| mapped(nodes, id, "fracture bone"))
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        cut_ragdoll: Bool::from(row.cut_ragdoll),
    })
}
