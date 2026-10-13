//! Indexed M3 model and animation documents with independently versioned sections.
//!
//! MD33 and MD34 select the reference encoding. Each index entry selects its own
//! record revision. Borrowed payloads remain in their native representation until
//! accessed or edited; section identities remain stable across writes.
//!
//! ```
//! use wc3::model::m3::{Document, Md34, Model, Ref};
//! use wc3::model::m3::records::{ModelRecord, V23};
//!
//! let mut document = Document::<Md34>::new();
//! let root = document.append(&[ModelRecord::<V23>::default()])?;
//! document.set_root(Ref::<ModelRecord<V23>>::from_section(1, root.index()));
//! let model = Model::from_document(document)?;
//! let bytes = model.write()?;
//! let decoded = Model::<Md34>::read(&bytes)?;
//! assert_eq!(decoded.root_record()?.revision(), 23);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ```
//! use wc3::model::m3::vertices::{Layout, Packed, Vertices};
//!
//! type PositionUv = Packed<1>;
//! let bytes = vec![0; PositionUv::STRIDE];
//! let mut vertices = Vertices::<_, PositionUv>::new(bytes)?;
//! vertices.vertex_mut(0).unwrap().set_position([1.0, 2.0, 3.0]);
//! vertices.vertex_mut(0).unwrap().set_uv::<0>([1024, 2048]);
//! let vertex = vertices.vertex(0).unwrap();
//! assert_eq!(vertex.position(), [1.0, 2.0, 3.0]);
//! assert_eq!(vertex.uv::<0>(), [1024, 2048]);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! Native field widths are selected by section dialects:
//!
//! ```compile_fail
//! use wc3::model::m3::records::{Region, V2};
//! let mut region = Region::<V2>::default();
//! region.first_vertex = u32::MAX; // Revision 2 stores this field as u16.
//! ```
//!
//! Vertex access also requires the field in the selected packed layout:
//!
//! ```compile_fail
//! use wc3::model::m3::vertices::{Layout, Packed, Vertices};
//! type FirstUv = Packed<1>;
//! let bytes = vec![0; FirstUv::STRIDE];
//! let vertices = Vertices::<_, FirstUv>::new(bytes).unwrap();
//! let _ = vertices.vertex(0).unwrap().uv::<1>(); // Only channel zero is present.
//! ```
pub mod codec;
mod dialect;
pub use dialect::*;
mod error;
pub use error::{ReadError, WriteError};
mod references;
pub use codec::{Array, Read, RecordLayout, Validate, Write};
pub use references::*;
pub use wc3_derive::{
    M3Read as Read, M3RecordLayout as RecordLayout, M3Validate as Validate,
    M3VisitReferences as VisitReferences, M3Write as Write,
};
mod document;
pub mod records;
pub use document::{Document, Model, Section, SectionId};
mod bundle;
pub mod vertices;
pub use bundle::*;
mod dynamic;
pub use crate::model::common::{Borrowed, IntoOwned};
pub use dynamic::{from_reader, from_reader_dynamic, to_writer, DynamicModel};
mod resources;
pub use resources::*;
mod root;
pub use crate::visit_m3_root;
pub use root::RootRecord;
mod profile;
pub use profile::{Beta, CreationProfile, HeroesOfTheStorm, StarCraft2};

/// Derives exact scalar enum wire-code conversions.
pub use wc3_derive::M3Value as Value;
