//! Warcraft III MDX container parsing and writing.
//!
//! MDX files start with `MDLX`, followed by tagged chunks. Each chunk has a
//! four-byte identifier, a little-endian payload length, and its payload.
//! Known chunks are decoded and encoded from their typed values. Unknown and
//! malformed chunks retain their raw payloads.
//! Use [`Model::version`] and
//! [`Model::set_version`] for the `VERS` chunk, and [`Model::chunks`] or
//! [`Model::chunk_mut`] for other chunks.
//!
//! ```
//! use wc3_mdx::Model;
//! use wc3_mdx::io::Encodable;
//! let mut model = Model::new(800);
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
pub use model::Model;
mod validation;
