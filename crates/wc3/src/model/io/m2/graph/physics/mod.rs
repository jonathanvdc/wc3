//! Native PHYS data maps directly to logical bodies and constraints.
use super::super::{failure, Chunk, ExportDialect};
use crate::model::conversion::{Context, Error};
use crate::model::formats::m2::{self as native, Bundle, Dialect, File};
use crate::model::{
    self as semantic, profiles::m2 as profile, profiles::M2 as M2Vocabulary, M2ResourceLocator,
    Model, ModelBuilder, NodeId, Transform,
};
use glam::Vec3;
type Graph<'a> = Model<'a, M2Vocabulary>;
type GraphBuilder<'a> = ModelBuilder<'a, M2Vocabulary>;
use native::companions::{PhysicsChunk as Pc, PhysicsFile};
mod bodies;
mod export;
mod export_joints;
mod export_shapes;
mod hull;
mod import;
mod joints;
mod shapes;
pub(super) use export::export;
pub(super) use import::import;
fn identity() -> [[f32; 4]; 4] {
    std::array::from_fn(|row| std::array::from_fn(|column| if row == column { 1. } else { 0. }))
}
fn frame(value: &native::companions::PhysicsFrame) -> profile::Frame {
    profile::Frame {
        axis_x: Vec3::from_array(value.axis_x),
        axis_y: Vec3::from_array(value.axis_y),
        axis_z: Vec3::from_array(value.axis_z),
        origin: Vec3::from_array(value.origin),
    }
}
fn native_frame(value: profile::Frame) -> native::companions::PhysicsFrame {
    native::companions::PhysicsFrame {
        axis_x: (value.axis_x).to_array(),
        axis_y: (value.axis_y).to_array(),
        axis_z: (value.axis_z).to_array(),
        origin: (value.origin).to_array(),
    }
}
fn anchor(origin: [f32; 3]) -> profile::Frame {
    profile::Frame {
        axis_x: Vec3::from_array([1., 0., 0.]),
        axis_y: Vec3::from_array([0., 1., 0.]),
        axis_z: Vec3::from_array([0., 0., 1.]),
        origin: Vec3::from_array(origin),
    }
}
