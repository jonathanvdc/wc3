//! Checked model editing and evaluation, with native file codecs alongside it.
//!
//! [`Model`](crate::model::Model) connects nodes, geometry, materials, animation, resources, and scene
//! features through typed IDs. Build one with [`Model::builder`](crate::model::Model::builder), or import a
//! native document through [`io`](crate::model::io). [`ModelBuilder::finish`](crate::model::ModelBuilder::finish) checks the graph;
//! [`Model::edit`](crate::model::Model::edit) stages changes and publishes them only after validation.
//!
//! # Navigate the graph
//!
//! A [`MeshInstance`](crate::model::MeshInstance) places a shared mesh at a node. Its [`Draw`](crate::model::Draw) values select
//! geometry and rendering properties, while a [`Look`](crate::model::Look) assigns materials to those
//! draws. Material inputs reference shared image declarations and own their
//! sampling and animated [`TextureTransform`](crate::model::TextureTransform) values. [`AttachedFeature`](crate::model::AttachedFeature) adds
//! node-bound behavior; [`System`](crate::model::System) connects independently authored scene elements,
//! such as physics bodies or world portals.
//!
//! [`profiles::Profile`](crate::model::profiles::Profile) selects the payloads allowed on these objects, including
//! model metadata, owner properties, surfaces, effects, and event sources. Built-in
//! profiles describe game semantics; native dialects separately select file
//! layouts. [`Model::widen`](crate::model::Model::widen) embeds a model into a containing profile without
//! changing its meanings or IDs.
//!
//! # Animate and evaluate
//!
//! [`Property`](crate::model::Property) identifies a typed slot in [`PropertyValues`](crate::model::PropertyValues). Nodes, materials,
//! and effects reference slots; authored defaults and evaluated values occupy
//! separate tables with the same identities. A [`Track`](crate::model::Track) connects a slot to a
//! [`Curve`](crate::model::Curve) and a [`TimeSource`](crate::model::TimeSource). Reusable animations also contain
//! [`EventOccurrence`](crate::model::EventOccurrence) values referencing model-owned event sources.
//!
//! Curve constructors pair interpolation with the required key shape, including
//! [`HermiteKeyframe`](crate::model::HermiteKeyframe), [`BezierKeyframe`](crate::model::BezierKeyframe), and [`TcbKeyframe`](crate::model::TcbKeyframe). See the evaluation
//! APIs on [`Model`](crate::model::Model) for CPU pose, geometry, and surface sampling.
//!
//! # Inspect rejected changes
//!
//! [`Error`](crate::model::Error) exposes a typed [`cause`](crate::model::error::ErrorKind) and a structured
//! [`location`](crate::model::error::Location), including actual object and property identities.
//! Use [`Model::try_edit`](crate::model::Model::try_edit) when an operation has an application-specific failure;
//! [`error::EditError`](crate::model::error::EditError) distinguishes that failure from final graph validation.
//! Both phases leave the original model unchanged. Custom validators can retain
//! their original typed cause with [`Error::extension`](crate::model::Error::extension).
//!
//! # Work across the native boundary
//!
//! [`io`](crate::model::io) imports and exports native documents with policies from [`conversion`](crate::model::conversion).
//! Use [`formats`](crate::model::formats) directly for native record edits and preservation of unknown
//! source data. Checked models retain supported meanings rather than source-file
//! layouts. Their fixed-width [`PackedArray`](crate::model::PackedArray) arrays can borrow compatible native
//! storage; [`ValueGroups`](crate::model::ValueGroups) stores variable-length rows as elements and offsets.

/// Typed semantic failures and structured diagnostic locations.
pub mod error;
/// Format-independent scene features and simulation systems.
pub mod scene;
/// Shared logical selector and flag domains.
pub mod values;
pub use error::Error;
mod ids;
pub use ids::*;
mod value_groups;
pub use crate::model::common::{IntoPackedArray, PackedArray, PackedValue};
pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
pub use value_groups::ValueGroups;
pub mod animation;
pub use animation::{
    curves::*, Animation, AnimationProgram, AnimationSample, Clip, Construct, DynamicTrack,
    EventOccurrence, Interpolation, Property, PropertyValue, PropertyValues, RetimedAnimation,
    SampledAnimation, SampledTrack, Time, TimeSource, Track, TrackValue, ValueObject, ValueSample,
    ValueSlot,
};
mod events;
pub use events::*;
mod playback_properties;
pub use playback_properties::*;
pub mod appearance;
pub use appearance::{
    AppearanceSchema, DrawAppearanceSchema, DrawLooks, MaterialSelection, SingleLook,
    UniversalAppearance,
};
mod binding;
pub use binding::*;
pub use profiles::m2::M2Geometry;

pub use profiles::mdlx::MdlxGeometry;
mod vertex_geometry;
pub use vertex_geometry::{JointInfluences, VertexData};
mod influence_rows;
pub use influence_rows::{JointWeightIter, JointWeights, VertexInfluences};
mod influences;
pub use influences::GeometryInfluences;
mod geometry_edit;
pub use geometry_edit::*;
mod geometry_schema;
pub use geometry_schema::*;
pub use profiles::wmo::WmoGeometry;
mod geometry_domains;
pub use geometry_domains::{ConnectivitySets, TriangleProperties};
pub mod geometry;
pub use geometry::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Bounds, Draw, Influence, Mesh,
    MeshInstance, NodeInfluence, Skin, Topology,
};
mod mounts;
pub use mounts::*;
mod node;
mod node_evaluation;
pub use node::*;
pub use node_evaluation::*;
pub mod materials;
pub use materials::program::{MaterialProgram, MaterialStep};
pub use materials::{AddressMode, ColorSpace, Material, MaterialSurface, Sampler, Texture};
mod links;
pub use links::*;
/// Compile-time selections and authoring rules for model families.
pub mod profiles;
mod space;
pub use space::*;
mod look;
pub use look::*;
mod feature_schema;
pub use feature_schema::*;

pub mod features;
pub use features::*;
/// Compile-time logical vocabularies and lossless embeddings.
pub mod properties;
pub use properties::{DrawProperties, InstanceProperties, MeshProperties, NodeProperties};
mod texture_transform;
pub use texture_transform::*;
pub mod systems;
pub use systems::{ModelMetadata, System, SystemMember, SystemPayload};
mod graph;
mod ownership;
pub use graph::*;

#[cfg(test)]
mod tests;

mod geometry_attributes;
pub use geometry_attributes::{GeometryAttributes, TextureCoordinates};

mod geometry_parts;
pub(crate) use geometry_domains::GeometryPools;
pub(crate) use geometry_parts::GeometryParts;

mod draw_subset;
pub use draw_subset::*;

/// Shared storage and binary mechanics.
pub mod common;
/// Semantic translation policies and diagnostics.
pub mod conversion;
/// Native file layouts and codecs.
pub mod formats;
/// Native document import and export.
pub mod io;

mod geometry_projection;
pub use geometry_projection::*;

mod pose_layout;
pub use pose_layout::*;

mod deformation;
pub use deformation::*;

mod surface_evaluation;
pub use surface_evaluation::*;

pub mod resources;
pub use resources::*;

mod table;
pub use table::Table;
mod dependencies;
pub use dependencies::Dependencies;

mod metadata;
