//! Warcraft III model data shared by the MDX and MDL codecs.
//!
//! Use [`Model<D>`](crate::model::Model) when the dialect is known and [`DynamicModel`](crate::model::DynamicModel) when
//! reading files of different versions. Version markers such as [`V800`](crate::model::V800) and
//! [`V1100`](crate::model::V1100) are standard dialects: they select record layouts and
//! leave application chunks opaque. Extended dialects select an application codec
//! alongside one of those same layouts.
//!
//! # Read, edit, and write
//!
//! Import [`mdx::Read`](crate::model::mdx::Read) / [`mdx::Write`](crate::model::mdx::Write) for binary I/O and [`mdl::Read`](crate::model::mdl::Read) /
//! [`mdl::Write`](crate::model::mdl::Write) for text I/O.
//!
//! ```
//! use wc3::model::{Model, V800};
//! use wc3::model::mdl::{Read as _, Write as _};
//! use wc3::model::mdx::Write as _;
//!
//! let source = r#"Version { FormatVersion 800, } Model "Example" {}"#;
//! let mut model = Model::<V800>::decode_mdl(source)?;
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
//! Collection getters return owned copies in file order. After editing a copy,
//! pass it to the corresponding setter to replace the model's collection. To
//! preserve chunk organization while editing in place, match the typed variants
//! in [`Model::chunks`](crate::model::Model::chunks) or use [`Model::chunk_mut`](crate::model::Model::chunk_mut).
//!
//! Names and paths use [`FixedText`](crate::model::FixedText); use `set_text()` for a validated replacement.
//! Ordinary record fields are public. Geometry methods coordinate arrays that
//! must agree in size, and animation-track constructors select the interpolation
//! and keyframe types together.
//!
//! # Resource references
//!
//! [`Model::resources`](crate::model::Model::resources) and
//! [`DynamicModel::resources`](crate::model::DynamicModel::resources) enumerate literal paths
//! and replaceable IDs with typed chunk/record/field locations. Repeated references,
//! duplicate chunks, and empty path fields remain visible. The [`resources`](crate::model::resources) module
//! explains transactional rewriting through `rewrite_resources`, including byte
//! preservation and caller-owned resolution and packaging policy.
//!
//! # Application chunks
//!
//! A [`ModelDialect`](crate::model::ModelDialect) selects a standard [`ModelVersion`](crate::model::ModelVersion) and a [`ModelExtension`](crate::model::ModelExtension)
//! codec. Use [`Extended<V, E>`](crate::model::Extended) to combine a standard version with your
//! application's chunk type, or implement `ModelDialect` for a named dialect.
//! Standard records remain parameterized by the base version, so
//! `Model<Extended<V1800, E>>` accepts ordinary `Geoset<V1800>` and `Material<V1800>`
//! values. The numeric `VERS` field still contains `1800`.
//!
//! Application chunks use the same binary codecs as standard chunks. `Chunk`
//! supplies the tag and payload writer, and `KnownChunk` supplies the payload
//! reader for a fixed tag. Types implementing `KnownChunk`, `Clone`, and `Debug`
//! automatically implement `ModelExtension`, as the following example shows.
//! Payload records can use the `mdx::Read` and `mdx::Write` derives.
//!
//! ```
//! use wc3::model::{Cursor, DynamicModel, Encoder, Extended, Model, Tag, V1800, mdx};
//! use wc3::model::chunks::{Chunk, KnownChunk, ModelChunk};
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
//! impl KnownChunk for Note {
//!     const TAG: Tag = *b"NOTE";
//!     fn decode_payload(input: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
//!         Ok(Self { value: input.read()? })
//!     }
//! }
//!
//! type ApplicationDialect = Extended<V1800, Note>;
//! let mut model = Model::<ApplicationDialect>::new();
//! model.chunks.push(ModelChunk::Extension(Note { value: 42 }));
//! let bytes = model.encode_mdx()?;
//! let dynamic = DynamicModel::<Note>::decode_mdx(&bytes, 800)?;
//! assert_eq!(dynamic.version(), 1800);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! To support several application chunk kinds, use an enum implementing `Chunk`
//! and `ModelExtension`. Its `read_extension` method dispatches on the input tag,
//! returning a decoded value for recognized tags and `None` for unknown tags.
//! The container preserves unknown chunks and validates decoded payload boundaries
//! and tags. Applications define chunk placement, duplicate rules, and references
//! within custom payloads.
//!
//! Version-dependent codecs can parameterize their chunk types by
//! `V: ModelVersion`, using `V::NUMBER` and versioned records in their implementations.
//! For example, `Extended<V1800, ApplicationChunk<V1800>>` selects version 1800 for
//! both standard records and application chunks. `DynamicModel<E>` uses one
//! extension type across its `Model<Extended<V, E>>` variants and supports the same
//! accessors, resource operations, and `visit_model!` macro as standard models.
//!
//! # Format conversion
//!
//! MDX retains unknown chunks, flag bits, and fixed-text bytes. MDL writes
//! canonical text and rejects data it cannot represent faithfully; it does not
//! preserve comments, formatting, or binary chunk organization. See [`mdl`](crate::model::mdl) for
//! output dialects and text restrictions.
//!
//! [`Model::convert`](crate::model::Model::convert) changes the dialect without modifying the source.
//! Standard records convert between the base versions. Application chunks are encoded
//! by their `Chunk` implementation and decoded by the target extension codec, which may reject
//! an incompatible payload. Unknown chunks are also offered to that codec. A tag the
//! target does not recognize becomes opaque; when the base version changes, the
//! unknown-chunk policy decides whether to reject, preserve, or drop it. At the same
//! base version, opaque bytes are retained. `normalized()` applies these same conversion
//! rules within the original dialect, including re-encoding application chunks.
//! MDL has no representation for application binary chunks and rejects their export.
//! Choose a loss policy explicitly and inspect the returned [`ConversionReport`](crate::model::ConversionReport);
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

pub use crate::visit_model;

mod container;
pub use container::{DynamicModel, Model};
mod model_access;
pub use model_access::{CommonModelAccess, TryModelAccess};
mod versions;
pub use versions::{
    ModelVersion, SupportsEmissiveGain, SupportsFresnel, SupportsLayerShaderTypeId,
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
pub use error::{IoError, ValueError};

mod dialect;
pub use dialect::{Extended, ModelDialect, ModelExtension, NoExtensions};
