//! Field decoding keeps initial/reset controls and resolves real graph identities.
use super::super::slots::Slot;
use super::context::Reader;
use crate::model::conversion::Error;
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::profiles::m3 as surface;
use crate::model::semantic::profiles::m3::properties::{Field, FieldPath};
use native::Envelope;
pub(super) fn decode<'view, E: Envelope, D: nr::ParticleEmitterDialect>(
    r: nr::ParticleEmitter<D>,
    reader: &mut Reader<'view, '_, '_, E>,
) -> Result<surface::Particle<'view>, Error>
where
    D::LegacySpeedRandomize: Slot<u32>,
    D::LegacyLifespanRandomize: Slot<u32>,
    D::LegacySizeAnimation: Slot<nr::AnimationValue<[f32; 3]>>,
    D::LegacySizeMidTime: Slot<f32>,
    D::LegacyRotationAnimation: Slot<nr::AnimationValue<[f32; 3]>>,
    D::LegacyRotationMidTime: Slot<f32>,
    D::LegacyColorStart: Slot<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::LegacyColorMid: Slot<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::LegacyColorEnd: Slot<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::LegacyColorMidTime: Slot<f32>,
    D::LegacyAlphaMidTime: Slot<f32>,
    D::LegacyMassRandomize: Slot<u32>,
    D::LegacyWorldSpace: Slot<u32>,
    D::DeprecatedNoiseSmoothness: Slot<f32>,
    D::IndexPlusLength: Slot<u32>,
    D::SizeMidTime: Slot<f32>,
    D::ColorMidTime: Slot<f32>,
    D::AlphaMidTime: Slot<f32>,
    D::RotationMidTime: Slot<f32>,
    D::SizeAnimation: Slot<nr::AnimationValue<[f32; 3]>>,
    D::RotationAnimation: Slot<nr::AnimationValue<[f32; 3]>>,
    D::ColorStart: Slot<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::ColorMid: Slot<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::ColorEnd: Slot<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::MassSizeMultiplier: Slot<f32>,
    D::FlipbookColumnFraction: Slot<f32>,
    D::FlipbookRowFraction: Slot<f32>,
    D::SizeMidHoldTime: Slot<f32>,
    D::ColorMidHoldTime: Slot<f32>,
    D::AlphaMidHoldTime: Slot<f32>,
    D::RotationMidHoldTime: Slot<f32>,
    D::ColorSmoothing: Slot<u32>,
    D::SizeSmoothing: Slot<u32>,
    D::RotationSmoothing: Slot<u32>,
    D::ShapeRegions: Slot<native::Ref<nr::targets::U32>>,
    D::AlphaThreshold: Slot<nr::AnimationValue<f32>>,
    D::UvOffset: Slot<nr::AnimationValue<[f32; 2]>>,
    D::UvAngle: Slot<nr::AnimationValue<[f32; 3]>>,
    D::UvTiling: Slot<nr::AnimationValue<[f32; 2]>>,
    D::InstanceDistance: Slot<f32>,
    D::AdditionalFlags: Slot<u32>,
    D::RotationFlags: Slot<u32>,
    D::PhaseShift: Slot<nr::AnimationValue<f32>>,
    D::SpawnRibbonOnBounceChance: Slot<f32>,
    D::RibbonLinkIndex: Slot<i32>,
    D::WorldForcesMassMultiplier: Slot<f32>,
{
    let material_index = r.material_index;
    let additional_flags = r.additional_flags.slot().unwrap_or_default();
    let initial_speed = r.initial_speed;
    let initial_speed_random = r.initial_speed_random;
    let legacy_speed_randomize = r.legacy_speed_randomize.slot().unwrap_or_default();
    let initial_yaw = r.initial_yaw;
    let initial_pitch = r.initial_pitch;
    let initial_horizontal = r.initial_horizontal;
    let initial_vertical = r.initial_vertical;
    let lifetime = r.lifetime;
    let lifetime_random = r.lifetime_random;
    let legacy_lifespan_randomize = r.legacy_lifespan_randomize.slot().unwrap_or_default();
    let kill_radius = r.kill_radius;
    let gravity_x = r.gravity_x;
    let gravity_y = r.gravity_y;
    let gravity = r.gravity;
    let size_mid_time = r
        .size_mid_time
        .slot()
        .or_else(|| r.legacy_size_mid_time.slot())
        .unwrap_or_default();
    let color_mid_time = r
        .color_mid_time
        .slot()
        .or_else(|| r.legacy_color_mid_time.slot())
        .unwrap_or_default();
    let alpha_mid_time = r
        .alpha_mid_time
        .slot()
        .or_else(|| r.legacy_alpha_mid_time.slot())
        .unwrap_or_default();
    let rotation_mid_time = r
        .rotation_mid_time
        .slot()
        .or_else(|| r.legacy_rotation_mid_time.slot())
        .unwrap_or_default();
    let size_mid_hold_time = r.size_mid_hold_time.slot().unwrap_or_default();
    let color_mid_hold_time = r.color_mid_hold_time.slot().unwrap_or_default();
    let alpha_mid_hold_time = r.alpha_mid_hold_time.slot().unwrap_or_default();
    let rotation_mid_hold_time = r.rotation_mid_hold_time.slot().unwrap_or_default();
    let size_animation = r
        .size_animation
        .slot()
        .or_else(|| r.legacy_size_animation.slot())
        .unwrap_or_default();
    let rotation_animation = r
        .rotation_animation
        .slot()
        .or_else(|| r.legacy_rotation_animation.slot())
        .unwrap_or_default();
    let color_start = r
        .color_start
        .slot()
        .or_else(|| r.legacy_color_start.slot())
        .unwrap_or_default();
    let color_mid = r
        .color_mid
        .slot()
        .or_else(|| r.legacy_color_mid.slot())
        .unwrap_or_default();
    let color_end = r
        .color_end
        .slot()
        .or_else(|| r.legacy_color_end.slot())
        .unwrap_or_default();
    let drag = r.drag;
    let legacy_mass_randomize = r.legacy_mass_randomize.slot().unwrap_or_default();
    let mass = r.mass;
    let mass_random = r.mass_random;
    let mass_size_multiplier = r.mass_size_multiplier.slot().unwrap_or_default();
    let legacy_world_space = r.legacy_world_space.slot().unwrap_or_default();
    let local_forces = r.local_forces;
    let world_forces = r.world_forces;
    let local_forces_fallback = r.local_forces_fallback;
    let world_forces_fallback = r.world_forces_fallback;
    let world_forces_mass_multiplier = r.world_forces_mass_multiplier.slot().unwrap_or_default();
    let noise_amplitude = r.noise_amplitude;
    let noise_frequency = r.noise_frequency;
    let noise_coherence = r.noise_coherence;
    let noise_edge = r.noise_edge;
    let deprecated_noise_smoothness = r.deprecated_noise_smoothness.slot().unwrap_or_default();
    let index_plus_length = r.index_plus_length.slot().unwrap_or_default();
    let max_particles = r.max_particles;
    let emission_rate = r.emission_rate;
    let emitter_shape = r.emitter_shape;
    let shape_outer = r.shape_outer;
    let shape_inner = r.shape_inner;
    let outer_radius = r.outer_radius;
    let inner_radius = r.inner_radius;
    let shape_regions = r.shape_regions.slot().unwrap_or_default();
    let velocity_type = r.velocity_type;
    let size_random_enable = r.size_random_enable;
    let size_random_animation = r.size_random_animation;
    let rotation_random_enable = r.rotation_random_enable;
    let rotation_random_animation = r.rotation_random_animation;
    let color_random_enable = r.color_random_enable;
    let color_start_random = r.color_start_random;
    let color_mid_random = r.color_mid_random;
    let color_end_random = r.color_end_random;
    let alpha_random_enable = r.alpha_random_enable;
    let squirt_amount = r.squirt_amount;
    let flipbook_start_init_index = r.flipbook_start_init_index;
    let flipbook_start_stop_index = r.flipbook_start_stop_index;
    let flipbook_end_init_index = r.flipbook_end_init_index;
    let flipbook_end_stop_index = r.flipbook_end_stop_index;
    let flipbook_mid_time = r.flipbook_mid_time;
    let flipbook_columns = r.flipbook_columns;
    let flipbook_rows = r.flipbook_rows;
    let flipbook_column_fraction = r.flipbook_column_fraction.slot().unwrap_or(f32::INFINITY);
    let flipbook_row_fraction = r.flipbook_row_fraction.slot().unwrap_or(f32::INFINITY);
    let bounce = r.bounce;
    let friction = r.friction;
    let collision_spawn_index = r.collision_spawn_index;
    let collision_spawn_min = r.collision_spawn_min;
    let collision_spawn_max = r.collision_spawn_max;
    let collision_spawn_chance = r.collision_spawn_chance;
    let collision_spawn_energy = r.collision_spawn_energy;
    let collision_die_bounce = r.collision_die_bounce;
    let instance_type = r.instance_type;
    let tail_length = r.tail_length;
    let instance_angle = r.instance_angle;
    let instance_distance = r.instance_distance.slot().unwrap_or_default();
    let pitch_type = r.pitch_type;
    let pitch_amplitude = r.pitch_amplitude;
    let pitch_frequency = r.pitch_frequency;
    let yaw_type = r.yaw_type;
    let yaw_amplitude = r.yaw_amplitude;
    let yaw_frequency = r.yaw_frequency;
    let speed_type = r.speed_type;
    let speed_amplitude = r.speed_amplitude;
    let speed_frequency = r.speed_frequency;
    let size_type = r.size_type;
    let size_amplitude = r.size_amplitude;
    let size_frequency = r.size_frequency;
    let alpha_type = r.alpha_type;
    let alpha_amplitude = r.alpha_amplitude;
    let alpha_frequency = r.alpha_frequency;
    let color_type = r.color_type;
    let color_amplitude = r.color_amplitude;
    let color_frequency = r.color_frequency;
    let rotation_type = r.rotation_type;
    let rotation_amplitude = r.rotation_amplitude;
    let rotation_frequency = r.rotation_frequency;
    let horizontal_type = r.horizontal_type;
    let horizontal_amplitude = r.horizontal_amplitude;
    let horizontal_frequency = r.horizontal_frequency;
    let vertical_type = r.vertical_type;
    let vertical_amplitude = r.vertical_amplitude;
    let vertical_frequency = r.vertical_frequency;
    let particle_velocity = r.particle_velocity;
    let phase_shift = r.phase_shift.slot().unwrap_or_default();
    let flags = r.flags;
    let rotation_flags = r.rotation_flags.slot().unwrap_or_default();
    let color_smoothing = r.color_smoothing.slot().unwrap_or_default();
    let size_smoothing = r.size_smoothing.slot().unwrap_or_default();
    let rotation_smoothing = r.rotation_smoothing.slot().unwrap_or_default();
    let alpha_threshold = r.alpha_threshold.slot().unwrap_or_default();
    let uv_offset = r.uv_offset.slot().unwrap_or_default();
    let uv_angle = r.uv_angle.slot().unwrap_or_default();
    let uv_tiling = r.uv_tiling.slot().unwrap_or_default();
    let spline_line_data = r.spline_line_data;
    let wind_multiplier = r.wind_multiplier;
    let lod_reduce = r.lod_reduce;
    let lod_cut = r.lod_cut;
    let lower_bound = r.lower_bound;
    let upper_bound = r.upper_bound;
    let trail_link_index = r.trail_link_index;
    let trail_chance = r.trail_chance;
    let trail_emission_rate = r.trail_emission_rate;
    let splat_projection_index = r.splat_projection_index;
    let splat_chance = r.splat_chance;
    let model_paths = r.model_paths;

    let spawn_ribbon_on_bounce_chance = r.spawn_ribbon_on_bounce_chance.slot().unwrap_or_default();
    let ribbon_link_index = r.ribbon_link_index.slot().unwrap_or(-1);
    let mut additional_flags = additional_flags;
    for (value, bit) in [
        (legacy_speed_randomize, 0),
        (legacy_lifespan_randomize, 1),
        (legacy_mass_randomize, 2),
        (legacy_world_space, 3),
    ] {
        if value > 1 {
            reader.context.unsupported(
                "m3.particle.legacy_toggle",
                "legacy randomization/world-space word is not a known boolean",
            )?;
        }
        if value != 0 {
            additional_flags |= 1 << bit;
        }
    }
    Ok(surface::Particle {
        material: Some(reader.material(material_index)?),
        additional_flags: surface::values::ParticleAdditionalFlags(additional_flags),
        initial_speed: reader.property(
            &FieldPath::root(Field::Particle).child(Field::InitialSpeed),
            initial_speed,
        ),
        initial_speed_random: reader.property(
            &FieldPath::root(Field::Particle).child(Field::InitialSpeedRandom),
            initial_speed_random,
        ),
        initial_yaw: reader.property(
            &FieldPath::root(Field::Particle).child(Field::InitialYaw),
            initial_yaw,
        ),
        initial_pitch: reader.property(
            &FieldPath::root(Field::Particle).child(Field::InitialPitch),
            initial_pitch,
        ),
        initial_horizontal: reader.property(
            &FieldPath::root(Field::Particle).child(Field::InitialHorizontal),
            initial_horizontal,
        ),
        initial_vertical: reader.property(
            &FieldPath::root(Field::Particle).child(Field::InitialVertical),
            initial_vertical,
        ),
        lifetime: reader.property(
            &FieldPath::root(Field::Particle).child(Field::Lifetime),
            lifetime,
        ),
        lifetime_random: reader.property(
            &FieldPath::root(Field::Particle).child(Field::LifetimeRandom),
            lifetime_random,
        ),
        kill_radius,
        gravity_x,
        gravity_y,
        gravity,
        size_mid_time,
        color_mid_time,
        alpha_mid_time,
        rotation_mid_time,
        size_mid_hold_time,
        color_mid_hold_time,
        alpha_mid_hold_time,
        rotation_mid_hold_time,
        size_animation: reader.property(
            &FieldPath::root(Field::Particle).child(Field::SizeAnimation),
            size_animation,
        ),
        rotation_animation: reader.property(
            &FieldPath::root(Field::Particle).child(Field::RotationAnimation),
            rotation_animation,
        ),
        color_start: reader.fixed_color(
            &FieldPath::root(Field::Particle).child(Field::ColorStart),
            color_start,
        ),
        color_mid: reader.fixed_color(
            &FieldPath::root(Field::Particle).child(Field::ColorMid),
            color_mid,
        ),
        color_end: reader.fixed_color(
            &FieldPath::root(Field::Particle).child(Field::ColorEnd),
            color_end,
        ),
        drag,
        mass,
        mass_random,
        mass_size_multiplier,
        local_forces: surface::values::LocalForces(local_forces.bits()),
        world_forces: surface::values::WorldForces(world_forces.bits()),
        local_forces_fallback: surface::values::LocalForces(local_forces_fallback.bits()),
        world_forces_fallback: surface::values::WorldForces(world_forces_fallback.bits()),
        world_forces_mass_multiplier,
        noise_amplitude,
        noise_frequency,
        noise_coherence,
        noise_edge,
        index_plus_length,
        max_particles,
        emission_rate: reader.property(
            &FieldPath::root(Field::Particle).child(Field::EmissionRate),
            emission_rate,
        ),
        emitter_shape: surface::values::EmitterShape::from_raw((emitter_shape).raw()),
        shape_outer: reader.property(
            &FieldPath::root(Field::Particle).child(Field::ShapeOuter),
            shape_outer,
        ),
        shape_inner: reader.property(
            &FieldPath::root(Field::Particle).child(Field::ShapeInner),
            shape_inner,
        ),
        outer_radius: reader.property(
            &FieldPath::root(Field::Particle).child(Field::OuterRadius),
            outer_radius,
        ),
        inner_radius: reader.property(
            &FieldPath::root(Field::Particle).child(Field::InnerRadius),
            inner_radius,
        ),
        shape_sections: reader.sections(shape_regions)?,
        velocity_type: surface::values::VelocityKind::from_raw(velocity_type.raw()),
        size_random_enable,
        size_random_animation: reader.property(
            &FieldPath::root(Field::Particle).child(Field::SizeRandomAnimation),
            size_random_animation,
        ),
        rotation_random_enable,
        rotation_random_animation: reader.property(
            &FieldPath::root(Field::Particle).child(Field::RotationRandomAnimation),
            rotation_random_animation,
        ),
        color_random_enable,
        color_start_random: reader.color(
            &FieldPath::root(Field::Particle).child(Field::ColorStartRandom),
            color_start_random,
        ),
        color_mid_random: reader.color(
            &FieldPath::root(Field::Particle).child(Field::ColorMidRandom),
            color_mid_random,
        ),
        color_end_random: reader.color(
            &FieldPath::root(Field::Particle).child(Field::ColorEndRandom),
            color_end_random,
        ),
        alpha_random_enable,
        squirt_amount: reader.property(
            &FieldPath::root(Field::Particle).child(Field::SquirtAmount),
            squirt_amount,
        ),
        flipbook_start_init_index,
        flipbook_start_stop_index,
        flipbook_end_init_index,
        flipbook_end_stop_index,
        flipbook_mid_time,
        flipbook_columns,
        flipbook_rows,
        flipbook_column_fraction: fraction(flipbook_column_fraction)?,
        flipbook_row_fraction: fraction(flipbook_row_fraction)?,
        bounce,
        friction,
        collision_spawn: reader.particle(collision_spawn_index)?,
        collision_spawn_min,
        collision_spawn_max,
        collision_spawn_chance,
        collision_spawn_energy,
        collision_die_bounce,
        instance_type: surface::values::ParticleInstance::from_raw((instance_type).raw()),
        tail_length,
        instance_angle,
        instance_distance,
        pitch: surface::VariationControl {
            kind: surface::values::Variation::from_raw((pitch_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Particle).child(Field::PitchAmplitude),
                pitch_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Particle).child(Field::PitchFrequency),
                pitch_frequency,
            ),
        },
        yaw: surface::VariationControl {
            kind: surface::values::Variation::from_raw((yaw_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Particle).child(Field::YawAmplitude),
                yaw_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Particle).child(Field::YawFrequency),
                yaw_frequency,
            ),
        },
        speed: surface::VariationControl {
            kind: surface::values::Variation::from_raw((speed_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Particle).child(Field::SpeedAmplitude),
                speed_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Particle).child(Field::SpeedFrequency),
                speed_frequency,
            ),
        },
        size: surface::VariationControl {
            kind: surface::values::Variation::from_raw((size_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Particle).child(Field::SizeAmplitude),
                size_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Particle).child(Field::SizeFrequency),
                size_frequency,
            ),
        },
        alpha: surface::VariationControl {
            kind: surface::values::Variation::from_raw((alpha_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Particle).child(Field::AlphaAmplitude),
                alpha_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Particle).child(Field::AlphaFrequency),
                alpha_frequency,
            ),
        },
        color: surface::VariationControl {
            kind: surface::values::Variation::from_raw((color_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Particle).child(Field::ColorAmplitude),
                color_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Particle).child(Field::ColorFrequency),
                color_frequency,
            ),
        },
        rotation: surface::VariationControl {
            kind: surface::values::Variation::from_raw((rotation_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Particle).child(Field::RotationAmplitude),
                rotation_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Particle).child(Field::RotationFrequency),
                rotation_frequency,
            ),
        },
        horizontal: surface::VariationControl {
            kind: surface::values::Variation::from_raw((horizontal_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Particle).child(Field::HorizontalAmplitude),
                horizontal_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Particle).child(Field::HorizontalFrequency),
                horizontal_frequency,
            ),
        },
        vertical: surface::VariationControl {
            kind: surface::values::Variation::from_raw((vertical_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Particle).child(Field::VerticalAmplitude),
                vertical_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Particle).child(Field::VerticalFrequency),
                vertical_frequency,
            ),
        },
        particle_velocity: reader.property(
            &FieldPath::root(Field::Particle).child(Field::ParticleVelocity),
            particle_velocity,
        ),
        phase_shift: reader.property(
            &FieldPath::root(Field::Particle).child(Field::PhaseShift),
            phase_shift,
        ),
        flags: surface::values::ParticleFlags(flags.bits()),
        rotation_flags: surface::values::ParticleRotationFlags(rotation_flags),
        color_smoothing: surface::values::Smoothing::from_raw(color_smoothing),
        size_smoothing: surface::values::Smoothing::from_raw(size_smoothing),
        rotation_smoothing: surface::values::Smoothing::from_raw(rotation_smoothing),
        alpha_threshold: reader.property(
            &FieldPath::root(Field::Particle).child(Field::AlphaThreshold),
            alpha_threshold,
        ),
        uv_offset: reader.property(
            &FieldPath::root(Field::Particle).child(Field::UvOffset),
            uv_offset,
        ),
        uv_angle: reader.property(
            &FieldPath::root(Field::Particle).child(Field::UvAngle),
            uv_angle,
        ),
        uv_tiling: reader.property(
            &FieldPath::root(Field::Particle).child(Field::UvTiling),
            uv_tiling,
        ),
        spline_points: reader.spline(spline_line_data)?,
        wind_multiplier,
        lod_reduce,
        lod_cut,
        lower_bound: reader.property(
            &FieldPath::root(Field::Particle).child(Field::LowerBound),
            lower_bound,
        ),
        upper_bound: reader.property(
            &FieldPath::root(Field::Particle).child(Field::UpperBound),
            upper_bound,
        ),
        trail: reader.particle(trail_link_index)?,
        trail_chance,
        trail_emission_rate: reader.property(
            &FieldPath::root(Field::Particle).child(Field::TrailEmissionRate),
            trail_emission_rate,
        ),
        projection: reader.projector(splat_projection_index)?,
        splat_chance,
        models: reader.models(model_paths)?,
        spawn_ribbon_on_bounce_chance,
        ribbon: reader.ribbon(ribbon_link_index)?,
        noise_smoothness: deprecated_noise_smoothness,
    })
}

fn fraction(value: f32) -> Result<Option<f32>, Error> {
    if value == f32::INFINITY {
        Ok(None)
    } else if value.is_finite() {
        Ok(Some(value))
    } else {
        Err(Error::new(
            "m3.particle.flipbook_fraction",
            "atlas fraction must be finite or the native undefined sentinel",
        ))
    }
}
