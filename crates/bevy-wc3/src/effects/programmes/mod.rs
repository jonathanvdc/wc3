//! External programmes enter through Universal features and share sampled sprite transport.
mod controls;
mod immutable;
mod prepared;
mod ribbon;
mod runtime;
mod sampling;
mod visibility;
pub use controls::{
    ParticleProgramAttribute, ParticleProgramControlError, ParticleProgramControls,
};
pub use prepared::{ParticleEffect, ParticleTextureBinding, ParticleTextureRequest};
pub use runtime::ParticleProgramEmitter;
pub(crate) use runtime::{ProgramInstances, RuntimeProgramParticles};

#[cfg(test)]
mod tests;

pub(super) const MODEL_UNITS_PER_EFFECT_UNIT: f32 = 100.;
