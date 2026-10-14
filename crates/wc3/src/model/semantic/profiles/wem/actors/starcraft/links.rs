use super::*;
use crate::model::semantic::profiles::wem::SectionId;
use crate::model::semantic::{Error, FeatureKind, Link, ValidateLinks};
impl ValidateLinks for Variation {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.amplitude.visit_links(v);
        self.frequency.visit_links(v);
    }
}
impl ValidateLinks for Particle<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.material_slot.visit_links(v);
        self.initial_speed.visit_links(v);
        self.initial_speed_random.visit_links(v);
        self.initial_yaw.visit_links(v);
        self.initial_pitch.visit_links(v);
        self.initial_horizontal.visit_links(v);
        self.initial_vertical.visit_links(v);
        self.lifetime.visit_links(v);
        self.lifetime_random.visit_links(v);
        if !self.kill_radius.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.gravity.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.size_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.color_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.alpha_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.rotation_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.size_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.color_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.alpha_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.rotation_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        self.size_animation.visit_links(v);
        self.rotation_animation.visit_links(v);
        self.color_start.visit_links(v);
        self.color_mid.visit_links(v);
        self.color_end.visit_links(v);
        if !self.drag.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.mass.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.mass_random.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.mass_size_multiplier.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.world_forces_mass_multiplier.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_amplitude.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_frequency.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_coherence.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_edge.is_finite() {
            v(Link::InvalidValue);
        }
        self.emission_rate.visit_links(v);
        self.shape_outer.visit_links(v);
        self.shape_inner.visit_links(v);
        self.outer_radius.visit_links(v);
        self.inner_radius.visit_links(v);
        self.size_random_animation.visit_links(v);
        self.rotation_random_animation.visit_links(v);
        self.color_start_random.visit_links(v);
        self.color_mid_random.visit_links(v);
        self.color_end_random.visit_links(v);
        self.squirt_amount.visit_links(v);
        if !self.flipbook_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.flipbook_column_fraction.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.flipbook_row_fraction.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.bounce.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.friction.is_finite() {
            v(Link::InvalidValue);
        }
        if let Some(id) = self.collision_spawn {
            v(Link::FeatureAlternatives {
                id,
                kinds: &[
                    FeatureKind::WemStarcraftParticle,
                    FeatureKind::WemStarcraftParticleCopy,
                ],
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
        if self.instance_angle.iter().any(|x| !x.is_finite()) {
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
        for value in self.spline_points.iter() {
            value.visit_links(v);
        }
        if !self.wind_multiplier.is_finite() {
            v(Link::InvalidValue);
        }
        self.lower_bound.visit_links(v);
        self.upper_bound.visit_links(v);
        if let Some(id) = self.trail_link {
            v(Link::FeatureAlternatives {
                id,
                kinds: &[
                    FeatureKind::WemStarcraftParticle,
                    FeatureKind::WemStarcraftParticleCopy,
                ],
            });
        }
        if !self.trail_chance.is_finite() {
            v(Link::InvalidValue);
        }
        self.trail_emission_rate.visit_links(v);
        if !self.splat_chance.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.spawn_ribbon_on_bounce_chance.is_finite() {
            v(Link::InvalidValue);
        }
        if let Some(id) = self.ribbon_link {
            v(Link::TypedFeature {
                id,
                kind: FeatureKind::WemStarcraftRibbon,
            });
        }
        if !self.noise_smoothness.is_finite() {
            v(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for Ribbon<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.material_slot.visit_links(v);
        self.initial_speed.visit_links(v);
        self.initial_speed_random.visit_links(v);
        self.initial_yaw.visit_links(v);
        self.initial_pitch.visit_links(v);
        self.initial_horizontal.visit_links(v);
        self.initial_vertical.visit_links(v);
        self.lifetime.visit_links(v);
        self.lifetime_random.visit_links(v);
        if !self.gravity_x.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.gravity_y.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.gravity.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.size_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.color_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.alpha_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.rotation_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.size_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.color_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.alpha_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.rotation_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        self.size_animation.visit_links(v);
        self.rotation_animation.visit_links(v);
        self.color_start.visit_links(v);
        self.color_mid.visit_links(v);
        self.color_end.visit_links(v);
        if !self.drag.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.mass.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.mass_random.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.mass_size_multiplier.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.world_forces_mass_multiplier.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_amplitude.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_frequency.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_coherence.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_edge.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.divisions.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.inner_radius.is_finite() {
            v(Link::InvalidValue);
        }
        self.max_length.visit_links(v);
        for point in self.spline_points.iter() {
            point.visit_links(v);
        }
        self.active.visit_links(v);
        if !self.friction.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.bounce.is_finite() {
            v(Link::InvalidValue);
        }
        self.yaw.visit_links(v);
        self.pitch.visit_links(v);
        self.speed.visit_links(v);
        self.size.visit_links(v);
        self.alpha.visit_links(v);
        self.particle_velocity.visit_links(v);
        self.overlay.visit_links(v);
    }
}
impl ValidateLinks for RibbonPoint {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.node.visit_links(v);
        if self.emission_offset.iter().any(|x| !x.is_finite()) {
            v(Link::InvalidValue);
        }
        if self.emission_vector.iter().any(|x| !x.is_finite()) {
            v(Link::InvalidValue);
        }
        self.velocity.visit_links(v);
        self.velocity_base_factor.visit_links(v);
        self.velocity_end_factor.visit_links(v);
        self.yaw_variation.visit_links(v);
        self.pitch_variation.visit_links(v);
        self.velocity_variation.visit_links(v);
        self.yaw.visit_links(v);
        self.pitch.visit_links(v);
        if !self.emission_vector_norm_factor.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.velocity_norm_factor.is_finite() {
            v(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for ParticleCopy {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        v(Link::TypedFeature {
            id: self.source,
            kind: FeatureKind::WemStarcraftParticle,
        });
        self.emission_rate.visit_links(v);
        self.squirt_amount.visit_links(v);
    }
}
impl ValidateLinks for Feature<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        match self {
            Self::Particle(p) => p.visit_links(v),
            Self::ParticleCopy(p) => p.visit_links(v),
            Self::Ribbon(p) => p.visit_links(v),
        }
    }
}

impl Particle<'_> {
    /// Checks that emission geometry belongs to the actual first mesh section arena.
    pub fn validate_sections(
        &self,
        mut exists: impl FnMut(SectionId) -> bool,
    ) -> Result<(), Error> {
        if self.shape_sections.iter().any(|id| !exists(id)) {
            Err(Error::new(
                "wem.starcraft.sections",
                "emission section belongs to another mesh",
            ))
        } else {
            Ok(())
        }
    }
}
