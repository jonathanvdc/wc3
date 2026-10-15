//! Warcraft III model data shared by the MDX and MDL codecs.
//!
//! Use [`Document<D>`](crate::model::formats::mdlx::Document) when the dialect is known and [`DynamicDocument`](crate::model::formats::mdlx::DynamicDocument) when
//! reading files of different versions. Version markers such as [`V800`](crate::model::formats::mdlx::V800) and
//! [`V1100`](crate::model::formats::mdlx::V1100) are standard dialects: they select record layouts and
//! leave application chunks opaque. Extended dialects select an application codec
//! alongside one of those same layouts.
//!
//! # Read, edit, and write
//!
//! Import [`mdx::Read`](crate::model::formats::mdlx::mdx::Read) / [`mdx::Write`](crate::model::formats::mdlx::mdx::Write) for binary I/O and [`mdl::Read`](crate::model::formats::mdlx::mdl::Read) /
//! [`mdl::Write`](crate::model::formats::mdlx::mdl::Write) for text I/O.
//!
//! ```
//! use wc3::model::formats::mdlx::{Document, V800};
//! use wc3::model::formats::mdlx::mdl::{Read as _, Write as _};
//! use wc3::model::formats::mdlx::mdx::Write as _;
//!
//! let source = r#"Version { FormatVersion 800, } Model "Example" {}"#;
//! let mut model = Document::<V800>::decode_mdl(source)?;
//! let mut info = model.model_info().unwrap();
//! info.name.set_text("Renamed")?;
//! model.set_model_info(&info);
//!
//! let binary = model.encode_mdx()?;
//! let text = model.encode_mdl()?;
//! assert!(text.contains("Renamed"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Editing records
//!
//! Getters for borrowed record collections return iterators in file order without
//! allocating.
//! Use `.cloned().collect::<Vec<_>>()` when you need cloned records, or
//! `.collect::<Vec<_>>()` for a vector of references. Edit records in place with
//! named mutable iterators such as [`Document::geosets_mut`](crate::model::formats::mdlx::Document::geosets_mut) or generic
//! [`Document::records_mut`](crate::model::formats::mdlx::Document::records_mut). Both retain duplicate chunks and their order. Use
//! [`Document::as_borrowed`](crate::model::formats::mdlx::Document::as_borrowed) for an independent view whose unchanged buffers borrow
//! the original model. Named record setters consume arrays, vectors, or iterators
//! of records and move their buffers into one replacement chunk, removing
//! duplicate chunks of that kind. [`Document::replace_records`](crate::model::formats::mdlx::Document::replace_records) provides the same
//! replacement operation for a typed collection. Matching [`Document::chunks`](crate::model::formats::mdlx::Document::chunks)
//! or using [`Document::chunk_mut`](crate::model::formats::mdlx::Document::chunk_mut) gives direct access to individual chunks.
//!
//! MDX decoding produces a `Document<'data, D>` borrowing its input byte slice.
//! Geometry arrays and animation keys use [`ZeroVec`](zerovec::ZeroVec) directly to read
//! little-endian values even when the source is unaligned. MDL parsing collects
//! into owned encoded arrays without an intermediate native vector. Use
//! `.iter().collect::<ZeroVec<_>>()` to construct arrays from generated values.
//! [`geometry::Geoset::new`](crate::model::formats::mdlx::geometry::Geoset::new) and the [`animation::Track`](crate::model::formats::mdlx::animation::Track) constructors accept
//! native arrays, vectors, and slices through [`IntoZeroVec`](crate::model::formats::mdlx::IntoZeroVec), encoding them into
//! owned storage. Passing a ZeroVec moves its buffer unchanged, preserving any
//! borrowing. Sorting unordered borrowed track keys creates an owned copy.
//! ZeroVec equality compares encoded bytes, including the exact bit patterns of
//! floating-point values.
//!
//! Editing follows ZeroVec's API: `to_mut_slice()` copies a borrowed array once
//! and exposes ULE elements; `with_mut()` also allows resizing. [`AsULE::to_unaligned`](zerovec::ule::AsULE::to_unaligned)
//! converts native values for those edits. Geometry's individual-value setters
//! perform that conversion for you. Opaque payloads use [`Cow`](std::borrow::Cow)
//! over byte slices. Skin rows use `SkinWeights<u8>` in versions 900–1300 and
//! `SkinWeights<u16>` from version 1400; [`geometry::SkinWeight`](crate::model::formats::mdlx::geometry::SkinWeight) provides common
//! accessors for generic code.
//!
//! `as_borrowed()` also borrows buffers that the original model already owns,
//! making a cheap editable view. `to_owned()` creates an independent deep copy.
//! `into_owned()` copies borrowed buffers and moves owned buffers into a `Document<'static, D>` independent of its source.
//! `Clone` preserves each buffer's ownership state: borrowed buffers remain
//! borrowed, while owned buffers are copied. `mdx::from_reader` buffers the input
//! and returns an owned model. `convert` borrows its source's buffers;
//! `into_convert` consumes its source and moves compatible buffers.
//!
//! ```
//! use wc3::model::formats::mdlx::{Document, V800, V1800, ConversionOptions};
//! use wc3::model::formats::mdlx::mdx::{Read as _, Write as _};
//! use wc3::model::formats::mdlx::chunks::GeosetsChunk;
//! let bytes = Document::<V800>::new().encode_mdx()?;
//! let original = Document::<V800>::decode_mdx(&bytes)?;
//! let mut view = original.as_borrowed();
//! for geoset in view.records_mut::<GeosetsChunk<V800>>() {
//!     geoset.material_id = 0;
//! }
//! let converted = view.into_convert::<V1800>(&ConversionOptions::strict())?;
//! let owned = converted.model.into_owned();
//! assert_eq!(owned.version(), 1800);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Names and paths use [`FixedText`](crate::model::formats::mdlx::FixedText); use `set_text()` for a validated replacement.
//! Ordinary record fields are public. Geometry methods coordinate arrays that
//! must agree in size, and animation-track constructors select the interpolation
//! and keyframe types together.
//!
//! # Resource references
//!
//! [`Document::resources`](crate::model::formats::mdlx::Document::resources) and
//! [`DynamicDocument::resources`](crate::model::formats::mdlx::DynamicDocument::resources) enumerate literal paths
//! and replaceable IDs with typed chunk/record/field locations. Repeated references,
//! duplicate chunks, and empty path fields remain visible. The [`resources`](crate::model::formats::mdlx::resources) module
//! explains transactional rewriting through `rewrite_resources`, including byte
//! preservation and caller-owned resolution and packaging policy.
//!
//! # Application chunks
//!
//! A [`Dialect`](crate::model::formats::mdlx::Dialect) selects a standard
//! [`FormatVersion`](crate::model::formats::mdlx::FormatVersion) and an application chunk type.
//! Use [`Extended<V, E>`](crate::model::formats::mdlx::Extended) to combine a standard version with your
//! application's chunk type, or implement `Dialect` for a named dialect.
//! Standard records remain parameterized by the base version, so
//! `Document<Extended<V1800, E>>` accepts ordinary `Geoset<V1800>` and `Material<V1800>`
//! values. The numeric `VERS` field still contains `1800`.
//!
//! Application chunks use the same binary codecs as standard chunks. `Chunk`
//! supplies the tag and payload writer, and `KnownChunk` supplies the payload
//! reader for a fixed tag. Types implementing `KnownChunk` automatically implement `mdx::Extension`, as the following example shows.
//! Payload records can use the `mdx::Read` and `mdx::Write` derives.
//!
//! ```
//! use wc3::model::formats::mdlx::{Cursor, DynamicDocument, Encoder, Extended, Document, Tag, V1800, mdx};
//! use wc3::model::formats::mdlx::chunks::{Chunk, KnownChunk, ModelChunk};
//! use mdx::{Read as _, Write as _};
//!
//! #[derive(Clone, Debug)]
//! struct Note { value: u32 }
//! impl Chunk for Note {
//!     fn tag(&self) -> Tag { *b"NOTE" }
//!     fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
//!         output.write(&self.value)
//!     }
//! }
//! impl<'data> KnownChunk<'data> for Note {
//!     const TAG: Tag = *b"NOTE";
//!     fn decode_payload(input: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
//!         Ok(Self { value: input.read()? })
//!     }
//! }
//!
//! type ApplicationDialect = Extended<V1800, Note>;
//! let mut model = Document::<ApplicationDialect>::new();
//! model.chunks.push(ModelChunk::Extension(Note { value: 42 }));
//! let bytes = model.encode_mdx()?;
//! let dynamic = DynamicDocument::<Note>::decode_mdx(&bytes, 800)?;
//! assert_eq!(dynamic.version(), 1800);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! To support several application chunk kinds, use an enum implementing `Chunk`
//! and `mdx::Extension`. Its `read_extension` method dispatches on the input tag,
//! returning a decoded value for recognized tags and `None` for unknown tags.
//! The container preserves unknown chunks and validates decoded payload boundaries
//! and tags. Applications define chunk placement, duplicate rules, and references
//! within custom payloads.
//!
//! MDL extensions implement `mdl::Extension` to dispatch block names to ordinary
//! `mdl::Read` codecs, with the parser positioned at the opening identifier.
//! Their `mdl::Write` implementations write complete blocks. The model writer
//! emits these blocks after the standard collections, preserving extension order.
//!
//! ```
//! use wc3::model::formats::mdlx::{Extended, Document, V800, mdl};
//! use mdl::{Read as _, Write as _};
//!
//! #[derive(Clone, Debug, mdl::Read, mdl::Write)]
//! #[mdl(block = "Note")]
//! struct Note {
//!     #[mdl(property = "Value")]
//!     value: u32,
//! }
//! impl mdl::Extension for Note {
//!     fn read_extension(name: &str, parser: &mut mdl::Parser<'_>) -> Result<Option<Self>, mdl::ReadError> {
//!         match name {
//!             "Note" => parser.read().map(Some),
//!             _ => Ok(None),
//!         }
//!     }
//! }
//!
//! let source = r#"Version { FormatVersion 800, } Model "Example" {} Note { Value 42, }"#;
//! let model = Document::<Extended<V800, Note>>::decode_mdl(source)?;
//! assert!(model.encode_mdl()?.contains("Value 42,"));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Each format's model codecs require its own extension traits; application values
//! can support MDX, MDL, or both.
//!
//! Version-dependent codecs can parameterize their chunk types by
//! `V: FormatVersion`, using `V::NUMBER` and versioned records in their implementations.
//! For example, `Extended<V1800, ApplicationChunk<V1800>>` selects version 1800 for
//! both standard records and application chunks. `DynamicDocument<E>` uses one
//! extension type across its `Document<Extended<V, E>>` variants and supports the same
//! accessors, resource operations, and `visit_document!` macro as standard models.
//!
//! # Format conversion
//!
//! MDX retains unknown chunks, flag bits, and fixed-text bytes. MDL writes
//! canonical text and rejects data it cannot represent faithfully; it does not
//! preserve comments, formatting, or binary chunk organization. See [`mdl`](crate::model::formats::mdlx::mdl) for
//! output dialects and text restrictions.
//!
//! [`Document::convert`](crate::model::formats::mdlx::Document::convert) changes the dialect without modifying the source.
//! Standard records convert between the base versions. Application chunks are encoded
//! by their `Chunk` implementation and decoded by the target extension codec, which may reject
//! an incompatible payload. Unknown chunks are also offered to that codec. A tag the
//! target does not recognize becomes opaque; when the base version changes, the
//! unknown-chunk policy decides whether to reject, preserve, or drop it. At the same
//! base version, opaque bytes are retained. `normalized()` applies these same conversion
//! rules within the original dialect, including re-encoding application chunks.
//! MDL exports application chunks through their `mdl::Write` implementations.
//! Choose a loss policy explicitly and inspect the returned [`ConversionReport`](crate::model::formats::mdlx::ConversionReport);
//! changing a version does not guarantee identical rendering in the game.

