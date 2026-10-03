//! Read and write Warcraft III models in text MDL format.
//!
//! Import [`Read`] and [`Write`] as `_` to use `decode_mdl()` and `encode_mdl()`.
//! [`crate::model::Model`] checks an expected version;
//! [`crate::model::DynamicModel`] selects it from the `Version` block.
//! Models also implement `FromStr`: use `source.parse::<Model<V800>>()` or
//! `source.parse::<DynamicModel>()`.
//! Whole-model input requires `Version` first and a `Model` block.
//!
//! ```
//! use wc3::model::{DynamicModel, mdl};
//! use wc3::model::mdl::{Read as _, Write as _};
//!
//! let source = r#"Version { FormatVersion 800, } Model "Example" {}"#;
//! let model = DynamicModel::decode_mdl(source)?;
//! let text = model.encode_mdl()?;
//! let hive = model.encode_mdl_with_dialect(mdl::Dialect::HiveWorkshop)?;
//! assert_eq!(model.version(), 800);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Choosing a dialect
//!
//! Readers accept Warcraft III and HiveWorkshop spellings, including mixed input.
//! HiveWorkshop output omits material `SortPrimsNearZ` and layer `WrapWidth`,
//! `WrapHeight`, `Unlit`, `BackFacesForShadows`, and `AmbientOcclusion`.
//! Use Warcraft III output to retain those flags.
//! Output defaults to [`Dialect::Warcraft3`]. Select [`Dialect::HiveWorkshop`] with
//! [`Write::encode_mdl_with_dialect`] or [`Writer::with_dialect`]; the choice
//! applies to all nested records. HiveWorkshop output can represent numeric shader
//! IDs, animations in non-diffuse HD texture slots, and raw geoset selection flags
//! that Warcraft III output cannot express.
//!
//! # Round trips and errors
//!
//! Output preserves represented values, object IDs, and references, while
//! canonicalizing field order and model collections. Comments and source formatting
//! are discarded. Readers reject unknown names, duplicate assignments, incorrect
//! counts, and invalid values. Use [`ReadError::diagnostic`] with the original source
//! for line/column diagnostics.
//!
//! Writers return an error for data that text cannot faithfully represent, such as
//! opaque binary chunks, unknown flag bits, non-UTF-8 fixed text, nonzero text
//! padding. Animation tracks replace their base values in output; reading the
//! output restores those bases to their defaults. NaN payload bits are not
//! preserved. Consequently, an MDX–MDL–MDX round trip need not reproduce the original bytes.
//!
//! Strings are literal: backslashes and line breaks are preserved, and there are
//! no escape sequences. Quotes and NUL cannot be written inside strings. Only
//! `//` comments are accepted. Finite floats round-trip exactly, including negative
//! zero.
//!
//! # Reading or writing part of a file
//!
//! [`Parser::read`] consumes one record from a larger input; `decode_mdl()` requires
//! exactly one value with no trailing input. [`Parser::counted`] reads list entries
//! one at a time. Exhaust the list or call its `finish()` method to validate unread
//! entries and the declared count; dropping it does not validate the remainder.
//!
//! [`Writer`] writes to any standard I/O sink. Errors can leave partial output;
//! use `encode_mdl()` for an owned string before replacing a file.

//!
//! # Standard I/O
//!
//! [`from_reader`] buffers the entire source through EOF and rejects trailing
//! input. [`to_writer`] streams output and checks block balance.
//! Writers do not flush their sinks; I/O failures may leave partial output.
//! Standard I/O adapters and streaming write methods return
//! [`crate::model::IoError`], separating sink/source failures from codec errors.
//! Owned `encode_mdl()` methods return only [`WriteError`].
//!
//! ```
//! use wc3::model::{mdl, Model, V800};
//!
//! let model = "Version { FormatVersion 800, } Model \"Example\" {}".parse::<Model<V800>>()?;
//! let mut output = Vec::new();
//! mdl::to_writer(&mut output, &model)?;
//! let decoded: Model<V800> = mdl::from_reader(output.as_slice())?;
//! assert_eq!(decoded.version(), 800);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use crate::model::IoError;
mod enumeration;
#[doc(hidden)]
pub use enumeration::enum_names_valid;
mod error;
pub use error::{Diagnostic, ReadError, ReadErrorKind, Span, WriteError};
mod fields;
#[doc(hidden)]
pub use fields::{dispatch_name, field_names_unique, read_mdl_body};
pub use fields::{ReadFields, WriteFields};
mod lexer;
mod model;
pub use lexer::{Lexer, Token, TokenKind};
mod parser;
pub use parser::{Block, Counted, Field, Parser};
mod property;
pub use property::{ReadAnimationProperty, ReadProperty, WriteAnimationProperty, WriteProperty};
mod value_eq;
pub use value_eq::ValueEq;
mod writer;
pub(crate) use writer::validate_fixed_text;
pub use writer::Writer;

