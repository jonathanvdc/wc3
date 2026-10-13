//! Semantic WEM model views and construction, separated from physical layout.
mod types;
pub use types::*;
mod codec;
use codec::*;
mod integer;
mod validation;
pub use validation::validate_model;
mod build;
mod editing;
mod projection;
mod read;
mod triangulation;
use crate::model::wem::records as r;
use crate::model::wem::{
    Borrowed, Document, InlineList, Options, RawDocument, Read, ReadError, Record, RecordLayout,
    Ref, Write, WriteError, V3,
};
use std::borrow::Cow;
use std::collections::BTreeMap;
/// Builds native WEM records from editable semantic values.
pub fn build(model: &Model<'_>) -> Result<Document<'static, V3>, WriteError> {
    build::build_impl(model, Options::default())
}
/// Reads a model with its requested profile without copying native payload buffers.
pub fn read<'view>(
    document: &'view Document<'_, V3>,
    model: u32,
    profile: r::ProfileId,
) -> Result<Model<'view>, ReadError> {
    read::read_impl(document, model, profile, None)
}

/// Builds a new document using an explicitly selected native ABI.
/// Existing native views retain their own document options.
pub fn build_with_options(
    model: &Model<'_>,
    options: Options,
) -> Result<Document<'static, V3>, WriteError> {
    build::build_impl(model, options)
}

/// Reads one model and resolves its logical material slots for a selected look.
pub fn read_with_look<'view>(
    document: &'view Document<'_, V3>,
    model: u32,
    profile: r::ProfileId,
    look: u32,
) -> Result<Model<'view>, ReadError> {
    read::read_impl(document, model, profile, Some(look))
}