/// A three-dimensional vector in MDX coordinates.
pub type Vec3 = [f32; 3];
/// A rotation quaternion in `[x, y, z, w]` order.
///
/// This is an alias for `[f32; 4]`. Rotation interpolation assumes unit
/// quaternions; the identity rotation is `[0.0, 0.0, 0.0, 1.0]`.
pub type Quaternion = [f32; 4];
/// An RGB color.
pub type Color = [f32; 3];
/// An MDX format version number.
pub type Version = u32;
/// A four-byte MDX chunk or track identifier.
pub type Tag = [u8; 4];

pub mod chunks;
pub(crate) use chunks::*;

pub mod mdx;
pub use mdx::{Cursor, Encoder, FixedText, SizeMarker};
pub mod animation;
pub mod mdl;
pub(crate) use animation::*;
pub mod geometry;
pub(crate) use geometry::*;
pub mod materials;
pub mod resources;
pub(crate) use materials::*;
pub mod scene;
pub(crate) use scene::*;
pub mod emitters;
pub(crate) use emitters::*;

pub use crate::visit_document;

mod container;
pub use container::{Document, DynamicDocument};
mod model_access;
pub use model_access::{CommonModelAccess, TryModelAccess};
mod versions;
pub use versions::{
    FormatVersion, SupportsEmissiveGain, SupportsFresnel, SupportsLayerShaderTypeId,
    SupportsLayerTextureSlots, SupportsLightFalloff, SupportsLightShadowCasting,
    SupportsLightShadowIntensity, SupportsMaterialShaderPath, SupportsReforgedChunks, V1000, V1100,
    V1200, V1300, V1400, V1600, V1800, V800, V900,
};

mod conversion;
pub use conversion::{
    Conversion, ConversionError, ConversionIssue, ConversionIssueKind, ConversionOptions,
    ConversionReport, LossPolicy, UnknownChunkPolicy,
};

mod error;
pub use crate::model::common::IoError;
pub use error::ValueError;

mod dialect;
pub use dialect::{Dialect, Extended, NoExtensions};

pub use zerovec::{ule::AsULE, ZeroVec};

pub use crate::model::common::IntoZeroVec;

mod ownership;
pub use crate::model::common::{Borrowed, Detached, IntoOwned};
pub use wc3_derive::ModelPartialEq;
