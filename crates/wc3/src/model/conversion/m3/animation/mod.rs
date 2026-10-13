//! Typed editable animation channels and native sequence/controller graphs.
use super::{
    append, append_string, native, nr, quantize, read_error, revision, string, wem, Context, Error,
};
use native::{Envelope, Ref};
use std::collections::BTreeMap;
use wem::{records as wr, semantic as sem};
mod blocks;
mod bounds;
mod clock;
mod events;
mod export;
mod graph;
mod import;
mod keys;
mod sequences;
mod states;
use blocks::{BlockInput, NativeBlocks};
mod native_export;
use native_export::export_native;
mod modes;
mod properties;
pub(super) use export::export_clips;
pub(super) use import::import_native_clips;
use keys::*;
pub(super) use modes::{modes, reconcile_controls, reconcile_material};
use properties::*;
fn put<T: wem::Record + wem::Write>(
    doc: &mut wem::RawDocument<'static>,
    rows: &[T],
) -> Result<wem::Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(wem::Ref::default());
    }
    let id = doc.append(rows)?;
    Ok(wem::Ref::from_section(rows.len() as u32, id.index()))
}
fn bytes(
    doc: &mut wem::RawDocument<'static>,
    rows: &[u8],
) -> Result<wem::Ref<wr::targets::Byte>, Error> {
    Ok(put(
        doc,
        &rows
            .iter()
            .map(|&value| wr::Byte { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
fn text(
    doc: &mut wem::RawDocument<'static>,
    value: &str,
) -> Result<wem::Ref<wr::targets::Character>, Error> {
    Ok(put(
        doc,
        &value
            .bytes()
            .map(|value| wr::Character { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}

struct ImportBlock<'view, 'data, E: Envelope> {
    source: &'view native::Document<'data, E>,
    controller: &'view nr::SubTrackContainer<nr::V4>,
    selector: u32,
    id: u32,
    start: u32,
    clip: usize,
}
struct BoundsTarget<'a> {
    raw: &'a mut wem::RawDocument<'static>,
    model: &'a mut wr::ModelRecord<wr::V9>,
    channels: &'a mut Vec<wr::AnimChannel>,
    tracks: &'a mut Vec<wr::SubTrack<wr::V2>>,
    fresh: &'a mut u32,
}
#[derive(Default)]
struct NativeExportOptions<'a> {
    vector_base: usize,
    quaternion_base: usize,
    selected_container: Option<usize>,
    bones: Option<&'a mut [nr::Bone<nr::V1>]>,
}
struct NativeOutput<'a> {
    ids: &'a mut Vec<nr::U32>,
    selectors: &'a mut Vec<nr::U32>,
}
