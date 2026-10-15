//! Curve-driven particle vocabulary selected by the model-shader profile.
pub use crate::model::LegacyAtlas;
use crate::model::ParticleAgeCurves;
/// Curves evaluated against normalized particle age.
pub type Appearance<'a> = ParticleAgeCurves<'a>;
use crate::model::AgeDrivenParticles;
/// Complete logical particle generator definition.
pub type Particle<'a> = AgeDrivenParticles<'a>;
