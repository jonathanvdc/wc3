//! A shared feature family vocabulary selected independently from graph storage.
use crate::model::profiles::FeaturePayload;
use crate::model::{FeatureKind, FeatureSchema, FeatureView, Link, ValidateLinks};
use std::fmt::Debug;
/// Selects the concrete forms admitted by each shared behavior family.
pub trait FeatureVocabulary<'a>: Clone + Debug + Send + Sync {
    /// Owning form admitted for this behavior family.
    type Light: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type Camera: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type Attachment: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type Particle: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type Ribbon: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type GeometryAnimation: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type CollisionShape: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type ParticleProgram: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type FaceActor: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type LightProgram: Clone + Debug + Send + Sync;

    /// Owning form admitted for this behavior family.
    type AtlasParticles: Clone + Debug + Send + Sync;
    /// Owning form admitted for this behavior family.
    type PointLight: Clone + Debug + Send + Sync;
    /// Traverses all graph relationships admitted by this vocabulary.
    fn visit_links(value: &FeatureSet<'a, Self>, visitor: &mut dyn FnMut(Link));
    /// Borrows the shared semantic inspection view.
    fn feature<'view>(value: &'view FeatureSet<'a, Self>) -> FeatureView<'view, 'a>;

    /// Classifies graph relationships requiring a specific behavior.
    fn kind(value: &FeatureSet<'a, Self>) -> FeatureKind;
}
/// Shared behavior families parameterized by a selected vocabulary.
///
/// Unsupported families have no constructible payload:
/// ```compile_fail
/// use wc3::model::profiles::wmo;
/// use wc3::model::scene;
/// let feature = wmo::Node::Camera(Box::new(scene::Camera::default()));
/// ```
#[derive(Clone, Debug)]
pub enum FeatureSet<'a, V: FeatureVocabulary<'a>> {
    /// Complete owning definition of this behavior.
    Light(<V as FeatureVocabulary<'a>>::Light),
    /// Complete owning definition of this behavior.
    Camera(<V as FeatureVocabulary<'a>>::Camera),
    /// Complete owning definition of this behavior.
    Attachment(<V as FeatureVocabulary<'a>>::Attachment),
    /// Complete owning definition of this behavior.
    Particle(<V as FeatureVocabulary<'a>>::Particle),
    /// Complete owning definition of this behavior.
    Ribbon(<V as FeatureVocabulary<'a>>::Ribbon),
    /// Complete owning definition of this behavior.
    GeometryAnimation(<V as FeatureVocabulary<'a>>::GeometryAnimation),
    /// Complete owning definition of this behavior.
    CollisionShape(<V as FeatureVocabulary<'a>>::CollisionShape),
    /// Complete owning definition of this behavior.
    ParticleProgram(<V as FeatureVocabulary<'a>>::ParticleProgram),
    /// Complete owning definition of this behavior.
    FaceActor(<V as FeatureVocabulary<'a>>::FaceActor),
    /// Complete owning definition of this behavior.
    LightProgram(<V as FeatureVocabulary<'a>>::LightProgram),

    /// Complete owning definition of this behavior.
    AtlasParticles(<V as FeatureVocabulary<'a>>::AtlasParticles),
    /// Complete owning definition of this behavior.
    PointLight(<V as FeatureVocabulary<'a>>::PointLight),
}
impl<'a, V: FeatureVocabulary<'a>> FeatureSet<'a, V> {
    /// Classifies payloads required by typed graph relationships.
    pub fn kind(&self) -> FeatureKind {
        V::kind(self)
    }
}
impl<'a, V: FeatureVocabulary<'a>> ValidateLinks for FeatureSet<'a, V> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        V::visit_links(self, visitor);
    }
}
impl<'a, V: FeatureVocabulary<'a>> FeatureSchema<'a> for FeatureSet<'a, V> {
    fn feature(&self) -> FeatureView<'_, 'a> {
        V::feature(self)
    }
}
impl<'a, V: FeatureVocabulary<'a>> FeaturePayload for FeatureSet<'a, V> {
    fn kind(&self) -> FeatureKind {
        V::kind(self)
    }
}
impl<'a, V: FeatureVocabulary<'a>> PartialEq for FeatureSet<'a, V>
where
    V::Light: PartialEq,
    V::Camera: PartialEq,
    V::Attachment: PartialEq,
    V::Particle: PartialEq,
    V::Ribbon: PartialEq,
    V::GeometryAnimation: PartialEq,
    V::CollisionShape: PartialEq,
    V::ParticleProgram: PartialEq,
    V::FaceActor: PartialEq,
    V::LightProgram: PartialEq,

    V::AtlasParticles: PartialEq,
    V::PointLight: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Light(left), Self::Light(right)) => left == right,
            (Self::Camera(left), Self::Camera(right)) => left == right,
            (Self::Attachment(left), Self::Attachment(right)) => left == right,
            (Self::Particle(left), Self::Particle(right)) => left == right,
            (Self::Ribbon(left), Self::Ribbon(right)) => left == right,
            (Self::GeometryAnimation(left), Self::GeometryAnimation(right)) => left == right,
            (Self::CollisionShape(left), Self::CollisionShape(right)) => left == right,
            (Self::ParticleProgram(left), Self::ParticleProgram(right)) => left == right,
            (Self::FaceActor(left), Self::FaceActor(right)) => left == right,
            (Self::LightProgram(left), Self::LightProgram(right)) => left == right,

            (Self::AtlasParticles(left), Self::AtlasParticles(right)) => left == right,
            (Self::PointLight(left), Self::PointLight(right)) => left == right,
            _ => false,
        }
    }
}
