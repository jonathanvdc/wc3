//! Relationship traversal for Warcraft III profile data.
use super::Node;
use crate::model::{FeatureKind, Link, ValidateLinks};
impl Node<'_> {
    pub(crate) fn visit_profile_links(&self, visitor: &mut dyn FnMut(Link)) {
        macro_rules! fields { ($value:ident; $($field:ident),* $(,)?) => {{ $( $value.$field.visit_links(visitor); )* }}; }
        match self {
            Node::GeometryAnimation(value) => fields!(value; mesh,opacity,color),
            Node::Particle(value) => {
                value.details.resource.visit_links(visitor);
                value.emission.rate.visit_links(visitor);
                value.motion.gravity.visit_links(visitor);
                value.details.longitude.visit_links(visitor);
                value.details.latitude.visit_links(visitor);
                value.lifetime.duration.visit_links(visitor);
                value.motion.speed.visit_links(visitor);
                value.emission.activation.visit_links(visitor);
            }
            Node::Particle2(value) => {
                value.motion.speed.visit_links(visitor);
                value.motion.variation.visit_links(visitor);
                value.details.shape.latitude.visit_links(visitor);
                value.motion.gravity.visit_links(visitor);
                value.emission.rate.visit_links(visitor);
                value.details.shape.length.visit_links(visitor);
                value.details.shape.width.visit_links(visitor);
                value.details.rendering.texture.visit_links(visitor);
                value.emission.activation.visit_links(visitor);
            }
            Node::Ribbon(value) => {
                value.geometry.height_above.visit_links(visitor);
                value.geometry.height_below.visit_links(visitor);
                value.appearance.opacity.visit_links(visitor);
                value.appearance.color.visit_links(visitor);
                value.appearance.texture_slot.visit_links(visitor);
                value.rendering.material.visit_links(visitor);
                value.behavior.visibility.visit_links(visitor);
            }
            Node::ParticleProgram(value) => {
                value.resource.visit_links(visitor);
                fields!(value; life_span,emission_rate,speed,color,opacity,visibility)
            }
            Node::Light(value) => value.visit_links(visitor),
            Node::Camera(value) => value.visit_links(visitor),
            Node::Attachment(value) => value.visit_links(visitor),
            Node::CollisionShape(_) => {}

            Node::FaceActor(unavailable)
            | Node::LightProgram(unavailable)
            | Node::PointLight(unavailable) => match *unavailable {},
        }
    }
}

impl ValidateLinks for super::SequenceProperties {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.move_speed.is_finite() || !self.rarity.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.extent.visit_links(visitor);
    }
}
impl ValidateLinks for super::NodeProperties {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Some(bone) = &self.bone {
            bone.mesh.visit_links(visitor);
            if let Some(id) = bone.animation {
                visitor(Link::TypedFeature {
                    id,
                    kind: FeatureKind::MdlxGeometryAnimation,
                });
            }
        }
        if self
            .bind_pose
            .is_some_and(|matrix| matrix.iter().any(|value| !value.is_finite()))
        {
            visitor(Link::InvalidValue);
        }
    }
}

impl ValidateLinks for super::MeshProperties {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (clip, extent) in &self.sequence_bounds {
            clip.visit_links(visitor);
            extent.visit_links(visitor);
        }
        for extent in &self.unassigned_bounds {
            extent.visit_links(visitor);
        }
    }
}
