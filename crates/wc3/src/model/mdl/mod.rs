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
//! CameraTrack also supports visibility and depth-of-field tracks; scalar
//! DOFDistance/FocalLength/FStop become time-zero stepped keys and write using
//! the keyed spellings to avoid the client's scalar swap defect.
//! GeosetAnim (`GeosetAnimation`) supports static or animated Alpha and Color.
//! Missing channels keep full alpha and white color; a Color property enables
//! the color-use flag. Writers reject unknown flags, duplicate tracks, and base
//! values or color-use states that the text representation would discard.
//! Helper (`Node`), Bone, Attachment, Material and Layer records are supported.
//! Material/Layer layouts use the existing version-selected storage. Layer
//! texture bindings support static slots and animated diffuse IDs; writers
//! reject non-diffuse animations and noncanonical binary texture-track order.
//! Light, EventObject and CollisionShape records are also supported. Light
//! fields follow their version-selected storage. ShadowIntensity supports only
//! the static form: no corresponding binary animation tag has been verified.
//! Classic ParticleEmitter and RibbonEmitter records are also supported,
//! including exact emitter flags, unsigned ribbon fields and track-only
//! visibility. Ribbon TextureSlot accepts the spec's bare scalar form and writes
//! canonical static properties. Writers reject hidden bases and unrepresentable
//! flag bits. ParticleEmitter's resource path occupies all 260 binary bytes.
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
//! `property = "Name", delegate` uses [`ReadProperty`] / [`WriteProperty`] instead of
//! value-level `Read` / `Write`. The record still dispatches the name, rejects
//! duplicates and unknown fields, and follows `write_order`. The field codec
//! reads the payload after the name (including its punctuation), resolves a
//! missing property, and writes or omits the entire property including framing.
//! Writer validation runs before any record output. No field `Default` or
//! `ValueEq` bound is added. A recognized but unavailable property can report
//! `ReadErrorKind::UnsupportedField` against the supplied name span.
//!
//! These codecs own defaults, requirements and omission, independently of a
//! container default; field `default`, `required`, `skip_if`, `read_with`, and
//! `write_with` cannot be combined with `delegate`. The initial form
//! delegates one ordinary body name; static/animated channel delegation is a
//! separate extension. Multi-name bodies use flattening, without version checks.
//! Version-selected field types can implement these interfaces without any
//! version metadata in the record or derive. `Option<T>` already implements
//! them for ordinary optional values:
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
//! Derives preserve generics and existing where clauses, adding codec and
//! Default bounds only for fields that use them. They support up to 64 body
//! names per group (each mapped flag counts separately). General tuple structs
//! remain handwritten; linked track collections use `tracks`.
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
//! Their types are checked against the actual struct on read and write; no wire
//! record or conversion is created. Projected animatable members link directly
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
//! The getter returns a BitRange/BitRangeMut<u32> value; the setter stores it.
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
//! collection getters may return borrowed slices. Structural getters may return
//! a borrowed view implementing WriteFields with the same State type as the
//! declared read type. Getters must be stable for an unchanged record.
//!
//! Reading first reconstructs every real member, then applies virtual hooks
//! in their metadata order, then runs presence hooks, after_read and validation.
//! Input field order does not change hook order. Custom hooks enforce any
//! mapping-specific checks; they can use earlier hooks' reconstructed storage.
//! Virtual scalar fields use explicit defaults rather than container defaults;
//! virtual packed flags start at zero. Headers and skipped fields cannot be
//! virtual. Light and Layer derive directly using these accessors, so writing
//! borrows existing animation tracks and texture slots without cloning them.
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
//! into a `Vec<T>`, checking the declared count without preallocating from it.
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
//! names are checked for valid identifiers and duplicates without allocation
//! before reading or writing. Default animation track groups use delegated
//! variants with these associated-constant names.
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
//! The omission decision is evaluated once and reused when writing. Checked
//! field types require ValueEq: floats compare by bits, preserving signed zero,
//! while all NaNs compare alike because MDL preserves only their class. Arrays
//! compare component by component. Custom value types with codec hooks can
//! implement ValueEq using the equality appropriate to those hooks.
//! `validate_write` remains available for record-specific constraints and opaque
//! skipped storage whose representation the field attributes do not describe.
//! Animatable fields do not support read_with/write_with value hooks. Static
//! properties support those hooks with the same comma framing as properties.
//!
//! Packed mappings use nonzero single-bit u32 masks. Storage can be `u32` or
//! any type implementing BitRange<u32>; parsing also requires BitRangeMut<u32>
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
pub use lexer::{Lexer, Token, TokenKind};
mod parser;
pub use parser::{Block, Counted, Field, Parser};
mod property;
pub use property::{ReadProperty, WriteProperty};
mod value_eq;
pub use value_eq::ValueEq;
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