/// Canonical text syntax selected independently of the binary model version.
/// Readers accept both dialects; writers enforce the selected dialect's limits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Dialect {
    #[default]
    /// Warcraft III syntax, preserving supported game material flags.
    Warcraft3,
    /// HiveWorkshop syntax, with its alternate names and representability limits.
    HiveWorkshop,
}

// Re-export the bitfield traits for generated code in downstream crates.
#[doc(hidden)]
pub use bitfield::{BitRange, BitRangeMut};
use std::io::Write as IoWrite;
pub use wc3_derive::{MdlRead as Read, MdlWrite as Write};

/// Reads one value directly into its final representation.
pub trait Read: Sized {
    /// Reads one value at the current parser position.
    /// Implementations own their value framing; failed reads may consume input.
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, ReadError>;
    /// Reads one value and rejects trailing input.
    fn decode_mdl(source: &str) -> Result<Self, ReadError> {
        let mut parser = Parser::new(source);
        let value = parser.read()?;
        parser.finish()?;
        Ok(value)
    }
}

/// Writes directly to a sink. Record codecs reject fields they cannot represent,
/// including unknown flag bits and opaque binary padding. Output may be partial
/// on error. Finite floats round-trip exactly; NaNs retain only their NaN class.
pub trait Write {
    /// Streams one value using the writer's dialect. Errors may leave partial output.
    fn write_mdl<W: IoWrite>(&self, writer: &mut Writer<W>) -> Result<(), IoError<WriteError>>;

    /// Encodes one value as UTF-8 text and checks block balance.
    fn encode_mdl(&self) -> Result<String, WriteError> {
        self.encode_mdl_with_dialect(Dialect::Warcraft3)
    }

    /// Encodes using the selected dialect and checks block balance.
    /// Rejects unrepresentable values, except the material flags explicitly
    /// omitted by HiveWorkshop output as described in the module documentation.
    fn encode_mdl_with_dialect(&self, dialect: Dialect) -> Result<String, WriteError> {
        let mut writer = Writer::with_dialect(Vec::new(), dialect);
        writer.write(self).map_err(buffer_error)?;
        let bytes = writer.finish()?;
        // Writer only writes UTF-8 strings and ASCII formatting.
        Ok(String::from_utf8(bytes).expect("MDL output is valid UTF-8"))
    }
}

/// Stack bitset used by field readers.
/// The supplied bit must identify one field uniquely.
#[derive(Default)]
pub struct Fields(u64);
impl Fields {
    /// Marks a field as present, rejecting a duplicate at the field's span.
    ///
    /// # Panics
    ///
    /// Panics if `bit` is 64 or greater.
    pub fn mark(&mut self, bit: u32, field: Field<'_>) -> Result<(), ReadError> {
        assert!(bit < 64, "field bit must be below 64");
        let mask = 1u64 << bit;
        if self.0 & mask != 0 {
            return Err(ReadError::new(field.span, ReadErrorKind::DuplicateField));
        }
        self.0 |= mask;
        Ok(())
    }
    /// Checks that a required field was marked, reporting absence at `span`.
    ///
    /// # Panics
    ///
    /// Panics if `bit` is 64 or greater.
    pub fn require(&self, bit: u32, name: &'static str, span: Span) -> Result<(), ReadError> {
        assert!(bit < 64, "field bit must be below 64");
        if self.0 & (1u64 << bit) == 0 {
            Err(ReadError::new(span, ReadErrorKind::MissingField(name)))
        } else {
            Ok(())
        }
    }
}

pub(crate) fn is_zero(value: &u32) -> bool {
    *value == 0
}
pub(crate) fn is_positive_zero(value: &f32) -> bool {
    value.to_bits() == 0
}

mod io;
pub use io::{from_reader, to_writer, to_writer_with_dialect};

fn buffer_error(error: IoError<WriteError>) -> WriteError {
    match error {
        IoError::Codec(error) => error,
        IoError::Io(_) => unreachable!("writing to Vec cannot fail"),
    }
}
