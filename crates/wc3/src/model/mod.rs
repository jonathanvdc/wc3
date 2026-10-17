//! Editable model meaning, independent of native encoding and record layout.
//!
//! A [`Model`](crate::model::Model) owns its graph, profile-specific metadata, and
//! image dependency declarations. It connects hierarchy nodes, shared geometry,
//! material definitions, resource references, animation, and scene systems through scoped IDs.
//! A [`MeshInstance`](crate::model::MeshInstance) places a mesh at a node; its [`Draw`](crate::model::Draw) values select the
//! geometry and rendering controls for individual steps. A [`Look`](crate::model::Look) supplies a complete
//! material assignment for those draws. Material inputs own their animated
//! [`TextureTransform`](crate::model::TextureTransform) values and independently select shared image resources. An [`AttachedFeature`](crate::model::AttachedFeature) gives a node an effect or behavior,
//! while [`System`](crate::model::System) describes independently authored relationships such as physics
//! bodies or world portals.
//!
//! The [`profiles::Profile`](crate::model::profiles::Profile) parameter selects the payload types allowed on these
//! objects. `Profile::ModelMetadata` supplies model-wide declarations. Owner-local `controls` belong to their node, material input, mesh, draw,
//! clip, or controller. Complete material surfaces and attached features use the
//! profile's surface and feature vocabulary. Built-in profiles describe game
//! semantics; native codec dialects separately select file layouts and revisions.
//!
//! [`Animated`] is a typed slot identity into the model's [`Values`] table.
//! The sealed [`Animatable`] trait selects storage for each supported value type.
//! Nodes, materials, and effects retain slot references; constants and evaluated
//! instance values live in separate tables with the same identities.
//! [`Track`] connects a slot to a checked [`Curve`] and explicit [`Clock`].
//! Slots and channels are profile-independent, and native exporters resolve
//! consuming fields when translating them into native tracks.
//! Curve constructors select interpolation together with its required key shape:
//! [`HermiteKeyframe`](crate::model::HermiteKeyframe) carries derivatives,
//! [`BezierKeyframe`](crate::model::BezierKeyframe) carries control points, and
//! [`TcbKeyframe`](crate::model::TcbKeyframe) carries authored spline parameters.
//! Capability traits prevent spherical scalar curves and interpolated discrete resource choices.
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
    curves::*, properties::*, Animatable, Animated, AnimationProgram, AnimationSample, Channel,
    ChannelValue, Clip, Clock, Construct, Controller, Event, Interpolation, RetimedAnimation,
    SampledAnimation, SampledTrack, Time, Track, ValueObject, ValueSample, ValueSlot, Values,
};
mod playback_controls;
pub use playback_controls::*;
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
pub mod feature_properties;
pub use feature_properties::{socket, FeatureProperty};
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
