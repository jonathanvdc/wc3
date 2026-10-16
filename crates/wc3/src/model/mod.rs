//! Editable model meaning, independent of native encoding and record layout.
//!
//! A [`Model`](crate::model::Model) owns its graph, profile-specific metadata, and
//! [`Resources`](crate::model::Resources) arena. It connects hierarchy nodes, shared geometry,
//! material definitions, resource references, animation, and scene systems through scoped IDs.
//! A [`MeshInstance`](crate::model::MeshInstance) places a mesh at a node; its [`Draw`](crate::model::Draw) values select the
//! geometry and rendering controls for individual passes. A [`Look`](crate::model::Look) supplies a complete
//! material assignment for those draws. Shared [`TextureTransform`](crate::model::TextureTransform) objects animate sampling
//! coordinates independently of node placement. An [`AttachedFeature`](crate::model::AttachedFeature) gives a node an effect or behavior,
//! while [`System`](crate::model::System) describes independently authored relationships such as physics
//! bodies or world portals.
//!
//! The [`profiles::Profile`](crate::model::profiles::Profile) parameter selects the payload types allowed on these
//! objects. `Profile::ModelMetadata` supplies model-wide declarations. Owner-local `controls` belong to their node, texture, mesh, draw,
//! clip, or controller. Complete material surfaces and attached features use the
//! profile's surface and feature vocabulary. Built-in profiles describe game
//! semantics; native codec dialects separately select file layouts and revisions.
//!
//! [`Track`](crate::model::Track) binds a value-typed and profile-typed
//! [`Property`](crate::model::Property) to a checked [`Curve`](crate::model::Curve)
//! and explicit [`Clock`](crate::model::Clock). Each profile selects its structural
//! property enum through `Profile::Property`; owner IDs, field variants, and collection
//! ordinals form the destination rather than display names. Widening and narrowing map
//! these enums along with programme channels while preserving identity and shared curves.
//! Curve constructors select interpolation together with its required key shape:
//! [`HermiteKeyframe`](crate::model::HermiteKeyframe) carries derivatives,
//! [`BezierKeyframe`](crate::model::BezierKeyframe) carries control points, and
//! [`TcbKeyframe`](crate::model::TcbKeyframe) carries authored spline parameters.
//! Capability traits prevent spherical scalar curves and interpolated discrete resource choices.
//! Custom profiles implement [`PropertyPayload`](crate::model::PropertyPayload)
//! and [`PropertyDescriptor`](crate::model::PropertyDescriptor) for their own destinations.
//!
//! Fixed-width arrays use [`ZeroVec`](crate::model::ZeroVec), borrowing compatible packed storage.
//! Native slices and differing layouts are encoded into owned arrays once. Variable-length
//! rows use [`ValueGroups`](crate::model::ValueGroups) with flat elements and row offsets. [`ModelBuilder::finish`](crate::model::ModelBuilder::finish) checks relationships and values before
//! publication. It returns a [`Model`](crate::model::Model) containing the graph, metadata, and resources.
//! [`Model::edit`](crate::model::Model::edit) checks dependency changes against the complete model.
//!
//! [`Model::widen`](crate::model::Model::widen) embeds payloads into a containing profile without changing
//! their meaning, IDs, or encoded array storage. Applications implement custom
//! profiles to select their own payload vocabularies. [`io`](crate::model::io) imports native documents
//! and plans exports, with representability policies and diagnostics from [`conversion`](crate::model::conversion).
//! [`formats`](crate::model::formats) contains the native documents and their codecs for faithful record editing.
mod error;
/// Format-independent scene features and simulation systems.
pub mod scene;
/// Shared logical selector and flag domains.
pub mod values;
pub use error::Error;
mod ids;
pub use ids::*;
mod value_groups;
pub use crate::model::common::ZeroVec;
pub use value_groups::ValueGroups;
mod rotation;
pub use rotation::*;
pub mod animation;
pub use animation::{
    curves::*, properties::*, Animated, AnimationProgram, AnimationSample, Channel, ChannelValue,
    Clip, Clock, Controller, Event, Interpolation, RetimedAnimation, SampledAnimation,
    SampledTrack, Time, Track,
};
mod playback_controls;
pub use playback_controls::*;
mod resources;
pub use resources::*;
pub mod appearance;
pub use appearance::{
    AppearanceSchema, DrawAppearanceSchema, DrawLooks, MaterialSelection, SingleLook,
    UniversalAppearance,
};
mod binding;
pub use binding::*;
pub use profiles::m2::M2Geometry;
pub use profiles::m3::{M3CollisionGeometry, M3DisplayGeometry, M3Geometry};
pub use profiles::mdlx::MdlxGeometry;
mod vertex_geometry;
pub use vertex_geometry::{JointInfluences, VertexData};
mod influences;
pub use influences::GeometryInfluences;
mod geometry_edit;
pub use geometry_edit::*;
mod geometry_schema;
pub use geometry_schema::*;
pub use profiles::wmo::WmoGeometry;
mod geometry_domains;
pub use geometry_domains::{ConnectivitySets, SurfaceTriangleControls, TriangleFlags};
pub mod geometry;
pub use geometry::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, Bounds, Draw, Influence, Mesh,
    MeshInstance, NodeInfluence, Skin, Topology, VertexInfluences,
};
mod node;
pub use node::*;
pub mod materials;
pub use materials::{
    AddressMode, ColorSpace, Material, MaterialSurface, Resource, Sampler, SnoNamespace,
    SurfaceSet, Texture,
};
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
pub mod feature_properties;
pub use feature_properties::FeatureProperty;
pub mod features;
pub use features::*;
/// Compile-time logical vocabularies and lossless embeddings.
pub mod controls;
pub use controls::{DrawControls, InstanceControls, MeshControls, NodeControls, TextureControls};
mod texture_transform;
pub use texture_transform::*;
pub mod systems;
pub use systems::{ModelMetadata, System, SystemMember, SystemPayload};
mod graph;
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
