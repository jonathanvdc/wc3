//! Editable model meaning, independent of native encoding and record layout.
//!
//! [`ModelCollection`](crate::model::semantic::ModelCollection) supplies one authoritative [`Resources`](crate::model::semantic::Resources) arena to its models.
//! `Resources` owns its texture map directly; model views retain the complete dependency context.
//! A [`Model`](crate::model::semantic::Model) connects hierarchy nodes, shared geometry, material definitions,
//! resource references, animation, and scene systems through graph-scoped IDs.
//! A [`MeshInstance`](crate::model::semantic::MeshInstance) places a mesh at a node; its [`Draw`](crate::model::semantic::Draw) values select the
//! geometry and rendering controls for individual passes. A [`Look`](crate::model::semantic::Look) supplies a complete
//! material assignment for those draws. Shared [`TextureTransform`](crate::model::semantic::TextureTransform) objects animate sampling
//! coordinates independently of node placement. An [`AttachedFeature`](crate::model::semantic::AttachedFeature) gives a node an effect or behavior,
//! while [`System`](crate::model::semantic::System) describes independently authored relationships such as physics
//! bodies or world portals.
//!
//! The [`profiles::Profile`](crate::model::semantic::profiles::Profile) parameter selects the payload types allowed on these
//! objects. Owner-local `controls` belong to their node, texture, mesh, draw,
//! clip, or controller. Complete material surfaces and attached features use the
//! profile's surface and feature vocabulary. Built-in profiles describe game
//! semantics; native codec dialects separately select file layouts and revisions.
//!
//! [`Track`](crate::model::semantic::Track) binds a value-typed and profile-typed
//! [`Property`](crate::model::semantic::Property) to a checked [`Curve`](crate::model::semantic::Curve)
//! and explicit [`Clock`](crate::model::semantic::Clock). Each profile selects its structural
//! property enum through `Profile::Property`; owner IDs, field variants, and collection
//! ordinals form the destination rather than display names. Widening and narrowing map
//! these enums along with programme channels while preserving identity and shared curves.
//! Curve constructors select interpolation together with its required key shape:
//! [`HermiteKeyframe`](crate::model::semantic::HermiteKeyframe) carries derivatives,
//! [`BezierKeyframe`](crate::model::semantic::BezierKeyframe) carries control points, and
//! [`TcbKeyframe`](crate::model::semantic::TcbKeyframe) carries authored spline parameters.
//! Capability traits prevent spherical scalar curves and interpolated discrete resource choices.
//! Custom profiles implement [`PropertyPayload`](crate::model::semantic::PropertyPayload)
//! and [`PropertyDescriptor`](crate::model::semantic::PropertyDescriptor) for their own destinations.
//!
//! [`Values`](crate::model::semantic::Values) can borrow aligned arrays, packed native storage, or indexed views.
//! Building a graph therefore need not materialize its geometry and animation
//! arrays. [`ModelBuilder::finish`](crate::model::semantic::ModelBuilder::finish) checks relationships and values before
//! publication. It returns a [`SingleModel`](crate::model::semantic::SingleModel) containing the graph and its resources.
//! [`ModelCollection::edit_model`](crate::model::semantic::ModelCollection::edit_model) checks dependency changes against the complete collection.
//!
//! [`Model::widen`](crate::model::semantic::Model::widen) embeds payloads into a containing profile without changing
//! their meaning, IDs, or shared array storage. [`Model::union`](crate::model::semantic::Model::union) combines two
//! vocabularies without translating one into the other. Native adapters and
//! representability policies live in [the conversion API](crate::model::conversion).
mod error;
/// Format-independent scene features and simulation systems.
pub mod scene;
pub use error::Error;
mod ids;
pub use ids::*;
mod values;
pub use values::Values;
mod rotation;
pub use rotation::*;
mod animation;
pub use animation::*;
mod playback_controls;
pub use playback_controls::*;
mod model_collection;
pub use model_collection::*;
mod resources;
pub use resources::*;
mod appearance;
pub use appearance::*;
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
mod geometry;
pub use geometry::*;
mod node;
pub use node::*;
mod material;
pub use material::*;
mod links;
pub use links::*;
/// Complete logical definitions for distinct game feature families.
pub mod profiles;
mod space;
pub use space::*;
mod look;
pub use look::*;
mod feature;
pub use feature::*;
/// Compile-time logical vocabularies and lossless embeddings.
mod controls;
pub use controls::*;
mod texture_transform;
pub use texture_transform::*;
mod systems;
pub use systems::*;
mod model;
pub use model::*;

#[cfg(test)]
mod tests;

mod geometry_attributes;
pub use geometry_attributes::{GeometryAttributes, TextureCoordinates};

mod geometry_parts;
pub(crate) use geometry_domains::GeometryPools;
pub(crate) use geometry_parts::GeometryParts;

mod draw_subset;
pub use draw_subset::*;
