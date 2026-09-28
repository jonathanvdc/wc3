//! Warcraft III MDX container parsing and writing.
//!
//! MDX files start with `MDLX`, followed by tagged chunks. Each chunk has a
//! four-byte identifier, a little-endian payload length, and its payload.
//! [`Model<V>`] ties a model's version to its chunks and records.
//! [`DynamicModel`] dispatches a file's runtime version to a typed model.
//! Unknown chunks retain their raw payloads; malformed known chunks fail decoding.
//!
//! ```
//! use wc3_mdx::{Model, V800};
//! use wc3_mdx::io::Writable;
//! let model = Model::<V800>::new();
//! // Populate the model
//! let bytes = model.encode().unwrap();
//! ```

extern crate self as wc3_mdx;

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

pub mod io;
pub use io::*;
pub mod animation;
pub(crate) use animation::*;
pub mod geometry;
pub(crate) use geometry::*;
pub mod materials;
pub(crate) use materials::*;
pub mod scene;
pub(crate) use scene::*;
pub mod emitters;
pub(crate) use emitters::*;

pub use wc3_mdx_derive::{Readable, Writable};
mod model;
pub use model::{DynamicModel, Model};
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
