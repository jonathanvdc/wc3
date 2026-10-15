//! Curve-driven particle vocabulary selected by the model-shader profile.
pub use crate::model::semantic::LegacyAtlas;
use crate::model::semantic::ParticleAgeCurves;
/// Curves evaluated against normalized particle age.
pub type Appearance<'a> = ParticleAgeCurves<'a>;
use crate::model::semantic::AgeDrivenParticles;
/// Complete logical particle generator definition.
pub type Particle<'a> = AgeDrivenParticles<'a>;
