//! World model root and group documents with borrowed native records.
//!
//! WMO's version number does not identify every client layout. Select a dialect
//! from client provenance, or use [`DynamicFile`](crate::model::wmo::DynamicFile) with an explicit [`Profile`](crate::model::wmo::Profile).
//! Root and group documents retain chunk order and unknown bytes. Their required
//! headers cannot be removed through the typed editing API.
pub mod codec;
mod dialect;
pub use dialect::*;
mod error;
pub use error::{ReadError, WriteError};
mod io;
pub use io::{
    from_reader, from_reader_dynamic, from_reader_group, to_writer, Read, RecordLayout, Validate,
    Write,
};
pub use wc3_derive::{
    WmoRead as Read, WmoRecordLayout as RecordLayout, WmoValidate as Validate, WmoWrite as Write,
};
mod document;
pub mod group;
pub mod records;
pub mod root;
pub use document::{File, Group, GroupFraming, RawChunk, RawDocument, Root};
mod dynamic;
pub use crate::visit_wmo_file;
pub use dynamic::{DynamicDialect, DynamicFile, Profile};
mod bundle;
pub use bundle::{Bundle, EncodedBundle, GroupKey};
pub mod resources;
pub use crate::model::common::{Borrowed, FileKey, IntoOwned, IntoZeroVec, ZeroVec};
pub use resources::{Dependency, DependencyKind, TextureReference};

mod validation;
mod validation_group;

/// Historical monolithic version-14 and split version-16 documents.
pub mod legacy;
