//! Editable world-model geometry, materials, environment nodes, and companion outputs.
use super::{
    Asset, Context, Conversion, Error, ExportOptions, Format, ImportOptions, IssueKind,
    OpaquePayload, Plan, UnknownPolicy,
};
use crate::model::common::FileKey;
use crate::model::wem::{self, records as wr, semantic as s, RawDocument, Ref};
use crate::model::wmo::{self as native, Bundle, Dialect, Group, GroupKey, Root};
use native::codec::{Array, Decoder};
use native::group::{GroupChunk, LiquidVertex, LiquidVertices};
use native::records::{self as nr, BatchMetadata, MaterialTailField};
use native::root::RootChunk;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::str::from_utf8;
mod environment;
mod export;
mod geometry;
mod import;
mod liquids;
mod materials;
mod metadata;
mod physics;
fn failure(path: impl Into<String>, error: impl std::fmt::Display) -> Error {
    Error::new(path, error.to_string())
}
fn collect<T: for<'data> native::Read<'data> + native::RecordLayout>(
    array: &Array<'_, T>,
    path: &str,
) -> Result<Vec<T>, Error> {
    array
        .iter()
        .map(|row| row.map_err(|error| failure(path, error)))
        .collect()
}
fn array<T: native::Write + native::RecordLayout>(
    records: &[T],
) -> Result<Array<'static, T>, Error> {
    Array::from_records(records).map_err(|error| failure("wmo.output", error))
}
fn bounds(value: [[f32; 3]; 2]) -> wr::Extent {
    let center = [0, 1, 2].map(|i| (value[0][i] + value[1][i]) * 0.5);
    let sphere_radius = (0..3)
        .map(|i| (value[1][i] - center[i]).powi(2))
        .sum::<f32>()
        .sqrt();
    wr::Extent {
        minimum: value[0],
        maximum: value[1],
        sphere_radius,
    }
}
fn text(bytes: &[u8], offset: u32, path: &str) -> Result<String, Error> {
    let value =
        native::resources::string_at(bytes, offset).map_err(|error| failure(path, error))?;
    Ok(from_utf8(value)
        .map_err(|error| failure(path, error))?
        .to_owned())
}
fn root_chunk<D: Dialect>(
    root: &mut Root<'static, D>,
    chunk: RootChunk<'static, D>,
) -> Result<(), Error> {
    root.insert_chunk(root.chunks().count(), chunk)
        .map_err(|error| failure("wmo.root", error))
}

mod legacy;
pub use legacy::{import, plan, Destination, Source, V14, V16};
