//! Traversal of authored bindings, including controls rejected by constrained exporters.
use super::*;
use crate::model::FeatureKind;
impl ValidateLinks for ParticleScalar {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Property(x) => x.visit_links(v),
            Self::Programmed(x) => x.visit_links(v),
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
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::Programmed(value) = self {
            value.visit_links(visitor);
        }
    }
}
impl ValidateLinks for ParticleActivation {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Fractional(value) => value.visit_links(visitor),
            Self::Boolean(value) => value.visit_links(visitor),
            Self::Programmed => {}
        }
    }
}
impl ValidateLinks for ParticleDetails<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Model(value) => value.visit_links(visitor),
            Self::Atlas(value) => value.visit_links(visitor),
            Self::AgeDriven(value) => value.visit_links(visitor),
            Self::Programmed(value) => value.visit_links(visitor),
        }
    }
}
impl ValidateLinks for ProgrammedParticleGravity {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.magnitude.visit_links(visitor);
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
impl ValidateLinks for ProgrammedParticle<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.forces.visit_links(v);
        self.noise.visit_links(v);
        self.material.visit_links(v);
        if !self.kill_radius.is_finite() {
            v(Link::InvalidValue);
        }
        self.shape_outer.visit_links(v);
        self.shape_inner.visit_links(v);
        self.outer_radius.visit_links(v);
        self.inner_radius.visit_links(v);
        self.shape_sections.visit_links(v);
        self.size_random_animation.visit_links(v);
        self.rotation_random_animation.visit_links(v);
        self.color_start_random.visit_links(v);
        self.color_mid_random.visit_links(v);
        self.color_end_random.visit_links(v);
        if !self.flipbook_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.bounce.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.friction.is_finite() {
            v(Link::InvalidValue);
        }
        if let Some(id) = self.collision_spawn {
            v(Link::TypedFeature {
                id,
                kind: FeatureKind::M3Particle,
            });
        }
        if !self.collision_spawn_chance.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.collision_spawn_energy.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.tail_length.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.instance_distance.is_finite() {
            v(Link::InvalidValue);
        }
        self.pitch.visit_links(v);
        self.yaw.visit_links(v);
        self.speed.visit_links(v);
        self.size.visit_links(v);
        self.alpha.visit_links(v);
        self.color.visit_links(v);
        self.rotation.visit_links(v);
        self.horizontal.visit_links(v);
        self.vertical.visit_links(v);
        self.particle_velocity.visit_links(v);
        self.phase_shift.visit_links(v);
        self.alpha_threshold.visit_links(v);
        self.uv_offset.visit_links(v);
        self.uv_angle.visit_links(v);
        self.uv_tiling.visit_links(v);
        for x in self.spline_points.iter() {
            x.visit_links(v);
        }
        if !self.wind_multiplier.is_finite() {
            v(Link::InvalidValue);
        }
        self.lower_bound.visit_links(v);
        self.upper_bound.visit_links(v);
        if let Some(id) = self.trail {
            v(Link::TypedFeature {
                id,
                kind: FeatureKind::M3Particle,
            });
        }
        if !self.trail_chance.is_finite() {
            v(Link::InvalidValue);
        }
        self.trail_emission_rate.visit_links(v);
        if let Some(id) = self.projection {
            v(Link::TypedFeature {
                id,
                kind: FeatureKind::M3Projection,
            });
        }
        if !self.splat_chance.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.spawn_ribbon_on_bounce_chance.is_finite() {
            v(Link::InvalidValue);
        }
        if let Some(id) = self.ribbon {
            v(Link::TypedFeature {
                id,
                kind: FeatureKind::M3Ribbon,
            });
        }
        if !self.noise_smoothness.is_finite() {
            v(Link::InvalidValue);
        }
        self.direction.visit_links(v);
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
            Self::Programmed { magnitude, .. } => {
                if !magnitude.is_finite() {
                    v(Link::InvalidValue);
                }
            }
            Self::ProgrammedVector { x, y, magnitude } => {
                if [x, y, magnitude]
                    .into_iter()
                    .any(|value| !value.is_finite())
                {
                    v(Link::InvalidValue);
                }
            }
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
            Self::Programmed(value) => value.visit_links(visitor),
            Self::Curves(value) => {
                // Curve constructors already check their keys, values and interpolation controls.
                if value.scale_variation.iter().any(|x| !x.is_finite()) {
                    visitor(Link::InvalidValue);
                }
            }
        }
    }
}
