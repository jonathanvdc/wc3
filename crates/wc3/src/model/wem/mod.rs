//! Indexed editable-model documents with independently revised native sections.
//!
//! The file envelope identifies WEM generation three. Each section carries its
//! own layout revision; native material payloads retain their source profile.
//! Encoded sections borrow their input and allocate replacement bytes on edits.
pub mod codec;
mod references;
pub use references::*;
mod error;
pub use codec::{
    Array, Bool, Decoder, Encoder, InlineList, InlineOption, NativeAbi, Read, RecordLayout,
    Validate, Write,
};
pub use error::{ReadError, ReadErrorKind, WriteError};
pub use wc3_derive::{
    WemRead as Read, WemRecordLayout as RecordLayout, WemValidate as Validate,
    WemVisitReferences as VisitReferences, WemWrite as Write,
};
mod document;
pub mod records;
pub use crate::model::common::{Borrowed, IntoOwned};
pub use document::{Dialect, Document, Options, RawDocument, Section, SectionId, V3};

mod io;
pub use io::{from_reader, from_reader_with_options, raw_from_reader, to_writer};

/// Semantic model access and authoring helpers.
pub mod semantic;

/// Derives exact scalar enum wire-code conversions.
pub use wc3_derive::WemValue as Value;

/// Native envelope and reference metadata flag words.
pub use records::{ProfileMask, ReferenceFlags};
