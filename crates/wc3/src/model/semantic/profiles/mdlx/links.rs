//! Relationship traversal for Warcraft III profile data.
use super::{Material, Node};
use crate::model::semantic::{FeatureKind, Link, ValidateLinks};
impl ValidateLinks for Material<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for layer in &self.layers {
            layer.texture_transform.visit_links(visitor);
            layer.texture.default.visit_links(visitor);
            layer.texture.visit_links(visitor);
            layer.opacity.visit_links(visitor);
            layer.emissive_gain.visit_links(visitor);
            if let Some(fresnel) = &layer.fresnel {
                fresnel.color.visit_links(visitor);
                fresnel.opacity.visit_links(visitor);
                fresnel.team_color.visit_links(visitor);
            }
            for slot in &layer.textures {
                slot.texture.default.visit_links(visitor);
                slot.texture.visit_links(visitor);
            }
        }
    }
}
impl ValidateLinks for Node<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        macro_rules! fields { ($value:ident; $($field:ident),* $(,)?) => {{ $( $value.$field.visit_links(visitor); )* }}; }
        match self {
            Node::GeometryAnimation(value) => fields!(value; mesh,opacity,color),
            Node::Particle(value) => {
                fields!(value; emission_rate,gravity,longitude,latitude,life_span,initial_velocity,visibility)
            }
            Node::Particle2(value) => {
                fields!(value; speed,variation,latitude,gravity,emission_rate,length,width,texture,visibility)
            }
            Node::Ribbon(value) => {
                fields!(value; height_above,height_below,opacity,color,texture_slot,material,visibility)
            }
            Node::Popcorn(value) => {
                fields!(value; life_span,emission_rate,speed,color,opacity,visibility)
            }
            Node::Light(value) => {
                fields!(value; color,intensity,ambient_color,ambient_intensity,visibility);
                for parameter in &value.attenuation {
                    parameter.visit_links(visitor);
                }
                for parameters in [&value.shadow_range].into_iter().flatten() {
                    for parameter in parameters {
                        parameter.visit_links(visitor);
                    }
                }
                for parameters in [&value.falloff].into_iter().flatten() {
                    for parameter in parameters {
                        parameter.visit_links(visitor);
                    }
                }
            }
            Node::Camera(value) => {
                fields!(value; eye_offset,target_offset,roll,visibility,focus_distance,focal_length,f_stop)
            }
            Node::Attachment(value) => value.visibility.visit_links(visitor),
            Node::Events(_) => {}
            Node::Collision(_) => {}
        }
    }
}

impl ValidateLinks for super::Sequence {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if !self.move_speed.is_finite()
            || !self.rarity.is_finite()
            || !self.radius.is_finite()
            || self.radius < 0.
            || self.bounds.iter().flatten().any(|value| !value.is_finite())
            || self.bounds[0]
                .iter()
                .zip(self.bounds[1])
                .any(|(minimum, maximum)| *minimum > maximum)
        {
            visitor(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for super::NodeControls {
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
impl ValidateLinks for super::TextureControls<'_> {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for super::Geometry {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (clip, bounds, radius) in &self.sequence_bounds {
            clip.visit_links(visitor);
            if !radius.is_finite()
                || *radius < 0.
                || bounds.iter().flatten().any(|value| !value.is_finite())
                || bounds[0]
                    .iter()
                    .zip(bounds[1])
                    .any(|(minimum, maximum)| *minimum > maximum)
            {
                visitor(Link::InvalidValue);
            }
        }
        if self
            .unassigned_bounds
            .iter()
            .any(|bounds| bounds.validate().is_err())
        {
            visitor(Link::InvalidValue);
        }
    }
}
