//! Meshes, bounds, skinning, and collision shapes.
//!
//! A [`Geoset`] groups geometry with a material reference. Its editing methods
//! keep per-vertex arrays consistent. [`CollisionShape`] describes collision
//! primitives attached to nodes; model pivot points locate node transform origins.
mod geoset;
pub use geoset::{
    Geoset, GeosetExtent, GeosetExtraSections, GeosetLayout, NoGeosetExtraSections,
    ReforgedGeosetExtraSections, SkinWeights,
};
mod collision;
pub use collision::{CollisionGeometry, CollisionShape};
mod pivot_point;
pub use pivot_point::PivotPoint;
mod bind_pose_matrix;
pub use bind_pose_matrix::BindPoseMatrix;
