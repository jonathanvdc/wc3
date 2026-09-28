//! Streaming Warcraft III MDL primitives over resident UTF-8 input.
//!
//! The lexer borrows token spelling and the parser keeps one token of lookahead;
//! neither constructs an AST or allocates. Strings are literal, including
//! backslashes and CR/LF. Only `//` comments are accepted. Numeric readers check
//! ranges; f32 supports case-insensitive nan/inf/-inf. The writer uses tabs and
//! shortest round-tripping floats. NaN payload bits have no text representation.
//!
//! Handwritten record codecs currently cover Bitmap (`Texture`), Anim
//! (`Sequence`), Duration (`GlobalSequence`) and anonymous `PivotPoint` entries.
//! This is not yet a whole-model MDL codec or an MDL derive implementation.
//!
//! ```
//! use wc3_mdx::materials::Texture;
//! use wc3_mdx::mdl::{MdlRead, MdlWrite, MdlWriter};
//!
//! let texture = Texture::parse_mdl(r#"Bitmap { Image "Textures\Armor.blp", WrapWidth, }"#)?;
//! let mut bytes = Vec::new();
//! let mut writer = MdlWriter::new(&mut bytes);
//! texture.write_mdl(&mut writer)?;
//! writer.finish()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
mod error;
pub use error::{Diagnostic, ReadError, ReadErrorKind, Span, WriteError};
mod lexer;
pub use lexer::{Lexer, Token, TokenKind};
mod parser;
pub use parser::{Block, Counted, Field, Parser};
mod writer;
pub(crate) use writer::fixed_text;
pub use writer::MdlWriter;

use std::io::Write;

/// Reads one value directly into its final representation.
pub trait MdlRead: Sized {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, ReadError>;
    /// Reads one value and rejects trailing input.
    fn parse_mdl(source: &str) -> Result<Self, ReadError> {
        let mut parser = Parser::new(source);
        let value = parser.read()?;
        parser.finish()?;
        Ok(value)
    }
}

/// Writes directly to a sink. Record codecs reject fields they cannot represent,
/// including unknown flag bits and opaque binary padding. Output may be partial
/// on error. Finite floats round-trip exactly; NaNs retain only their NaN class.
pub trait MdlWrite {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError>;
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
