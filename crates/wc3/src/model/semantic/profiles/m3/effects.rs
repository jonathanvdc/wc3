//! Programmed effect vocabulary selected by the scene profile.
pub use crate::model::semantic::{ExternalModel, ParticleCopy, SplinePoint, VariationControl};
use crate::model::semantic::{ProgrammedParticles, ProgrammedTrail};
/// Complete programmed particle definition.
pub type Particle<'a> = ProgrammedParticles<'a>;
/// Complete programmed trail definition.
pub type Ribbon<'a> = ProgrammedTrail<'a>;
