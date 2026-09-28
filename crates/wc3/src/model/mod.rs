//! Warcraft III MDX container parsing and writing.
//!
//! MDX files start with `MDLX`, followed by tagged chunks. Each chunk has a
//! four-byte identifier, a little-endian payload length, and its payload.
//! [`Model<V>`](crate::model::Model) ties a model's version to its chunks and records.
//! [`DynamicModel`](crate::model::DynamicModel) dispatches a file's runtime version to a typed model.
//! Unknown chunks retain their raw payloads; malformed known chunks fail decoding.
//!
//! Plain records expose their scalar data, embedded records, and ordinary vectors
//! as public fields. Names and paths use [`FixedText`](crate::model::FixedText): edit them with `set_text`
//! or replace their exact bytes with `from_bytes`. Methods provide computed views,
//! version-dependent properties, and edits that preserve structural invariants.
//! Model collection getters return owned records collected across chunks; edit
//! [`Model::chunks`](crate::model::Model::chunks) directly or use [`Model::chunk_mut`](crate::model::Model::chunk_mut) for in-place changes.
//!
//! ```
//! use wc3::model::{Model, V800};
//! use wc3::model::mdx::Write;
//! let model = Model::<V800>::new();
//! // Populate the model
//! let bytes = model.encode_mdx().unwrap();
//! ```

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
