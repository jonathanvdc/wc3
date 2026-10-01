//! PRE2 particle simulation and rendering internals.

mod render;
mod simulation;

use bevy::prelude::Component;

pub(crate) use render::{ParticleInstances, ParticleRenderPlugin};
pub(crate) use simulation::{update_particles, Particle2State};

/// Associates a spawned emitter with its texture binding slot.
#[derive(Component)]
pub(crate) struct ParticleTextureSlot(pub(crate) usize);
