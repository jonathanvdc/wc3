//! External programmes enter through Universal features and share sampled sprite transport.
mod controls;
mod prepared;
mod runtime;
mod ribbon;
mod sampling;
mod visibility;
pub use controls::{ParticleProgramAttribute, ParticleProgramControlError, ParticleProgramControls};
pub use prepared::{ParticleEffect, ParticleTextureBinding, ParticleTextureRequest};
pub use runtime::ParticleProgramEmitter;
pub(crate) use runtime::{ProgramInstances, RuntimeProgramParticles};

#[cfg(test)]
mod tests;
