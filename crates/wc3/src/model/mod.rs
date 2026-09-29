//! Warcraft III model data shared by the MDX and MDL codecs.
//!
//! Use [`Model<V>`](crate::model::Model) when the format version is known and [`DynamicModel`](crate::model::DynamicModel) when
//! reading files of different versions. Version markers such as [`V800`](crate::model::V800) and
//! [`V1100`](crate::model::V1100) ensure that records added to a typed model use a compatible layout.
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
//! # Format conversion
//!
//! MDX retains unknown chunks, flag bits, and fixed-text bytes. MDL writes
//! canonical text and rejects data it cannot represent faithfully; it does not
//! preserve comments, formatting, or binary chunk organization. See [`mdl`](crate::model::mdl) for
//! output dialects and text restrictions.
//!
//! [`Model::convert`](crate::model::Model::convert) changes the format version without modifying the source.
//! Choose a loss policy explicitly and inspect the returned [`ConversionReport`](crate::model::ConversionReport);
//! changing a version does not guarantee identical rendering in the game.

/// A three-dimensional vector in MDX coordinates.
pub type Vec3 = [f32; 3];
/// A four-component vector, used for rotation tracks.
pub type Vec4 = [f32; 4];
/// An RGB color.
pub type Color = [f32; 3];
/// An MDX format version number.
pub type Version = u32;
/// A four-byte MDX chunk or track identifier.
pub type Tag = [u8; 4];

pub mod chunks;
pub(crate) use chunks::*;

pub mod mdx;
pub use mdx::{Cursor, Encoder, FixedText, ReadError, SizeMarker, ValueError, WriteError};
pub mod animation;
pub mod mdl;
pub(crate) use animation::*;
pub mod geometry;
pub(crate) use geometry::*;
pub mod materials;
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
