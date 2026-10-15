//! Individually typed authored WEM behaviors may mix supported game families.
use crate::model::semantic::profiles::FeaturePayload;
use crate::model::semantic::{FeatureKind, Link, PropertyIdentityKey, ValidateLinks};
/// Complete attached behavior, independent of a model-wide game selector.
#[derive(Clone, Debug)]
pub enum Feature<'a> {
    /// World of Warcraft declarations belonging to the editable-model vocabulary.
    Wow(Box<super::actors::wow::Particle<'a>>),
    /// Warcraft III declarations belonging to the editable-model vocabulary.
    Warcraft(Box<super::actors::warcraft::Feature<'a>>),
    /// Force volume affecting particles or simulation bodies.
    Force(super::Force),
    /// Geometric deformation control.
    VertexWarp(super::VertexWarp),
    /// Node-local collision primitive.
    Collision(super::Collision),
    /// Common WEM camera, light, or external actor.
    Common(super::actors::common::Feature<'a>),
    /// StarCraft II behavior.
    Starcraft(super::actors::starcraft::Feature<'a>),
}
impl ValidateLinks for Feature<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Wow(value) => value.visit_links(visitor),
            Self::Warcraft(value) => value.visit_links(visitor),
            Self::Force(v) => v.visit_links(visitor),
            Self::VertexWarp(v) => v.visit_links(visitor),
            Self::Collision(v) => v.visit_links(visitor),
            Self::Common(v) => v.visit_links(visitor),
            Self::Starcraft(v) => v.visit_links(visitor),
        }
    }
}
impl FeaturePayload for Feature<'_> {
    fn supports_property(&self, key: &PropertyIdentityKey) -> bool {
        key.downcast_ref::<super::properties::Target>()
            .is_some_and(|key| key.supports_feature(self))
    }

    fn kind(&self) -> FeatureKind {
        match self {
            Self::Warcraft(_) | Self::Wow(_) => FeatureKind::Other,
            Self::Force(_) | Self::VertexWarp(_) | Self::Collision(_) => FeatureKind::Other,
            Self::Common(_) => FeatureKind::Other,
            Self::Starcraft(v) => match v {
                super::actors::starcraft::Feature::Particle(_) => FeatureKind::WemStarcraftParticle,
                super::actors::starcraft::Feature::ParticleCopy(_) => {
                    FeatureKind::WemStarcraftParticleCopy
                }
                super::actors::starcraft::Feature::Ribbon(_) => FeatureKind::WemStarcraftRibbon,
            },
        }
    }
}
