//! Bone-attached scene features with typed logical animation bindings.
use super::values::LightKind;
use crate::model::profiles::M2;
use crate::model::M2RibbonEmitter;
use crate::model::SocketDefinition;
use crate::model::{AmbientDiffuseLight, LookAtCamera};
use crate::model::{AttachedFeature, FeatureSet, Link, Property, ValidateLinks};

use super::BoneProperties;
/// A hierarchy node carrying a model-family feature.
pub type Actor<'a> = AttachedFeature<'a, M2>;
/// Distinct scene feature families.
pub type ActorKind<'a> = FeatureSet<'a, M2>;
/// Runtime attachment role and enable control.
pub type Attachment = SocketDefinition<u32, Property<bool>, ()>;
/// Model light with independently animated ambient and diffuse components.
pub type Light = AmbientDiffuseLight<LightKind, Property<bool>>;
/// Camera projection and model-local look-at animation.
pub type Camera = LookAtCamera;
/// A generated ribbon trail with texture and shader graph bindings.
pub type RibbonEmitter = M2RibbonEmitter;

impl ActorKind<'_> {
    pub(crate) fn visit_profile_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Attachment(x) => x.visibility.visit_links(v),
            Self::Light(x) => x.visit_links(v),
            Self::Camera(x) => x.visit_links(v),
            Self::Ribbon(x) => x.visit_links(v),
            Self::Particle(x) => x.visit_links(v),

            Self::GeometryAnimation(unavailable)
            | Self::CollisionShape(unavailable)
            | Self::ParticleProgram(unavailable)
            | Self::FaceActor(unavailable)
            | Self::LightProgram(unavailable)
            | Self::Particle2(unavailable)
            | Self::PointLight(unavailable) => match *unavailable {},
        }
    }
}
impl ValidateLinks for BoneProperties {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.flags.bits() & 0x7f != 0 {
            visitor(Link::InvalidValue);
        }
    }
}
