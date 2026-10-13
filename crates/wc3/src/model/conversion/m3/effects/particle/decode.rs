//! Field mapping for the particle decode path.
use super::super::emitter_fields::{color, fixed_color, property, squirt, Slot};
use super::{models, shape_sections, spline_points};
use super::{native, nr, wem, wr, Envelope, Error};
use std::collections::BTreeMap;
pub(super) fn decode<E: Envelope, D: nr::ParticleEmitterDialect>(
    r: nr::ParticleEmitter<D>,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    values: &mut BTreeMap<String, i64>,
) -> Result<wr::Sc2ParticleEmitterPayload, Error>
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
    let legacy_size_mid_time = r.legacy_size_mid_time.slot().unwrap_or_default();
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
    let legacy_rotation_mid_time = r.legacy_rotation_mid_time.slot().unwrap_or_default();
    let rotation_mid_time = r
        .rotation_mid_time
        .slot()
        .or_else(|| r.legacy_rotation_mid_time.slot())
        .unwrap_or_default();
    let size_mid_hold_time = r.size_mid_hold_time.slot().unwrap_or_default();
    let color_mid_hold_time = r.color_mid_hold_time.slot().unwrap_or_default();
    let alpha_mid_hold_time = r.alpha_mid_hold_time.slot().unwrap_or_default();
    let legacy_color_mid_time = r.legacy_color_mid_time.slot().unwrap_or_default();
    let rotation_mid_hold_time = r.rotation_mid_hold_time.slot().unwrap_or_default();
    let legacy_alpha_mid_time = r.legacy_alpha_mid_time.slot().unwrap_or_default();
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
    let flipbook_column_fraction = r.flipbook_column_fraction.slot().unwrap_or_default();
    let flipbook_row_fraction = r.flipbook_row_fraction.slot().unwrap_or_default();
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
    values.insert(
        "m3.particle.legacy_speed_randomize".into(),
        i64::from(legacy_speed_randomize),
    );
    values.insert(
        "m3.particle.legacy_lifespan_randomize".into(),
        i64::from(legacy_lifespan_randomize),
    );
    values.insert(
        "m3.particle.legacy_size_mid_time".into(),
        i64::from(legacy_size_mid_time.to_bits()),
    );
    values.insert(
        "m3.particle.legacy_rotation_mid_time".into(),
        i64::from(legacy_rotation_mid_time.to_bits()),
    );
    values.insert(
        "m3.particle.legacy_color_mid_time".into(),
        i64::from(legacy_color_mid_time.to_bits()),
    );
    values.insert(
        "m3.particle.legacy_alpha_mid_time".into(),
        i64::from(legacy_alpha_mid_time.to_bits()),
    );
    values.insert(
        "m3.particle.legacy_mass_randomize".into(),
        i64::from(legacy_mass_randomize),
    );
    values.insert(
        "m3.particle.legacy_world_space".into(),
        i64::from(legacy_world_space),
    );
    values.insert(
        "m3.particle.deprecated_noise_smoothness".into(),
        i64::from(deprecated_noise_smoothness.to_bits()),
    );
    values.insert("m3.particle.source_version".into(), i64::from(D::VERSION));
    Ok(wr::Sc2ParticleEmitterPayload {
        copy_of: u32::MAX,
        material_slot: material_index,
        additional_flags,
        initial_speed: property(values, "particle.initial_speed", initial_speed),
        initial_speed_random: property(
            values,
            "particle.initial_speed_random",
            initial_speed_random,
        ),
        initial_yaw: property(values, "particle.initial_yaw", initial_yaw),
        initial_pitch: property(values, "particle.initial_pitch", initial_pitch),
        initial_horizontal: property(values, "particle.initial_horizontal", initial_horizontal),
        initial_vertical: property(values, "particle.initial_vertical", initial_vertical),
        lifetime: property(values, "particle.lifetime", lifetime),
        lifetime_random: property(values, "particle.lifetime_random", lifetime_random),
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
        size_animation: property(values, "particle.size_animation", size_animation),
        rotation_animation: property(values, "particle.rotation_animation", rotation_animation),
        color_start: fixed_color(values, "particle.color_start", color_start),
        color_mid: fixed_color(values, "particle.color_mid", color_mid),
        color_end: fixed_color(values, "particle.color_end", color_end),
        drag,
        mass,
        mass_random,
        mass_size_multiplier,
        local_forces,
        world_forces,
        local_forces_fallback,
        world_forces_fallback,
        world_forces_mass_multiplier,
        noise_amplitude,
        noise_frequency,
        noise_coherence,
        noise_edge,
        index_plus_length,
        max_particles,
        emission_rate: property(values, "particle.emission_rate", emission_rate),
        emitter_shape: wr::Sc2EmitterShape(emitter_shape),
        shape_outer: property(values, "particle.shape_outer", shape_outer),
        shape_inner: property(values, "particle.shape_inner", shape_inner),
        outer_radius: property(values, "particle.outer_radius", outer_radius),
        inner_radius: property(values, "particle.inner_radius", inner_radius),
        shape_sections: shape_sections(source, doc, shape_regions)?,
        velocity_type,
        size_random_enable,
        size_random_animation: property(
            values,
            "particle.size_random_animation",
            size_random_animation,
        ),
        rotation_random_enable,
        rotation_random_animation: property(
            values,
            "particle.rotation_random_animation",
            rotation_random_animation,
        ),
        color_random_enable,
        color_start_random: color(values, "particle.color_start_random", color_start_random),
        color_mid_random: color(values, "particle.color_mid_random", color_mid_random),
        color_end_random: color(values, "particle.color_end_random", color_end_random),
        alpha_random_enable,
        squirt_amount: squirt(values, "particle.squirt_amount", squirt_amount),
        flipbook_start_init_index,
        flipbook_start_stop_index,
        flipbook_end_init_index,
        flipbook_end_stop_index,
        flipbook_mid_time,
        flipbook_columns,
        flipbook_rows,
        flipbook_column_fraction,
        flipbook_row_fraction,
        bounce,
        friction,
        collision_spawn: collision_spawn_index as u32,
        collision_spawn_min,
        collision_spawn_max,
        collision_spawn_chance,
        collision_spawn_energy,
        collision_die_bounce,
        instance_type: wr::Sc2ParticleInstance(instance_type),
        tail_length,
        instance_angle,
        instance_distance,
        pitch: wr::Sc2Variation {
            type_: pitch_type,
            amplitude: property(values, "particle.pitch_amplitude", pitch_amplitude),
            frequency: property(values, "particle.pitch_frequency", pitch_frequency),
        },
        yaw: wr::Sc2Variation {
            type_: yaw_type,
            amplitude: property(values, "particle.yaw_amplitude", yaw_amplitude),
            frequency: property(values, "particle.yaw_frequency", yaw_frequency),
        },
        speed: wr::Sc2Variation {
            type_: speed_type,
            amplitude: property(values, "particle.speed_amplitude", speed_amplitude),
            frequency: property(values, "particle.speed_frequency", speed_frequency),
        },
        size: wr::Sc2Variation {
            type_: size_type,
            amplitude: property(values, "particle.size_amplitude", size_amplitude),
            frequency: property(values, "particle.size_frequency", size_frequency),
        },
        alpha: wr::Sc2Variation {
            type_: alpha_type,
            amplitude: property(values, "particle.alpha_amplitude", alpha_amplitude),
            frequency: property(values, "particle.alpha_frequency", alpha_frequency),
        },
        color: wr::Sc2Variation {
            type_: color_type,
            amplitude: property(values, "particle.color_amplitude", color_amplitude),
            frequency: property(values, "particle.color_frequency", color_frequency),
        },
        rotation: wr::Sc2Variation {
            type_: rotation_type,
            amplitude: property(values, "particle.rotation_amplitude", rotation_amplitude),
            frequency: property(values, "particle.rotation_frequency", rotation_frequency),
        },
        horizontal: wr::Sc2Variation {
            type_: horizontal_type,
            amplitude: property(
                values,
                "particle.horizontal_amplitude",
                horizontal_amplitude,
            ),
            frequency: property(
                values,
                "particle.horizontal_frequency",
                horizontal_frequency,
            ),
        },
        vertical: wr::Sc2Variation {
            type_: vertical_type,
            amplitude: property(values, "particle.vertical_amplitude", vertical_amplitude),
            frequency: property(values, "particle.vertical_frequency", vertical_frequency),
        },
        particle_velocity: property(values, "particle.particle_velocity", particle_velocity),
        phase_shift: property(values, "particle.phase_shift", phase_shift),
        flags,
        rotation_flags,
        color_smoothing: wr::Sc2Smoothing(color_smoothing),
        size_smoothing: wr::Sc2Smoothing(size_smoothing),
        rotation_smoothing: wr::Sc2Smoothing(rotation_smoothing),
        alpha_threshold: property(values, "particle.alpha_threshold", alpha_threshold),
        uv_offset: property(values, "particle.uv_offset", uv_offset),
        uv_angle: property(values, "particle.uv_angle", uv_angle),
        uv_tiling: property(values, "particle.uv_tiling", uv_tiling),
        spline_points: spline_points(source, doc, spline_line_data, values)?,
        wind_multiplier,
        lod_reduce,
        lod_cut,
        lower_bound: property(values, "particle.lower_bound", lower_bound),
        upper_bound: property(values, "particle.upper_bound", upper_bound),
        trail_link: trail_link_index as u32,
        trail_chance,
        trail_emission_rate: property(values, "particle.trail_emission_rate", trail_emission_rate),
        splat_projection_index,
        splat_chance,
        models: models(source, doc, model_paths)?,
        spawn_ribbon_on_bounce_chance,
        ribbon_link: ribbon_link_index as u32,
        noise_smoothness: deprecated_noise_smoothness,
    })
}
