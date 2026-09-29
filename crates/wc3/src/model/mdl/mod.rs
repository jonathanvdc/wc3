//! Text Warcraft III models, records, and custom codecs.
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
//!
//!
//! ## Deriving codecs
//!
//! Use `#[derive(mdl::Read, mdl::Write)]` and `#[mdl(block = "Name")]` to
//! give a named-field struct a text codec. Annotate each field with its MDL form.
//! Output follows declaration order; `write_order(field_a, field_b, ...)` can
//! choose a different order without changing the Rust struct or MDX layout.
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
//! | `property = "Name", delegate` | Delegate payload, presence, and full output to the field type. |
//! | `static_property = "Name"` | `static Name value,`; never a track. |
//! | `animatable = "Name"` | Static base value or a linked animation track. |
//! | `tracks` | One `Vec<TrackEnum>` holding linked animation tracks. |
//! | `flag = "Name"` | Bare name followed by a comma; the field must be `bool`. |
//! | `flags(Name = 1, Other = 2)` | Bare flags mapped to shared bitfield storage. |
//! | `skip` | No text representation; an explicit default is required. |
//!
//! ## Delegated properties
//!
//! Use `property = "Name", delegate` for optional values or custom properties
//! that need to control their own punctuation, defaults, or omission. Implement
//! [`ReadProperty`] and [`WriteProperty`] on the field type. `Option<T>` already
//! supports this form: an absent property becomes `None`, and `Some(value)`
//! always writes the property.
//!
//! Delegated fields cannot also use `default`, `required`, `skip_if`,
//! `read_with`, or `write_with`; the property codec supplies those policies.
//!
//! ```
//! use wc3::model::mdl;
//! use wc3::model::mdl::{Read as _, Write as _};
//! #[derive(mdl::Read, mdl::Write)]
//! #[mdl(block = "Example")]
//! struct Example {
//!     #[mdl(property = "Value", delegate)]
//!     value: Option<f32>,
//! }
//! let value = Example::decode_mdl("Example {}")?;
//! assert!(value.value.is_none());
//! assert_eq!(value.encode_mdl()?, "Example {\n}\n");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Properties, static properties, and boolean flags are required unless annotated `default` (`Default::default()`) or
//! `default = "factory"` (a zero-argument function returning the field's type).
//! Flags support only the bare, false default. A required flag must be present
//! when reading, and must be true when writing. Headers cannot have defaults.
//! `skip_if = "predicate"` accepts `&T` and returns `bool`; it is supported only
//! on properties, static properties, and animatable fields with defaults. Omission is separate from parsing defaults:
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
//! before output. Container `after_read = "function"` calls
//! `fn(&mut Self, Span) -> Result<(), ReadError>` after field reconstruction and
//! presence hooks, before validation. Use it to restore implicit binary values,
//! such as the object-kind bit supplied by a containing Bone block.
//! Function paths may name associated functions (`Type::check`).
//! Skipped fields are explicitly omitted on write; use validation to reject
//! skipped binary data that must not be discarded. ModelInfo rejects nonzero
//! animation-file data, which has no property in the supported MDL dialect.
//!
//! Derives support generics and add the traits required by each field's codec.
//! A field group may have at most 64 body names, counting each mapped flag.
//!
//! Container `#[mdl(default)]` uses `Self::default()` as the source for omitted
//! body fields, including skipped fields and packed flags. Headers stay required;
//! `#[mdl(required)]` keeps a property, static property, singleton block, or
//! counted list required. Explicit
//! field defaults override the record default. A tracks collection always starts
//! empty, because it records only channels present in the text. Reading requires
//! Self: Default. Writing also requires it when inheriting flag defaults, so it
//! can reject cleared flags that omission would restore to true. Writers also
//! evaluate defaults when checking omitted base values or skip_if properties.
//!
//! ## Projected members and track-only channels
//!
//! `#[mdl(project(#[mdl(property = "Id")] id: u32, ...))]` on a named struct
//! field describes its stored members in the enclosing record's MDL schema.
//! List all members, including explicit `skip` entries for binary-only data.
//! Projected animatable members link directly
//! to the parent's `tracks` collection. Container defaults read the nested
//! member from Self::default(), and write_order lists the containing field name.
//!
//! `#[mdl(tracks, channels(Visibility = "Track::Visibility"))]` adds track-only
//! channels without dummy base fields. The same collection retains source order
//! across ordinary and track-only channels, rejecting duplicates and static
//! spellings for track-only channels.
//!
//! `#[mdl(flatten, extra_flags(get = "Type::flags", set = "Type::set_flags",
//! Extra = 1))]` adds context-specific flags stored inside a flattened record.
//! The getter returns a `BitRange<u32>`/`BitRangeMut<u32>` value; the setter stores it.
//! Read reconstruction ORs these bits into the flattened value. Use the owning
//! record's validation hook to check its implicit kind and unrepresentable bits.
//!
//! ## Fields accessed through storage hooks
//!
//! Container `virtual_fields(...)` describes MDL fields that do not correspond
//! to Rust members. These use the same properties, flags, tracks and structural
//! codecs as stored fields, and are listed individually in `write_order`.
//! Every real Rust member still needs its own MDL annotation; mark storage
//! reconstructed by the hooks as `skip` with a default.
//!
//! Each virtual field needs `get = "function"` and either `slot = "function"`
//! or `set = "function"`. A slot getter has signature `fn(&Self) -> Option<T>`;
//! its slot hook returns `Option<&mut T>`. Both select the same version-specific
//! storage. An explicit field default is required. Missing storage omits the
//! property when writing, and any explicit scalar or animated presence rejects
//! reading with UnsupportedField. Writers also reject tracks for unavailable
//! storage and hidden nondefault bases. The derive supplies all these checks.
//!
//! A custom setter has signature `fn(&mut Self, T, bool, Span) -> Result<(),
//! ReadError>`. The boolean distinguishes explicit presence from defaults,
//! including empty blocks or tracks and explicit default-valued properties.
//! The span covers the entire record. Its getter supplies the write value;
//! collection getters may return borrowed slices or views whose `iter()` yields
//! borrowed entry adapters. Delegated getters may return a borrowed value
//! implementing WriteProperty; their declared read type need not implement it.
//! Structural getters may return
//! a borrowed view implementing WriteFields with the same State type as the
//! declared read type. Getters must be stable for an unchanged record.
//!
//! Reading first reconstructs every real member, then applies virtual hooks
//! in their metadata order, then runs presence hooks, after_read and validation.
//! Input field order does not change hook order. Custom hooks enforce any
//! mapping-specific checks; they can use earlier hooks' reconstructed storage.
//! Virtual scalar fields use explicit defaults rather than container defaults;
//! virtual packed flags start at zero. Headers and skipped fields cannot be
//! virtual.
//!
//! ```
//! use wc3::model::mdl;
//! use wc3::model::mdl::{Read as _, Write as _};
//! #[derive(mdl::Read, mdl::Write)]
//! #[mdl(block = "Example", virtual_fields(
//!     #[mdl(property = "Value", default, get = "Self::value", slot = "Self::value_mut")]
//!     value: u32,
//! ))]
//! struct Example {
//!     #[mdl(skip, default = "Self::initial_storage")]
//!     storage: Option<u32>,
//! }
//! impl Example {
//!     fn initial_storage() -> Option<u32> { Some(0) }
//!     fn value(&self) -> Option<u32> { self.storage }
//!     fn value_mut(&mut self) -> Option<&mut u32> { self.storage.as_mut() }
//! }
//! let value = Example::decode_mdl("Example { Value 7, }")?;
//! assert_eq!(value.storage, Some(7));
//! assert_eq!(Example::decode_mdl(&value.encode_mdl()?)?.storage, Some(7));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Flattened fields, nested blocks, and collections
//!
//! `#[mdl(fields)]` derives `ReadFields`/`WriteFields` for a named-field group
//! without a containing block. Block derives also implement these traits.
//! `#[mdl(flatten)]` embeds a group's headers and properties in the parent.
//! Body fields may interleave while retaining their own defaults, requirements,
//! duplicate markers, tracks, and validation hooks. A parent's container default
//! does not replace a flattened group's defaults. Overlapping names across
//! groups are rejected before reading headers or writing output. Field-group
//! read validation receives the containing record's span.
//!
//! `#[mdl(block = "Target")]` places a field group in a singleton nested block.
//! The attribute supplies the name; the group owns headers and body fields.
//! The block is required unless a field or container default supplies it.
//! Nested blocks have no trailing comma. `write_order` includes structural fields;
//! it orders their body output without reordering flattened header values.
//!
//! `#[mdl(repeated(Translation, Rotation, Scaling))]` dispatches several names
//! into the same collection. `unique_by = "Type::key"` rejects repeated keys on
//! both read and write; without it repeated records may share their names.
//! `#[mdl(repeated = "Anim")]` collects zero or more complete named records into
//! a `Vec<T>`, preserving source order. Each item's `Read`/`Write` owns its name,
//! headers, and punctuation, which must match the declared name.
//! `#[mdl(counted = "Points")]` reads/writes a single `Points N { ... }` list
//! into a `Vec<T>`, checking the declared count.
//! Items own their framing, so scalar/vector entries use an `#[mdl(entry)]`
//! wrapper. Counted lists are required unless given a default, and an empty
//! list is emitted with count zero. Collections reject nonadvancing readers.
//! Structural fields do not support value codec hooks or omission predicates;
//! flatten/repeated fields also initialize themselves rather than taking defaults.
//!
//! ```
//! use wc3::model::{animation::Sequence, geometry::{GeosetExtent, PivotPoint}, mdl};
//! use wc3::model::mdl::{Read as _, Write as _};
//! #[derive(mdl::Read, mdl::Write)]
//! #[mdl(block = "Envelope")]
//! struct Envelope {
//!     #[mdl(flatten)] extent: GeosetExtent,
//!     #[mdl(block = "Target")] target: GeosetExtent,
//!     #[mdl(repeated = "Anim")] sequences: Vec<Sequence>,
//!     #[mdl(counted = "Points")] points: Vec<PivotPoint>,
//! }
//! let record = Envelope::decode_mdl(
//!     "Envelope { Target {} Points 0 {} Anim \"Stand\" { Interval { 0, 1000 }, } }"
//! )?;
//! assert_eq!(record.sequences.len(), 1);
//! assert!(record.points.is_empty());
//! Envelope::decode_mdl(&record.encode_mdl()?)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Enums
//!
//! `#[mdl(value)]` derives a scalar keyword enum with unit variants. Names default
//! to the Rust variant identifier; `#[mdl(name = "DontInterp")]` renames a variant.
//! Scalar enum codecs consume/emit only the identifier. Their containing property
//! or entry owns its comma and indentation. Names are case-sensitive, and unknown
//! names fail at the discriminator's source span. Interpolation uses this form.
//!
//! `#[mdl(tagged)]` derives complete records selected by their leading identifier.
//! Every variant declares its framing explicitly:
//! - `#[mdl(flag = "Ready")] Ready` owns a flag and its comma.
//! - `#[mdl(property = "Duration")] Duration(u32)` owns a scalar property.
//! - `#[mdl(block = "Target")] Target(Position)` uses the payload's field codecs.
//! - `#[mdl(name = "Child", delegate)] Child(Child)` delegates the complete record
//!   to the payload's `Read`/`Write`, leaving the discriminator unconsumed on read.
//!
//! Delegated codecs must use the declared discriminator. Variants cannot have
//! multiple or named payload fields; use a derived record as the single payload.
//! Tagged enums can be items in counted collections, preserving mixed variant
//! order. Generic payloads gain only the bounds required by the chosen framing
//! and derive direction. Enum validation hooks have the same signatures as block
//! hooks and validate the complete enum value before output.
//!
//! Variant name expressions can also be constant paths, including associated
//! constants such as `<Kind as TrackKind>::MDL_NAME`; ordinary Rust paths do not
//! need quotes. Literal names and duplicates are checked by the derive. Constant
//! names are checked for valid identifiers and duplicates before I/O.
//!
//! ```
//! use wc3::model::mdl;
//! use wc3::model::mdl::{Read as _, Write as _};
//! #[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
//! #[mdl(value)]
//! enum Filter { None, #[mdl(name = "Blend")] AlphaBlend }
//! assert_eq!(Filter::decode_mdl("Blend")?, Filter::AlphaBlend);
//! assert_eq!(Filter::None.encode_mdl()?, "None");
//! #[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
//! #[mdl(tagged)]
//! enum Record {
//!     #[mdl(flag = "Ready")] Ready,
//!     #[mdl(property = "Duration")] Duration(u32),
//! }
//! assert_eq!(Record::decode_mdl("Duration 1000,")?, Record::Duration(1000));
//! assert_eq!(Record::Ready.encode_mdl()?, "Ready,\n");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Static and animated properties
//!
//! An `animatable` field requires a field or container default and a `track`
//! path naming a tuple variant in the record's single `#[mdl(tracks)]` Vec.
//! The variant wraps a readable track whose MDL name matches the attribute.
//! Reading `static Name value,` assigns the base field; reading a track appends
//! it to the collection and keeps the default base value. Both forms share one
//! duplicate marker. Missing properties also retain their defaults. Writers
//! emit the track if present, otherwise the static value. The collection is
//! emitted at its position in declaration order or `write_order`, preserving
//! stored track order. Duplicate or unmapped variants are rejected before output.
//!
//! ```
//! use wc3::model::animation::GeosetTrack;
//! use wc3::model::mdl;
//! use wc3::model::mdl::{Read as _, Write as _};
//!
//! #[derive(mdl::Read, mdl::Write)]
//! #[mdl(block = "Example", default)]
//! struct Example {
//!     #[mdl(animatable = "Alpha", track = "GeosetTrack::Alpha")]
//!     alpha: f32,
//!     #[mdl(tracks)]
//!     tracks: Vec<GeosetTrack>,
//! }
//! impl Default for Example {
//!     fn default() -> Self { Self { alpha: 1.0, tracks: Vec::new() } }
//! }
//! let fixed = Example::decode_mdl("Example { static Alpha 0.5, }")?;
//! assert_eq!(fixed.alpha, 0.5);
//! assert!(fixed.tracks.is_empty());
//! let animated = Example::decode_mdl("Example { Alpha 0 { Linear, } }")?;
//! assert_eq!(animated.alpha, 1.0);
//! assert!(!animated.encode_mdl()?.contains("static Alpha"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Optional paired hooks `enabled_if = "predicate"` (`fn(&Self) -> bool`) and
//! `enable_with = "function"` (`fn(&mut Self)`) model properties whose presence
//! enables a flag, such as GeosetAnim Color. The reader calls enable_with after
//! constructing the record, before validate_read, when either form was present.
//! The writer omits the static property when disabled and rejects a track for a
//! disabled property. Enable hooks run in field declaration order.
//!
//! An animatable field may add `animated_only` to reject its static spelling
//! and omit its base on write. Its default then represents the absence of binary
//! base storage; writing rejects a nondefault base even with no track. This is
//! useful for channels such as Light visibility.
//!
//! `bare_static` is an opt-in modifier for animatable fields with a static form.
//! It also accepts `Name value,` as a static alias, while `Name count { ... }`
//! remains a track. Both spellings share duplicate detection and presence hooks.
//! Writers always emit `static Name value,`. Ribbon TextureSlot uses this to
//! accept the scalar spelling in the supplied MDL specification.
//!
//! Writers compare omitted animatable base values and skip_if properties with
//! the defaults the reader restores, rejecting differences before any output.
//! Checked field types require [`ValueEq`]: floats compare by bits, preserving signed zero,
//! while all NaNs compare alike because MDL preserves only their class. Arrays
//! compare component by component. Custom value types with codec hooks can
//! implement ValueEq using the equality appropriate to those hooks.
//! `validate_write` remains available for record-specific constraints and opaque
//! skipped storage whose representation the field attributes do not describe.
//! Animatable fields do not support read_with/write_with value hooks. Static
//! properties support those hooks with the same comma framing as properties.
//!
//! Packed mappings use nonzero single-bit u32 masks. Storage can be `u32` or
//! any type implementing `BitRange<u32>`; parsing also requires `BitRangeMut<u32>`
//! and, without a container default, Default. SequenceFlags and TextureFlags
//! provide these traits.
//! Mapped flags are independently optional and initialize storage to zero
//! unless inheriting a container default. Repeating a name is an error, and writing unknown bits is an error. Mapping
//! masks and MDL names must be unique. Only an optional bare `default` is allowed;
//! factory defaults, codec hooks, and omission predicates are not supported on
//! a packed mapping. `allow_bits = MASK` additionally permits bits represented
//! indirectly by other properties or hooks; those bits are not printed as flags
//! and must not overlap mapped masks. The hooks must account for those bits;
//! validate_write can check constraints not expressed by the field attributes.
//! GeosetAnimation uses allow_bits for its color-use bit. Flags are printed in
//! their mapping declaration order.
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
