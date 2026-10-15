//! Particle effects and ribbon trails attached to model nodes.
//!
//! [`ParticleEmitter`] emits model or image resources; [`ParticleEmitter2`] emits
//! textured particles with head and tail rendering. [`RibbonEmitter`] creates
//! trails, and [`PopcornEmitter`] references a Reforged PopcornFX effect. Each
//! emitter exposes its own flag type because shared node bits have different
//! meanings for different effects.
mod particle;
pub use particle::{ParticleEmitter, ParticleEmitterFlags};
mod particle2;
pub use particle2::{Particle2FilterMode, Particle2Flags, Particle2Frames, ParticleEmitter2};
mod popcorn;
pub use popcorn::{PopcornEmitter, PopcornFlags};
mod ribbon;
pub use ribbon::RibbonEmitter;
