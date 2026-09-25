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
//! use wc3_mdx::{RawChunk, Model, Record};
//! let mut model = Model::new(800);
//! model.push(RawChunk::new(*b"TEST", vec![1, 2, 3]));
//! let bytes = model.encode().unwrap();
//! assert_eq!(Model::decode(&bytes, 800).unwrap(), model);
//! ```

pub mod chunks;
pub use chunks::*;
mod error;
pub use error::Error;
mod cursor;
mod model;
pub use model::Model;
mod record;
pub use record::Record;

mod animation;
pub use animation::{AnimationTrack, Keyframe, TrackValueKind};
mod attachment;
pub use attachment::Attachment;
mod collision;
pub use collision::{CollisionKind, CollisionShape};
mod camera;
pub use camera::Camera;
mod event;
pub use event::EventObject;
mod face_fx;
pub use face_fx::FaceFx;
mod geoset;
pub use geoset::{Geoset, GeosetExtent};
mod geoset_animation;
pub use geoset_animation::{GeosetAnimation, GeosetAnimationFlags};
mod light;
pub use light::Light;
mod material;
pub use material::{Layer, LayerShadingFlags, LayerTextureSlot, Material, MaterialRenderFlags};
mod model_info;
pub use model_info::ModelInfo;
mod node;
pub use node::{Bone, Node, NodeFlags};
mod popcorn;
pub use popcorn::PopcornEmitter;
mod particle;
pub use particle::ParticleEmitter;
mod particle2;
pub use particle2::{Particle2Fields, Particle2Frames, ParticleEmitter2};
mod ribbon;
pub use ribbon::{RibbonEmitter, RibbonFields};
mod sequence;
pub use sequence::{Sequence, SequenceFlags};
mod simple_chunks;
mod sized_node;
mod texture;
pub use texture::{Texture, TextureFlags};
mod texture_animation;
pub use texture_animation::TextureAnimation;
mod utils;
mod validation;
