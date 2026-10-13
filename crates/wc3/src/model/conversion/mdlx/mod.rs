//! Bidirectional conversion between Warcraft III models and editable WEM graphs.
use super::{
    Asset, Context, Conversion, Error, ExportOptions, Format, ImportOptions, IssueKind,
    OpaquePayload, Plan, UnknownPolicy,
};
use crate::model::common::FileKey;
use crate::model::mdlx::{self as native, ModelDialect, ModelVersion};
use crate::model::wem::{self, records as wr, records::ProfileId, semantic as s, RawDocument, Ref};
use native::animation::{Interpolation, Track, TrackValue};
use native::geometry::SkinWeight;
use native::{mdx, Chunk, ModelChunk};
use std::borrow::Cow;
use std::collections::BTreeMap;
mod wem_view;
use wem_view::graph;
mod geometry;
use geometry::{extent, influences, native_extent};
mod hierarchy;
use hierarchy::{common_flags, native_flags, native_ids, patch_hierarchy};
mod animation;
use animation::{
    clip_starts, import_properties, merge_track, native_globals, native_track, overlay_integer,
    overlay_properties, read_track, semantic_track,
};
mod materials;
use materials::{common_material_flags, filter, native_layers, overlay_material};
mod effects;
use effects::{active_graph, decorate_native, overlay_effects};
mod metadata;
use metadata::{append, append_string, base, decorate_metadata, metadata, overlay_metadata};
fn text<const N: usize>(text: &native::FixedText<N>) -> String {
    text.text().into_owned()
}

fn failure(path: impl Into<String>, error: impl std::fmt::Display) -> Error {
    Error::new(path, error.to_string())
}

mod import;
pub use import::import;

mod export;
pub use export::plan;
