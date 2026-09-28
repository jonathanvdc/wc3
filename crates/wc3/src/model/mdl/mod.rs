//! Streaming Warcraft III MDL primitives over resident UTF-8 input.
//!
//! The lexer borrows token spelling and the parser keeps one token of lookahead;
//! neither constructs an AST or allocates. Strings are literal, including
//! backslashes and CR/LF. Only `//` comments are accepted. Numeric readers check
//! ranges; f32 supports case-insensitive nan/inf/-inf. The writer uses tabs and
//! shortest round-tripping floats. NaN payload bits have no text representation.
//!
//! Record codecs cover Bitmap (`Texture`), Anim (`Sequence`), Model
//! (`ModelInfo`), Duration (`GlobalSequence`) and anonymous `PivotPoint` entries.
//! Typed animation tracks and TVertexAnim (`TextureAnimation`) records are also
//! supported, including signed frame times, interpolation tangents and optional
//! global sequences. CameraTrack's Read/Write codecs operate in the camera body;
//! its read_mdl_target/write_mdl_target methods operate inside a Target body,
//! whose framing and Position property belong to the enclosing record codec.
//! GeosetAnim (`GeosetAnimation`) supports static or animated Alpha and Color.
//! Missing channels keep full alpha and white color; a Color property enables
//! the color-use flag. Writers reject unknown flags, duplicate tracks, and base
//! values or color-use states that the text representation would discard.
//! Whole-model conversion is not implemented yet.
//!
//! ```
//! use wc3::model::materials::Texture;
//! use wc3::model::mdl::{Read, Write, MdlWriter};
//!
//! let texture = Texture::decode_mdl(r#"Bitmap { Image "Textures\Armor.blp", WrapWidth, }"#)?;
//! let mut bytes = Vec::new();
//! let mut writer = MdlWriter::new(&mut bytes);
//! texture.write_mdl(&mut writer)?;
//! writer.finish()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Deriving codecs
//!
//! `Read` and `Write` derive named-field structs representing named blocks.
//! Parsing matches borrowed field names, rejects duplicates and unknown fields,
//! and constructs the struct without an AST. Writing follows declaration order
//! unless the container specifies `write_order(field_a, field_b, ...)`, listing
//! every body field once. This changes MDL output order without changing MDX
//! storage order. Header arguments retain their declaration order before `{`.
//!
//! ```
//! use wc3::model::FixedText;
//! use wc3::model::mdl;
//! use wc3::model::mdl::Read as _;
//! use wc3::model::mdl::MdlWriter;
//!
//! fn positive_zero(value: &f32) -> bool { value.to_bits() == 0 }
//!
//! #[derive(mdl::Read, mdl::Write)]
//! #[mdl(block = "Anim")]
//! struct Example {
//!     #[mdl(header)]
//!     name: FixedText<80>,
//!     #[mdl(property = "Interval")]
//!     interval: [u32; 2],
//!     #[mdl(property = "MoveSpeed", default, skip_if = "positive_zero")]
//!     move_speed: f32,
//!     #[mdl(flag = "NonLooping", default)]
//!     non_looping: bool,
//! }
//!
//! let value = Example::decode_mdl(r#"Anim "Stand" { Interval { 0, 1000 }, }"#)?;
//! let mut writer = MdlWriter::new(Vec::new());
//! writer.write(&value)?;
//! let bytes = writer.finish()?;
//! assert!(!std::str::from_utf8(&bytes)?.contains("MoveSpeed"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Each field must declare exactly one form:
//!
//! | Attribute | Representation |
//! | --- | --- |
//! | `header` | Required positional value before `{`. |
//! | `property = "Name"` | Named value followed by a comma. |
//! | `flag = "Name"` | Bare name followed by a comma; the field must be `bool`. |
//! | `flags(Name = 1, Other = 2)` | Bare flags mapped to shared bitfield storage. |
//! | `skip` | No text representation; an explicit default is required. |
//!
//! Properties and boolean flags are required unless annotated `default` (`Default::default()`) or
//! `default = "factory"` (a zero-argument function returning the field's type).
//! Flags support only the bare, false default. A required flag must be present
//! when reading, and must be true when writing. Headers cannot have defaults.
//! `skip_if = "predicate"` accepts `&T` and returns `bool`; it is supported only
//! on properties with defaults. Omission is separate from parsing defaults:
//! choose a predicate that preserves the intended value, including signed zero.
//!
//! `read_with = "function"` has signature `fn(&mut Parser<'_>) -> Result<T,
//! ReadError>`. `write_with = "function"` has signature
//! `fn<W: std::io::Write>(&T, &mut MdlWriter<W>) -> Result<(), WriteError>`. These hooks
//! replace the field's value codec, not its header/property framing. The derive
//! consumes/emits the property comma. Hooks are available on headers and
//! properties, and remove the corresponding Read/Write bound on the field.
//!
//! Container `validate_read = "function"` calls `fn(&Self, Span) -> Result<(),
//! ReadError>` after parsing, with the entire record's byte range. Container
//! `validate_write = "function"` calls `fn(&Self) -> Result<(), WriteError>`
//! before output. Function paths may name associated functions (`Type::check`).
//! Skipped fields are explicitly omitted on write; use validation to reject
//! skipped binary data that must not be discarded. ModelInfo rejects nonzero
//! animation-file data, which has no property in the supported MDL dialect.
//!
//! Derives preserve generics and existing where clauses, adding codec and
//! Default bounds only for fields that use them. They support up to 64 body
//! names (each mapped flag counts separately). Counted collections, enums, and
//! general tuple structs remain handwritten.
//!
//! Packed mappings use nonzero single-bit u32 masks. Storage can be `u32` or
//! any type implementing BitRange<u32>; parsing also requires Default and
//! BitRangeMut<u32>, as provided by SequenceFlags and TextureFlags.
//! Mapped flags are independently optional and initialize storage to zero;
//! repeating a name is an error, and writing unknown bits is an error. Mapping
//! masks and MDL names must be unique. Only an optional bare `default` is allowed;
//! factory defaults, codec hooks, and omission predicates are not supported on
//! a packed mapping. Flags are printed in their mapping declaration order.
//!
//! Single-field tuple structs can represent complete properties or anonymous
//! entries. Both forms consume/emit a trailing comma and support container
//! validation hooks, but not MDL field attributes or `write_order`:
//!
//! ```
//! use wc3::model::mdl;
//! use wc3::model::mdl::Read as _;
//! #[derive(mdl::Read, mdl::Write)]
//! #[mdl(property = "Duration")]
//! struct Duration(u32);
//! #[derive(mdl::Read, mdl::Write)]
//! #[mdl(entry)]
//! struct Point([f32; 3]);
//! assert_eq!(Duration::decode_mdl("Duration 1000,")?.0, 1000);
//! assert_eq!(Point::decode_mdl("{ 1.0, 2.0, 3.0 },")?.0, [1.0, 2.0, 3.0]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Duplicate names and conflicting attributes are compile-time errors:
//!
//! ```compile_fail
//! use wc3::model::mdl;
//! #[derive(mdl::Read)]
//! #[mdl(block = "Example")]
//! struct Duplicate {
//!     #[mdl(property = "Value")] a: u32,
//!     #[mdl(flag = "Value", default)] b: bool,
//! }
//! ```
//!
//! ```compile_fail
//! use wc3::model::mdl;
//! #[derive(mdl::Read)]
//! #[mdl(block = "Example")]
//! struct Conflicting {
//!     #[mdl(header, default)] name: u32,
//! }
//! ```

mod error;
pub use error::{Diagnostic, ReadError, ReadErrorKind, Span, WriteError};
mod lexer;
pub use lexer::{Lexer, Token, TokenKind};
mod parser;
pub use parser::{Block, Counted, Field, Parser};
mod writer;
pub use writer::MdlWriter;

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
        let mut writer = MdlWriter::new(Vec::new());
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
