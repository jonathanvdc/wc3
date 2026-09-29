//! Read and write Warcraft III models in text MDL format.
//!
//! Import [`Read`] and [`Write`] as `_` to use `decode_mdl()` and `encode_mdl()`.
//! [`crate::model::Model`] checks an expected version;
//! [`crate::model::DynamicModel`] selects it from the `Version` block.
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
//! Output defaults to [`Dialect::Warcraft3`]. Select [`Dialect::HiveWorkshop`] with
//! [`Write::encode_mdl_with_dialect`] or [`MdlWriter::with_dialect`]; the choice
//! applies to all nested records. HiveWorkshop output can represent numeric shader
//! IDs, animations in non-diffuse HD texture slots, raw geoset selection flags,
//! and LOD names that Warcraft III output cannot express.
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
//! padding, and nondefault base values hidden by animation tracks. NaN payload
//! bits are not preserved. Consequently, an MDX–MDL–MDX round trip need not reproduce
//! the original bytes.
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
//! [`MdlWriter`] writes to any standard I/O sink. Errors can leave partial output;
//! use `encode_mdl()` for an owned string before replacing a file.

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
pub use property::{ReadProperty, WriteProperty};
mod value_eq;
pub use value_eq::ValueEq;
mod writer;
pub use writer::MdlWriter;

/// Canonical text syntax selected independently of the binary model version.
/// Readers accept both dialects; writers enforce the selected dialect's limits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Dialect {
    #[default]
    Warcraft3,
    HiveWorkshop,
}

// Re-export the bitfield traits for generated code in downstream crates.
#[doc(hidden)]
pub use bitfield::{BitRange, BitRangeMut};

use std::io::Write as IoWrite;
pub use wc3_derive::{MdlRead as Read, MdlWrite as Write};

/// Reads one value directly into its final representation.
pub trait Read: Sized {
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
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError>;

    /// Encodes one value as UTF-8 text and checks block balance.
    fn encode_mdl(&self) -> Result<String, WriteError> {
        self.encode_mdl_with_dialect(Dialect::Warcraft3)
    }

    /// Encodes using the selected dialect without discarding unrepresentable data.
    fn encode_mdl_with_dialect(&self, dialect: Dialect) -> Result<String, WriteError> {
        let mut writer = MdlWriter::with_dialect(Vec::new(), dialect);
        writer.write(self)?;
        let bytes = writer.finish()?;
        // MdlWriter only writes UTF-8 strings and ASCII formatting.
        Ok(String::from_utf8(bytes).expect("MDL output is valid UTF-8"))
    }
}

/// Stack bitset used by field readers.
/// The supplied bit must identify one field uniquely.
#[derive(Default)]
pub struct Fields(u64);
impl Fields {
    pub fn mark(&mut self, bit: u32, field: Field<'_>) -> Result<(), ReadError> {
        assert!(bit < 64, "field bit must be below 64");
        let mask = 1u64 << bit;
        if self.0 & mask != 0 {
            return Err(ReadError::new(field.span, ReadErrorKind::DuplicateField));
        }
        self.0 |= mask;
        Ok(())
    }
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
