//! Logical cameras, illumination, resource systems, and attachments.
use crate::model::features::IndexedChildModel;
use crate::model::{EffectResourceId, ModelResourceId};
use crate::model::{Link, NodeCamera, SingleEmissionLight, ValidateLinks};
/// A portable feature attached to a graph hierarchy node.
#[derive(Clone, Debug)]
pub enum Feature {
    /// Authored camera parameters.
    Camera(Box<NodeCamera>),
    /// Authored illumination.
    Light(Box<SingleEmissionLight>),
    /// External model attachment.
    Attachment(Attachment),
    /// External particle-system instance.
    Particles(EffectResourceId),
    /// External ribbon-system instance.
    Ribbon(EffectResourceId),
    /// External face-animation actor.
    FaceActor(EffectResourceId),
}
impl ValidateLinks for Feature {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Attachment(value) => value.visit_links(visitor),
            Self::Camera(value) => value.visit_links(visitor),
            Self::Light(value) => value.visit_links(visitor),
            Self::Particles(value) | Self::Ribbon(value) | Self::FaceActor(value) => {
                value.visit_links(visitor)
            }
        }
    }
}
/// External attachment resource and animation visibility.
pub type Attachment = IndexedChildModel<ModelResourceId>;
