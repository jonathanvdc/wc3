//! Logical WEM authoring vocabulary, independent of sections and record revisions.
mod metadata;
/// Structural animation-property selectors for actor vocabularies.
pub mod properties;
pub use metadata::{Metadata, MetadataEntry};
mod node;
pub use node::{DeclaredRole, NodeControls, SupplementalNodeFlags};
mod model;
pub use model::{HierarchyConvention, ModelControls, TestPoseId, TestPoseObject};
mod texture;
pub use texture::{SupplementalTextureFlags, TextureControls, TextureSlot};
mod scene_values;
pub use scene_values::*;
mod forces;
pub use forces::{Bone, Collision, Force, VertexWarp, WarpMode};
/// Playback declarations and recorded simulation conventions.
pub mod animation;
pub use animation::{ClipControls, ControllerControls};
mod geometry;
pub use geometry::{EdgeAttribute, Geometry};
mod attributes;
pub use attributes::{Layer, LayerData, LayerId, LayerObject};
/// Surface bodies, compositing layers, and authored shader inputs.
pub mod materials;
pub use materials::*;
mod scene;
pub use scene::Feature;

mod sections;
pub use sections::*;
mod appearance;
pub use appearance::{AlternativeId, AlternativeObject, Appearance, Family, SetId, SetObject};
pub use appearance::{Look as AppearanceLook, Set as AppearanceSet};
mod document;
pub use document::{
    AnimationSet, AnimationSetId, AnimationSetObject, CoordinateConvention, DocumentControls,
};
/// Mixed-family simulation bodies, colliders, and constraints.
pub mod physics;
/// Saved transforms and skin-authoring pose conventions.
pub mod poses;

mod vocabulary;
pub use vocabulary::Wem;
pub(crate) use vocabulary::{validate_universal_asset, validate_universal_model};

mod availability;
pub use availability::Availability;

mod mesh;
pub use mesh::MeshControls;

mod lod;
pub use lod::LodPreparation;

/// Attached behavior declarations retaining the editable-model vocabulary.
pub mod actors;

mod stages;
pub use stages::*;

mod attachment_validation;
