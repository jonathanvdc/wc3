//! Read, edit, and write Warcraft III models and BLP textures.
//!
//! Start with [`model::Model`] for a known format version or [`model::DynamicModel`]
//! for files whose version is determined at runtime. Both use the same model types
//! for binary MDX and text MDL; import the corresponding codec traits to enable I/O.
//! See [`model`] for a complete example and guidance on editing records.
//! [`blp`] reads and writes BLP1 and BLP2 encoded texture containers.

extern crate self as wc3;

/// Warcraft III models and their MDX and MDL codecs.
pub mod model;

/// BLP1 and BLP2 texture containers.
pub mod blp;
