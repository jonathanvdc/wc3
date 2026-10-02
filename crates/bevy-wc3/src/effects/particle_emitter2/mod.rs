//! PRE2 particle spawning and rendering internals.
//!
//! The CPU samples emission tracks, generates immutable spawn records, and
//! retires particles using a per-emitter simulation clock. The vertex shader
//! evaluates analytic constant-gravity motion, lifetime curves, and atlas UVs.
//!
//! Growing rings keep records resident on the GPU. Birth cursors select upload
//! ranges; draw-order indices provide per-view depth sorting for SortPrimsFarZ.

mod emission;
mod render;
mod simulation;
mod spawn;

use bevy::prelude::Component;

pub(crate) use render::{ParticleInstances, ParticleRenderPlugin};
pub(crate) use simulation::{update_particles, Particle2State};

/// Associates a spawned emitter with its texture binding slot.
#[derive(Component)]
pub(crate) struct ParticleTextureSlot(pub(crate) usize);

pub(crate) use spawn::spawn_particles2;

pub(crate) mod textures;
