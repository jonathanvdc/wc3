//! Bone-attached scene features with typed logical animation bindings.
use super::values::{CameraKind, LightKind};
use crate::model::profiles::M2;
use crate::model::AngleTrail;
use crate::model::AttachmentPointDefinition;
use crate::model::{AimedCamera, AmbientDiffuseLight};
use crate::model::{Animated, AttachedFeature, FeatureSet, Link, ValidateLinks};

use super::BoneControls;
/// A hierarchy node carrying a model-family feature.
pub type Actor<'a> = AttachedFeature<'a, M2>;
/// Distinct scene feature families.
pub type ActorKind<'a> = FeatureSet<'a, M2>;
/// Runtime attachment role and enable control.
pub type Attachment = AttachmentPointDefinition<u32, Animated<bool>, ()>;
pub use crate::model::scene::model::Event;
/// Model light with independently animated ambient and diffuse components.
pub type Light = AmbientDiffuseLight<LightKind, Animated<bool>>;
/// Camera projection and model-local look-at animation.
pub type Camera = AimedCamera<Animated<f32>, CameraKind>;
/// A generated ribbon trail with texture and shader graph bindings.
pub type Ribbon = AngleTrail;

impl ActorKind<'_> {
    pub(crate) fn visit_profile_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Attachment(x) => x.activation.visit_links(v),
            Self::Light(x) => x.visit_links(v),
            Self::Camera(x) => x.visit_links(v),
            Self::Ribbon(x) => x.visit_links(v),
            Self::Particle(x) => x.visit_links(v),
            Self::Event(_) => {}

            Self::GeometryAnimation(unavailable)
            | Self::CollisionShape(unavailable)
            | Self::ParticleProgram(unavailable)
            | Self::FaceActor(unavailable)
            | Self::LightProgram(unavailable)
            | Self::Extent(unavailable)
            | Self::ParticleCopy(unavailable)
            | Self::Projector(unavailable)
            | Self::Force(unavailable)
            | Self::Warp(unavailable)
            | Self::HitTest(unavailable)
            | Self::AttachmentVolume(unavailable)
            | Self::Billboard(unavailable)
            | Self::Turret(unavailable)
            | Self::IkJoint(unavailable)
            | Self::IkTwoJoint(unavailable)
            | Self::IkCcd(unavailable)
            | Self::OneBoneSolver(unavailable)
            | Self::ShadowBox(unavailable)
            | Self::ViewVolume(unavailable)
            | Self::AtlasParticles(unavailable)
            | Self::PointLight(unavailable) => match *unavailable {},
        }
    }
}
impl ValidateLinks for BoneControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.flags.bits() & 0x7f != 0 {
            visitor(Link::InvalidValue);
        }
    }
}
