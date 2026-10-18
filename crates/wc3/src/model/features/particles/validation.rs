//! Traversal of authored bindings, including controls rejected by constrained exporters.
use super::*;
impl ValidateLinks for ParticleScalar {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Property(x) => x.visit_links(v),
        }
    }
}
impl<E: ValidateLinks, M: ValidateLinks, L: ValidateLinks, A: ValidateLinks, D: ValidateLinks>
    ValidateLinks for ParticleComposition<E, M, L, A, D>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.emission.visit_links(visitor);
        self.motion.visit_links(visitor);
        self.lifetime.visit_links(visitor);
        self.appearance.visit_links(visitor);
        self.details.visit_links(visitor);
    }
}
impl<R: ValidateLinks, T: ValidateLinks, A: ValidateLinks, V: ValidateLinks> ValidateLinks
    for ParticleEmission<R, T, A, V>
{
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.rate.visit_links(visitor);
        self.trigger.visit_links(visitor);
        self.activation.visit_links(visitor);
        self.variation.visit_links(visitor);
    }
}
impl ValidateLinks for ParticleTrigger {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
impl ValidateLinks for ParticleActivation {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Fractional(value) => value.visit_links(visitor),
            Self::Boolean(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for ParticleDetails<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Model(value) => value.visit_links(visitor),
            Self::Atlas(value) => value.visit_links(visitor),
            Self::AgeDriven(value) => value.visit_links(visitor),
        }
    }
}

impl ValidateLinks for ParticleAgeCurves<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.scale_variation.visit_links(visitor);
    }
}
impl ValidateLinks for AtlasParticle {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.rendering.visit_links(v);
        self.shape.visit_links(v);
    }
}
impl ValidateLinks for AgeDrivenParticle<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        if !self.twinkle_speed.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.twinkle_percent.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.inherit_velocity.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.drag.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.wind_time.is_finite() {
            v(Link::InvalidValue);
        }
        self.rendering.visit_links(v);
        self.shape.visit_links(v);
    }
}

impl ValidateLinks for StagedParticleAppearance {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        if !self.middle_time.is_finite() {
            v(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for AtlasParticleRendering {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        if !self.tail_length.is_finite() {
            v(Link::InvalidValue);
        }
        self.texture.visit_links(v);
    }
}
impl ValidateLinks for AtlasParticleShape {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.latitude.visit_links(v);
        self.length.visit_links(v);
        self.width.visit_links(v);
    }
}
impl ValidateLinks for AgeDrivenParticleRendering {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.model.visit_links(v);
        self.child_model.visit_links(v);
        self.textures.visit_links(v);
        if !self.tail_length.is_finite() {
            v(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for AgeDrivenParticleShape<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.vertical_range.visit_links(v);
        self.horizontal_range.visit_links(v);
        self.area_width.visit_links(v);
        self.area_length.visit_links(v);
        self.z_source.visit_links(v);
    }
}
impl ValidateLinks for ParticleGravity {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Directional(value) => value.visit_links(v),
            Self::Vector(value) => value.visit_links(v),
        }
    }
}
impl<S: ValidateLinks, V: ValidateLinks, G: ValidateLinks> ValidateLinks
    for ParticleMotion<S, V, G>
{
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.speed.visit_links(v);
        self.variation.visit_links(v);
        self.gravity.visit_links(v);
    }
}
impl<D: ValidateLinks, V: ValidateLinks> ValidateLinks for ParticleLifetime<D, V> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.duration.visit_links(v);
        self.variation.visit_links(v);
    }
}
impl ValidateLinks for ModelParticle {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.resource.visit_links(visitor);
        self.longitude.visit_links(visitor);
        self.latitude.visit_links(visitor);
    }
}

impl ValidateLinks for ParticleAppearance<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Resource => {}
            Self::Staged(value) => value.visit_links(visitor),

            Self::Curves(value) => {
                // Curve constructors already check their keys, values and interpolation controls.
                if value.scale_variation.iter().any(|x| !x.is_finite()) {
                    visitor(Link::InvalidValue);
                }
            }
        }
    }
}
