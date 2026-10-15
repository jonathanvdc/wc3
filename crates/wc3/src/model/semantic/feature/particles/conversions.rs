//! Checked projection of shared components into constrained simulation vocabularies.
use super::*;
impl<'a> From<mdlx::Particle<'a>> for ParticleDefinition<'a> {
    fn from(v: mdlx::Particle<'a>) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Animated(v.emission_rate),
                variation: 0.0,
                trigger: ParticleTrigger::Continuous,
                activation: ParticleActivation::Fractional(v.visibility),
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Animated(v.initial_velocity),
                variation: ParticleScalar::Animated(Animated::constant(0.0)),
                gravity: ParticleGravity::Directional(v.gravity),
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Animated(v.life_span),
                variation: ParticleScalar::Animated(Animated::constant(0.0)),
            },
            appearance: ParticleAppearance::Resource,
            details: ParticleDetails::Model(ModelParticle {
                resource: v.resource,
                longitude: v.longitude,
                latitude: v.latitude,
                model_resource: v.model_resource,
                text_resource: v.text_resource,
                unclassified: v.unclassified,
            }),
        }
    }
}
impl<'a> TryFrom<ParticleDefinition<'a>> for mdlx::Particle<'a> {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from(v: ParticleDefinition<'a>) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.rate, ParticleScalar::Animated(_))
            && matches!(&v.emission.activation, ParticleActivation::Fractional(_))
            && matches!(&v.motion.speed, ParticleScalar::Animated(_))
            && matches!(&v.motion.gravity, ParticleGravity::Directional(_))
            && matches!(&v.lifetime.duration, ParticleScalar::Animated(_))
            && v.emission.variation == 0.0
            && matches!(&v.motion.variation, ParticleScalar::Animated(a) if a.property.is_none() && a.default == 0.0)
            && matches!(&v.lifetime.variation, ParticleScalar::Animated(a) if a.property.is_none() && a.default == 0.0)
            && matches!(v.emission.trigger, ParticleTrigger::Continuous))
            || !matches!(&v.details, ParticleDetails::Model(_))
            || !matches!(&v.appearance, ParticleAppearance::Resource)
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Animated(emission_rate) = v.emission.rate else {
            unreachable!()
        };
        let ParticleActivation::Fractional(visibility) = v.emission.activation else {
            unreachable!()
        };
        let ParticleScalar::Animated(initial_velocity) = v.motion.speed else {
            unreachable!()
        };
        let ParticleGravity::Directional(gravity) = v.motion.gravity else {
            unreachable!()
        };
        let ParticleScalar::Animated(life_span) = v.lifetime.duration else {
            unreachable!()
        };
        let ParticleDetails::Model(d) = v.details else {
            unreachable!()
        };
        Ok(Self {
            resource: d.resource,
            emission_rate,
            gravity,
            longitude: d.longitude,
            latitude: d.latitude,
            life_span,
            initial_velocity,
            visibility,
            model_resource: d.model_resource,
            text_resource: d.text_resource,
            unclassified: d.unclassified,
        })
    }
}
impl<'a> From<mdlx::Particle2> for ParticleDefinition<'a> {
    fn from(v: mdlx::Particle2) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Animated(v.emission_rate),
                variation: 0.0,
                trigger: if v.squirt {
                    ParticleTrigger::KeyedBurst
                } else {
                    ParticleTrigger::Continuous
                },
                activation: ParticleActivation::Fractional(v.visibility),
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Animated(v.speed),
                variation: ParticleScalar::Animated(v.variation),
                gravity: ParticleGravity::Directional(v.gravity),
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Animated(Animated::constant(v.life_span)),
                variation: ParticleScalar::Animated(Animated::constant(0.0)),
            },
            appearance: ParticleAppearance::Staged(StagedParticleAppearance {
                middle_time: v.middle_time,
                colors: v.colors,
                alpha: v.alpha,
                scale: v.scale,
                uv_intervals: v.uv_intervals,
            }),
            details: ParticleDetails::Atlas(Box::new(AtlasParticle {
                model_space: v.model_space,
                unclassified: v.unclassified,
                rendering: AtlasParticleRendering {
                    blend: v.blend,
                    atlas: v.atlas,
                    frames: v.frames,
                    tail_length: v.tail_length,
                    texture: v.texture,
                    priority_plane: v.priority_plane,
                    replaceable: v.replaceable,
                    xy_quad: v.xy_quad,
                    sort_far_z: v.sort_far_z,
                    unlit: v.unlit,
                    unfogged: v.unfogged,
                },
                shape: AtlasParticleShape {
                    latitude: v.latitude,
                    length: v.length,
                    width: v.width,
                    line_emitter: v.line_emitter,
                },
            })),
        }
    }
}
impl<'a> TryFrom<ParticleDefinition<'a>> for mdlx::Particle2 {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from(v: ParticleDefinition<'a>) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.rate, ParticleScalar::Animated(_))
            && matches!(&v.emission.activation, ParticleActivation::Fractional(_))
            && matches!(&v.motion.speed, ParticleScalar::Animated(_))
            && matches!(&v.motion.variation, ParticleScalar::Animated(_))
            && matches!(&v.motion.gravity, ParticleGravity::Directional(_))
            && matches!(&v.lifetime.duration, ParticleScalar::Animated(a) if a.property.is_none())
            && matches!(
                &v.emission.trigger,
                ParticleTrigger::Continuous | ParticleTrigger::KeyedBurst
            )
            && v.emission.variation == 0.0
            && matches!(&v.lifetime.variation, ParticleScalar::Animated(a) if a.property.is_none() && a.default == 0.0))
            || !matches!(&v.details, ParticleDetails::Atlas(_))
            || !matches!(&v.appearance, ParticleAppearance::Staged(_))
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Animated(emission_rate) = v.emission.rate else {
            unreachable!()
        };
        let ParticleActivation::Fractional(visibility) = v.emission.activation else {
            unreachable!()
        };
        let ParticleScalar::Animated(speed) = v.motion.speed else {
            unreachable!()
        };
        let ParticleScalar::Animated(variation) = v.motion.variation else {
            unreachable!()
        };
        let ParticleGravity::Directional(gravity) = v.motion.gravity else {
            unreachable!()
        };
        let ParticleScalar::Animated(life_span) = v.lifetime.duration else {
            unreachable!()
        };
        let life_span = life_span.default;
        let squirt = matches!(v.emission.trigger, ParticleTrigger::KeyedBurst);
        let ParticleAppearance::Staged(appearance) = v.appearance else {
            unreachable!()
        };
        let ParticleDetails::Atlas(d) = v.details else {
            unreachable!()
        };
        Ok(Self {
            speed,
            variation,
            latitude: d.shape.latitude,
            gravity,
            life_span,
            emission_rate,
            length: d.shape.length,
            width: d.shape.width,
            blend: d.rendering.blend,
            atlas: d.rendering.atlas,
            frames: d.rendering.frames,
            tail_length: d.rendering.tail_length,
            middle_time: appearance.middle_time,
            colors: appearance.colors,
            alpha: appearance.alpha,
            scale: appearance.scale,
            uv_intervals: appearance.uv_intervals,
            texture: d.rendering.texture,
            squirt,
            priority_plane: d.rendering.priority_plane,
            replaceable: d.rendering.replaceable,
            visibility,
            model_space: d.model_space,
            xy_quad: d.rendering.xy_quad,
            line_emitter: d.shape.line_emitter,
            sort_far_z: d.rendering.sort_far_z,
            unlit: d.rendering.unlit,
            unfogged: d.rendering.unfogged,
            unclassified: d.unclassified,
        })
    }
}
impl<'a> From<m2::Particle<'a>> for ParticleDefinition<'a> {
    fn from(v: m2::Particle<'a>) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Animated(v.rate),
                variation: v.rate_variation,
                trigger: ParticleTrigger::Continuous,
                activation: ParticleActivation::Boolean(v.enabled),
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Animated(v.speed),
                variation: ParticleScalar::Animated(v.speed_variation),
                gravity: ParticleGravity::Vector(v.gravity),
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Animated(v.lifespan),
                variation: ParticleScalar::Animated(Animated::constant(v.lifespan_variation)),
            },
            appearance: ParticleAppearance::Curves(Box::new(v.appearance)),
            details: ParticleDetails::AgeDriven(Box::new(AgeDrivenParticle {
                id: v.id,
                flags: v.flags,
                twinkle_speed: v.twinkle_speed,
                twinkle_percent: v.twinkle_percent,
                twinkle_scale: v.twinkle_scale,
                inherit_velocity: v.inherit_velocity,
                drag: v.drag,
                spin: v.spin,
                tumble_minimum: v.tumble_minimum,
                tumble_maximum: v.tumble_maximum,
                wind: v.wind,
                wind_time: v.wind_time,
                follow: v.follow,
                rendering: AgeDrivenParticleRendering {
                    blend: v.blend,
                    color_index: v.color_index,
                    textures: v.textures,
                    model: v.model,
                    child_model: v.child_model,
                    head_tail: v.head_tail,
                    texture_scale: v.texture_scale,
                    texture_tile_rotation: v.texture_tile_rotation,
                    atlas: v.atlas,
                    tail_length: v.tail_length,
                    scroll: v.scroll,
                },
                shape: AgeDrivenParticleShape {
                    generator: v.generator,
                    legacy_shape: v.legacy_shape,
                    vertical_range: v.vertical_range,
                    horizontal_range: v.horizontal_range,
                    area_width: v.area_width,
                    area_length: v.area_length,
                    z_source: v.z_source,
                    spline: v.spline,
                },
            })),
        }
    }
}
impl<'a> TryFrom<ParticleDefinition<'a>> for m2::Particle<'a> {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from(v: ParticleDefinition<'a>) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.rate, ParticleScalar::Animated(_))
            && matches!(&v.emission.activation, ParticleActivation::Boolean(_))
            && matches!(&v.motion.speed, ParticleScalar::Animated(_))
            && matches!(&v.motion.variation, ParticleScalar::Animated(_))
            && matches!(&v.motion.gravity, ParticleGravity::Vector(_))
            && matches!(&v.lifetime.duration, ParticleScalar::Animated(_))
            && matches!(&v.lifetime.variation, ParticleScalar::Animated(a) if a.property.is_none())
            && matches!(v.emission.trigger, ParticleTrigger::Continuous))
            || !matches!(&v.details, ParticleDetails::AgeDriven(_))
            || !matches!(&v.appearance, ParticleAppearance::Curves(_))
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Animated(rate) = v.emission.rate else {
            unreachable!()
        };
        let ParticleActivation::Boolean(enabled) = v.emission.activation else {
            unreachable!()
        };
        let ParticleScalar::Animated(speed) = v.motion.speed else {
            unreachable!()
        };
        let ParticleScalar::Animated(speed_variation) = v.motion.variation else {
            unreachable!()
        };
        let ParticleGravity::Vector(gravity) = v.motion.gravity else {
            unreachable!()
        };
        let ParticleScalar::Animated(lifespan) = v.lifetime.duration else {
            unreachable!()
        };
        let ParticleScalar::Animated(lifespan_variation) = v.lifetime.variation else {
            unreachable!()
        };
        let lifespan_variation = lifespan_variation.default;
        let ParticleAppearance::Curves(appearance) = v.appearance else {
            unreachable!()
        };
        let ParticleDetails::AgeDriven(d) = v.details else {
            unreachable!()
        };
        Ok(Self {
            id: d.id,
            flags: d.flags,
            blend: d.rendering.blend,
            generator: d.shape.generator,
            color_index: d.rendering.color_index,
            textures: d.rendering.textures,
            model: d.rendering.model,
            child_model: d.rendering.child_model,
            legacy_shape: d.shape.legacy_shape,
            head_tail: d.rendering.head_tail,
            texture_scale: d.rendering.texture_scale,
            texture_tile_rotation: d.rendering.texture_tile_rotation,
            atlas: d.rendering.atlas,
            speed,
            speed_variation,
            vertical_range: d.shape.vertical_range,
            horizontal_range: d.shape.horizontal_range,
            gravity,
            lifespan,
            lifespan_variation,
            rate,
            rate_variation: v.emission.variation,
            area_width: d.shape.area_width,
            area_length: d.shape.area_length,
            z_source: d.shape.z_source,
            appearance: *appearance,
            tail_length: d.rendering.tail_length,
            twinkle_speed: d.twinkle_speed,
            twinkle_percent: d.twinkle_percent,
            twinkle_scale: d.twinkle_scale,
            inherit_velocity: d.inherit_velocity,
            drag: d.drag,
            spin: d.spin,
            tumble_minimum: d.tumble_minimum,
            tumble_maximum: d.tumble_maximum,
            wind: d.wind,
            wind_time: d.wind_time,
            follow: d.follow,
            spline: d.shape.spline,
            enabled,
            scroll: d.rendering.scroll,
        })
    }
}
impl<'a> From<m3::Particle<'a>> for ParticleDefinition<'a> {
    fn from(v: m3::Particle<'a>) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Programmed(v.emission_rate),
                variation: 0.0,
                trigger: ParticleTrigger::Programmed(v.squirt_amount),
                activation: ParticleActivation::Programmed,
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Programmed(v.initial_speed),
                variation: ParticleScalar::Programmed(v.initial_speed_random),
                gravity: ParticleGravity::Programmed {
                    x: v.gravity_x,
                    y: v.gravity_y,
                    magnitude: v.gravity,
                },
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Programmed(v.lifetime),
                variation: ParticleScalar::Programmed(v.lifetime_random),
            },
            appearance: ParticleAppearance::Programmed(Box::new(ProgrammedAge {
                size_mid_time: v.size_mid_time,
                color_mid_time: v.color_mid_time,
                alpha_mid_time: v.alpha_mid_time,
                rotation_mid_time: v.rotation_mid_time,
                size_mid_hold_time: v.size_mid_hold_time,
                color_mid_hold_time: v.color_mid_hold_time,
                alpha_mid_hold_time: v.alpha_mid_hold_time,
                rotation_mid_hold_time: v.rotation_mid_hold_time,
                size_animation: v.size_animation,
                rotation_animation: v.rotation_animation,
                color_start: v.color_start,
                color_mid: v.color_mid,
                color_end: v.color_end,
            })),
            details: ParticleDetails::Programmed(Box::new(ProgrammedParticle {
                forces: ProgrammedForces {
                    mass: v.mass,
                    mass_random: v.mass_random,
                    mass_size_multiplier: v.mass_size_multiplier,
                    local_forces: v.local_forces,
                    world_forces: v.world_forces,
                    local_forces_fallback: v.local_forces_fallback,
                    world_forces_fallback: v.world_forces_fallback,
                    world_forces_mass_multiplier: v.world_forces_mass_multiplier,
                },
                noise: ProgrammedNoise {
                    noise_amplitude: v.noise_amplitude,
                    noise_frequency: v.noise_frequency,
                    noise_coherence: v.noise_coherence,
                    noise_edge: v.noise_edge,
                },
                material: v.material,
                additional_flags: v.additional_flags,
                kill_radius: v.kill_radius,
                index_plus_length: v.index_plus_length,
                max_particles: v.max_particles,
                emitter_shape: v.emitter_shape,
                shape_outer: v.shape_outer,
                shape_inner: v.shape_inner,
                outer_radius: v.outer_radius,
                inner_radius: v.inner_radius,
                shape_sections: v.shape_sections,
                velocity_type: v.velocity_type,
                size_random_enable: v.size_random_enable,
                size_random_animation: v.size_random_animation,
                rotation_random_enable: v.rotation_random_enable,
                rotation_random_animation: v.rotation_random_animation,
                color_random_enable: v.color_random_enable,
                color_start_random: v.color_start_random,
                color_mid_random: v.color_mid_random,
                color_end_random: v.color_end_random,
                alpha_random_enable: v.alpha_random_enable,
                flipbook_start_init_index: v.flipbook_start_init_index,
                flipbook_start_stop_index: v.flipbook_start_stop_index,
                flipbook_end_init_index: v.flipbook_end_init_index,
                flipbook_end_stop_index: v.flipbook_end_stop_index,
                flipbook_mid_time: v.flipbook_mid_time,
                flipbook_columns: v.flipbook_columns,
                flipbook_rows: v.flipbook_rows,
                flipbook_column_fraction: v.flipbook_column_fraction,
                flipbook_row_fraction: v.flipbook_row_fraction,
                bounce: v.bounce,
                friction: v.friction,
                collision_spawn: v.collision_spawn,
                collision_spawn_min: v.collision_spawn_min,
                collision_spawn_max: v.collision_spawn_max,
                collision_spawn_chance: v.collision_spawn_chance,
                collision_spawn_energy: v.collision_spawn_energy,
                collision_die_bounce: v.collision_die_bounce,
                instance_type: v.instance_type,
                tail_length: v.tail_length,
                instance_angle: v.instance_angle,
                instance_distance: v.instance_distance,
                pitch: v.pitch,
                yaw: v.yaw,
                speed: v.speed,
                size: v.size,
                alpha: v.alpha,
                color: v.color,
                rotation: v.rotation,
                horizontal: v.horizontal,
                vertical: v.vertical,
                particle_velocity: v.particle_velocity,
                phase_shift: v.phase_shift,
                flags: v.flags,
                rotation_flags: v.rotation_flags,
                color_smoothing: v.color_smoothing,
                size_smoothing: v.size_smoothing,
                rotation_smoothing: v.rotation_smoothing,
                alpha_threshold: v.alpha_threshold,
                uv_offset: v.uv_offset,
                uv_angle: v.uv_angle,
                uv_tiling: v.uv_tiling,
                spline_points: v.spline_points,
                wind_multiplier: v.wind_multiplier,
                lod_reduce: v.lod_reduce,
                lod_cut: v.lod_cut,
                lower_bound: v.lower_bound,
                upper_bound: v.upper_bound,
                trail: v.trail,
                trail_chance: v.trail_chance,
                trail_emission_rate: v.trail_emission_rate,
                projection: v.projection,
                splat_chance: v.splat_chance,
                models: v.models,
                spawn_ribbon_on_bounce_chance: v.spawn_ribbon_on_bounce_chance,
                ribbon: v.ribbon,
                noise_smoothness: v.noise_smoothness,
                direction: ProgrammedDirection {
                    initial_yaw: v.initial_yaw,
                    initial_pitch: v.initial_pitch,
                    initial_horizontal: v.initial_horizontal,
                    initial_vertical: v.initial_vertical,
                    drag: v.drag,
                },
            })),
        }
    }
}
impl<'a> TryFrom<ParticleDefinition<'a>> for m3::Particle<'a> {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from(v: ParticleDefinition<'a>) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.activation, ParticleActivation::Programmed)
            && matches!(&v.emission.rate, ParticleScalar::Programmed(_))
            && matches!(&v.emission.trigger, ParticleTrigger::Programmed(_))
            && matches!(&v.motion.speed, ParticleScalar::Programmed(_))
            && matches!(&v.motion.variation, ParticleScalar::Programmed(_))
            && matches!(&v.lifetime.duration, ParticleScalar::Programmed(_))
            && matches!(&v.lifetime.variation, ParticleScalar::Programmed(_))
            && matches!(&v.motion.gravity, ParticleGravity::Programmed { .. })
            && v.emission.variation == 0.0)
            || !matches!(&v.details, ParticleDetails::Programmed(_))
            || !matches!(&v.appearance, ParticleAppearance::Programmed(_))
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Programmed(emission_rate) = v.emission.rate else {
            unreachable!()
        };
        let ParticleTrigger::Programmed(squirt_amount) = v.emission.trigger else {
            unreachable!()
        };
        let ParticleScalar::Programmed(initial_speed) = v.motion.speed else {
            unreachable!()
        };
        let ParticleScalar::Programmed(initial_speed_random) = v.motion.variation else {
            unreachable!()
        };
        let ParticleScalar::Programmed(lifetime) = v.lifetime.duration else {
            unreachable!()
        };
        let ParticleScalar::Programmed(lifetime_random) = v.lifetime.variation else {
            unreachable!()
        };
        let ParticleGravity::Programmed {
            x: gravity_x,
            y: gravity_y,
            magnitude: gravity,
        } = v.motion.gravity
        else {
            unreachable!()
        };
        let ParticleAppearance::Programmed(appearance) = v.appearance else {
            unreachable!()
        };
        let ParticleDetails::Programmed(d) = v.details else {
            unreachable!()
        };
        Ok(Self {
            material: d.material,
            additional_flags: d.additional_flags,
            initial_speed,
            initial_speed_random,
            initial_yaw: d.direction.initial_yaw,
            initial_pitch: d.direction.initial_pitch,
            initial_horizontal: d.direction.initial_horizontal,
            initial_vertical: d.direction.initial_vertical,
            lifetime,
            lifetime_random,
            kill_radius: d.kill_radius,
            gravity_x,
            gravity_y,
            gravity,
            size_mid_time: appearance.size_mid_time,
            color_mid_time: appearance.color_mid_time,
            alpha_mid_time: appearance.alpha_mid_time,
            rotation_mid_time: appearance.rotation_mid_time,
            size_mid_hold_time: appearance.size_mid_hold_time,
            color_mid_hold_time: appearance.color_mid_hold_time,
            alpha_mid_hold_time: appearance.alpha_mid_hold_time,
            rotation_mid_hold_time: appearance.rotation_mid_hold_time,
            size_animation: appearance.size_animation,
            rotation_animation: appearance.rotation_animation,
            color_start: appearance.color_start,
            color_mid: appearance.color_mid,
            color_end: appearance.color_end,
            drag: d.direction.drag,
            mass: d.forces.mass,
            mass_random: d.forces.mass_random,
            mass_size_multiplier: d.forces.mass_size_multiplier,
            local_forces: d.forces.local_forces,
            world_forces: d.forces.world_forces,
            local_forces_fallback: d.forces.local_forces_fallback,
            world_forces_fallback: d.forces.world_forces_fallback,
            world_forces_mass_multiplier: d.forces.world_forces_mass_multiplier,
            noise_amplitude: d.noise.noise_amplitude,
            noise_frequency: d.noise.noise_frequency,
            noise_coherence: d.noise.noise_coherence,
            noise_edge: d.noise.noise_edge,
            index_plus_length: d.index_plus_length,
            max_particles: d.max_particles,
            emission_rate,
            emitter_shape: d.emitter_shape,
            shape_outer: d.shape_outer,
            shape_inner: d.shape_inner,
            outer_radius: d.outer_radius,
            inner_radius: d.inner_radius,
            shape_sections: d.shape_sections,
            velocity_type: d.velocity_type,
            size_random_enable: d.size_random_enable,
            size_random_animation: d.size_random_animation,
            rotation_random_enable: d.rotation_random_enable,
            rotation_random_animation: d.rotation_random_animation,
            color_random_enable: d.color_random_enable,
            color_start_random: d.color_start_random,
            color_mid_random: d.color_mid_random,
            color_end_random: d.color_end_random,
            alpha_random_enable: d.alpha_random_enable,
            squirt_amount,
            flipbook_start_init_index: d.flipbook_start_init_index,
            flipbook_start_stop_index: d.flipbook_start_stop_index,
            flipbook_end_init_index: d.flipbook_end_init_index,
            flipbook_end_stop_index: d.flipbook_end_stop_index,
            flipbook_mid_time: d.flipbook_mid_time,
            flipbook_columns: d.flipbook_columns,
            flipbook_rows: d.flipbook_rows,
            flipbook_column_fraction: d.flipbook_column_fraction,
            flipbook_row_fraction: d.flipbook_row_fraction,
            bounce: d.bounce,
            friction: d.friction,
            collision_spawn: d.collision_spawn,
            collision_spawn_min: d.collision_spawn_min,
            collision_spawn_max: d.collision_spawn_max,
            collision_spawn_chance: d.collision_spawn_chance,
            collision_spawn_energy: d.collision_spawn_energy,
            collision_die_bounce: d.collision_die_bounce,
            instance_type: d.instance_type,
            tail_length: d.tail_length,
            instance_angle: d.instance_angle,
            instance_distance: d.instance_distance,
            pitch: d.pitch,
            yaw: d.yaw,
            speed: d.speed,
            size: d.size,
            alpha: d.alpha,
            color: d.color,
            rotation: d.rotation,
            horizontal: d.horizontal,
            vertical: d.vertical,
            particle_velocity: d.particle_velocity,
            phase_shift: d.phase_shift,
            flags: d.flags,
            rotation_flags: d.rotation_flags,
            color_smoothing: d.color_smoothing,
            size_smoothing: d.size_smoothing,
            rotation_smoothing: d.rotation_smoothing,
            alpha_threshold: d.alpha_threshold,
            uv_offset: d.uv_offset,
            uv_angle: d.uv_angle,
            uv_tiling: d.uv_tiling,
            spline_points: d.spline_points,
            wind_multiplier: d.wind_multiplier,
            lod_reduce: d.lod_reduce,
            lod_cut: d.lod_cut,
            lower_bound: d.lower_bound,
            upper_bound: d.upper_bound,
            trail: d.trail,
            trail_chance: d.trail_chance,
            trail_emission_rate: d.trail_emission_rate,
            projection: d.projection,
            splat_chance: d.splat_chance,
            models: d.models,
            spawn_ribbon_on_bounce_chance: d.spawn_ribbon_on_bounce_chance,
            ribbon: d.ribbon,
            noise_smoothness: d.noise_smoothness,
        })
    }
}
