//! Direct M3 graph mappings. Native documents are read at the adapter boundary only.
mod animation;
mod bindings;
mod cameras;
mod forces;
mod geometry;
mod hierarchy;
mod lights;
mod materials;
mod skeleton_write;
mod topology;
mod vertices;
use crate::model::profiles::M3;
use crate::model::{Model, ModelBuilder};
pub(super) type Graph<'a> = Model<'a, M3>;
pub(super) type GraphBuilder<'a> = ModelBuilder<'a, M3>;
mod mesh_bounds;

mod packing;

mod vertex_write;

mod write;

mod property_ids;

mod geometry_write;

mod ik;

pub(super) mod profiles;

mod orchestration;
pub(super) use orchestration::{plan, read};

mod companions;

mod static_scene;

mod projectors;

mod particles;
mod slots;

mod ribbons;

mod collision;

mod physics;

mod cloth;

mod volumes;

mod turrets;

mod partitions;

mod qualifiers;

mod undrawn_geometry;

mod definition_bounds;

mod emitter_links;
