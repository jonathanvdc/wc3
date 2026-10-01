//! PRE2 particle spawning and rendering internals.
//!
//! The CPU samples emission tracks, generates immutable spawn records, and
//! retires particles using a per-emitter simulation clock. The vertex shader
//! evaluates analytic constant-gravity motion, lifetime curves, and atlas UVs.
//! Stable slots keep records resident on the GPU; draw-order indices select
//! live particles and provide per-view CPU depth sorting for SortPrimsFarZ.

mod render;
mod simulation;

use bevy::prelude::Component;

pub(crate) use render::{ParticleInstances, ParticleRenderPlugin};
pub(crate) use simulation::{update_particles, Particle2State};

/// Associates a spawned emitter with its texture binding slot.
#[derive(Component)]
pub(crate) struct ParticleTextureSlot(pub(crate) usize);
