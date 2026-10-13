//! Field mapping for the particle encode path.
use super::super::emitter_fields::{native_color, native_fixed_color, native_squirt, Assign};
use super::super::metadata::native_property;
use super::Context;
use super::Ref;
use super::{native, nr, wem, wr, Envelope, Error};
use super::{native_models, native_shape_sections, native_spline_points};
use std::collections::BTreeMap;
pub(super) fn encode<E: Envelope, D: nr::ParticleEmitterDialect>(
    r: wr::Sc2ParticleEmitterPayload,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    values: &BTreeMap<String, i64>,
    bone: u32,
    copies: Ref<nr::targets::U32>,
    context: &mut Context,
) -> Result<nr::ParticleEmitter<D>, Error>
where
    D::LegacySpeedRandomize: Assign<u32>,
    D::LegacyLifespanRandomize: Assign<u32>,
    D::LegacySizeAnimation: Assign<nr::AnimationValue<[f32; 3]>>,
    D::LegacySizeMidTime: Assign<f32>,
    D::LegacyRotationAnimation: Assign<nr::AnimationValue<[f32; 3]>>,
    D::LegacyRotationMidTime: Assign<f32>,
    D::LegacyColorStart: Assign<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::LegacyColorMid: Assign<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::LegacyColorEnd: Assign<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::LegacyColorMidTime: Assign<f32>,
    D::LegacyAlphaMidTime: Assign<f32>,
    D::LegacyMassRandomize: Assign<u32>,
    D::LegacyWorldSpace: Assign<u32>,
    D::DeprecatedNoiseSmoothness: Assign<f32>,
    D::IndexPlusLength: Assign<u32>,
    D::SizeMidTime: Assign<f32>,
    D::ColorMidTime: Assign<f32>,
    D::AlphaMidTime: Assign<f32>,
    D::RotationMidTime: Assign<f32>,
    D::SizeAnimation: Assign<nr::AnimationValue<[f32; 3]>>,
    D::RotationAnimation: Assign<nr::AnimationValue<[f32; 3]>>,
    D::ColorStart: Assign<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::ColorMid: Assign<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::ColorEnd: Assign<nr::AnimationValue<nr::FixedBytes<4>>>,
    D::MassSizeMultiplier: Assign<f32>,
    D::FlipbookColumnFraction: Assign<f32>,
    D::FlipbookRowFraction: Assign<f32>,
    D::SizeMidHoldTime: Assign<f32>,
    D::ColorMidHoldTime: Assign<f32>,
    D::AlphaMidHoldTime: Assign<f32>,
    D::RotationMidHoldTime: Assign<f32>,
    D::ColorSmoothing: Assign<u32>,
    D::SizeSmoothing: Assign<u32>,
    D::RotationSmoothing: Assign<u32>,
    D::ShapeRegions: Assign<native::Ref<nr::targets::U32>>,
    D::AlphaThreshold: Assign<nr::AnimationValue<f32>>,
    D::UvOffset: Assign<nr::AnimationValue<[f32; 2]>>,
    D::UvAngle: Assign<nr::AnimationValue<[f32; 3]>>,
    D::UvTiling: Assign<nr::AnimationValue<[f32; 2]>>,
    D::InstanceDistance: Assign<f32>,
    D::AdditionalFlags: Assign<u32>,
    D::RotationFlags: Assign<u32>,
    D::PhaseShift: Assign<nr::AnimationValue<f32>>,
    D::SpawnRibbonOnBounceChance: Assign<f32>,
    D::RibbonLinkIndex: Assign<i32>,
    D::WorldForcesMassMultiplier: Assign<f32>,
{
    if !D::AdditionalFlags::PRESENT && r.additional_flags != Default::default() {
        context.unsupported(
            "particle.additional_flags",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::WorldForcesMassMultiplier::PRESENT
        && r.world_forces_mass_multiplier != Default::default()
    {
        context.unsupported(
            "particle.world_forces_mass_multiplier",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::IndexPlusLength::PRESENT && r.index_plus_length != Default::default() {
        context.unsupported(
            "particle.index_plus_length",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SizeMidHoldTime::PRESENT && r.size_mid_hold_time != Default::default() {
        context.unsupported(
            "particle.size_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::ColorMidHoldTime::PRESENT && r.color_mid_hold_time != Default::default() {
        context.unsupported(
            "particle.color_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::AlphaMidHoldTime::PRESENT && r.alpha_mid_hold_time != Default::default() {
        context.unsupported(
            "particle.alpha_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::RotationMidHoldTime::PRESENT && r.rotation_mid_hold_time != Default::default() {
        context.unsupported(
            "particle.rotation_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SizeSmoothing::PRESENT && r.size_smoothing != Default::default() {
        context.unsupported(
            "particle.size_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::ColorSmoothing::PRESENT && r.color_smoothing != Default::default() {
        context.unsupported(
            "particle.color_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::MassSizeMultiplier::PRESENT && r.mass_size_multiplier != Default::default() {
        context.unsupported(
            "particle.mass_size_multiplier",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::FlipbookColumnFraction::PRESENT && r.flipbook_column_fraction != Default::default() {
        context.unsupported(
            "particle.flipbook_column_fraction",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::FlipbookRowFraction::PRESENT && r.flipbook_row_fraction != Default::default() {
        context.unsupported(
            "particle.flipbook_row_fraction",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::RotationSmoothing::PRESENT && r.rotation_smoothing != Default::default() {
        context.unsupported(
            "particle.rotation_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::AlphaThreshold::PRESENT && r.alpha_threshold != Default::default() {
        context.unsupported(
            "particle.alpha_threshold.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::UvOffset::PRESENT && r.uv_offset != Default::default() {
        context.unsupported(
            "particle.uv_offset.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::UvAngle::PRESENT && r.uv_angle != Default::default() {
        context.unsupported(
            "particle.uv_angle.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::UvTiling::PRESENT && r.uv_tiling != Default::default() {
        context.unsupported(
            "particle.uv_tiling.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::InstanceDistance::PRESENT && r.instance_distance != Default::default() {
        context.unsupported(
            "particle.instance_distance",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::RotationFlags::PRESENT && r.rotation_flags != Default::default() {
        context.unsupported(
            "particle.rotation_flags",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::PhaseShift::PRESENT && r.phase_shift != Default::default() {
        context.unsupported(
            "particle.phase_shift.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SpawnRibbonOnBounceChance::PRESENT
        && r.spawn_ribbon_on_bounce_chance != Default::default()
    {
        context.unsupported(
            "particle.spawn_ribbon_on_bounce_chance",
            "selected source revision omits this emitter control",
        )?;
    }
    Ok(nr::ParticleEmitter {
        bone_index: bone,
        material_index: r.material_slot,
        additional_flags: D::AdditionalFlags::assign(r.additional_flags),
        initial_speed: native_property(values, "particle.initial_speed", r.initial_speed.clone()),
        initial_speed_random: native_property(
            values,
            "particle.initial_speed_random",
            r.initial_speed_random.clone(),
        ),
        legacy_speed_randomize: D::LegacySpeedRandomize::assign(
            values
                .get("m3.particle.legacy_speed_randomize")
                .copied()
                .unwrap_or(0) as u32,
        ),
        initial_yaw: native_property(values, "particle.initial_yaw", r.initial_yaw.clone()),
        initial_pitch: native_property(values, "particle.initial_pitch", r.initial_pitch.clone()),
        initial_horizontal: native_property(
            values,
            "particle.initial_horizontal",
            r.initial_horizontal.clone(),
        ),
        initial_vertical: native_property(
            values,
            "particle.initial_vertical",
            r.initial_vertical.clone(),
        ),
        lifetime: native_property(values, "particle.lifetime", r.lifetime.clone()),
        lifetime_random: native_property(
            values,
            "particle.lifetime_random",
            r.lifetime_random.clone(),
        ),
        legacy_lifespan_randomize: D::LegacyLifespanRandomize::assign(
            values
                .get("m3.particle.legacy_lifespan_randomize")
                .copied()
                .unwrap_or(0) as u32,
        ),
        kill_radius: r.kill_radius,
        gravity_x: r.gravity_x,
        gravity_y: r.gravity_y,
        gravity: r.gravity,
        legacy_size_animation: D::LegacySizeAnimation::assign(native_property(
            values,
            "particle.size_animation",
            r.size_animation.clone(),
        )),
        size_mid_time: D::SizeMidTime::assign(r.size_mid_time),
        legacy_size_mid_time: D::LegacySizeMidTime::assign(r.size_mid_time),
        color_mid_time: D::ColorMidTime::assign(r.color_mid_time),
        legacy_rotation_animation: D::LegacyRotationAnimation::assign(native_property(
            values,
            "particle.rotation_animation",
            r.rotation_animation.clone(),
        )),
        alpha_mid_time: D::AlphaMidTime::assign(r.alpha_mid_time),
        legacy_rotation_mid_time: D::LegacyRotationMidTime::assign(r.rotation_mid_time),
        rotation_mid_time: D::RotationMidTime::assign(r.rotation_mid_time),
        legacy_color_start: D::LegacyColorStart::assign(native_fixed_color(
            values,
            "particle.color_start",
            r.color_start.clone(),
            context,
        )?),
        size_mid_hold_time: D::SizeMidHoldTime::assign(r.size_mid_hold_time),
        legacy_color_mid: D::LegacyColorMid::assign(native_fixed_color(
            values,
            "particle.color_mid",
            r.color_mid.clone(),
            context,
        )?),
        color_mid_hold_time: D::ColorMidHoldTime::assign(r.color_mid_hold_time),
        legacy_color_end: D::LegacyColorEnd::assign(native_fixed_color(
            values,
            "particle.color_end",
            r.color_end.clone(),
            context,
        )?),
        alpha_mid_hold_time: D::AlphaMidHoldTime::assign(r.alpha_mid_hold_time),
        legacy_color_mid_time: D::LegacyColorMidTime::assign(r.color_mid_time),
        rotation_mid_hold_time: D::RotationMidHoldTime::assign(r.rotation_mid_hold_time),
        legacy_alpha_mid_time: D::LegacyAlphaMidTime::assign(r.alpha_mid_time),
        size_animation: D::SizeAnimation::assign(native_property(
            values,
            "particle.size_animation",
            r.size_animation.clone(),
        )),
        rotation_animation: D::RotationAnimation::assign(native_property(
            values,
            "particle.rotation_animation",
            r.rotation_animation.clone(),
        )),
        color_start: D::ColorStart::assign(native_fixed_color(
            values,
            "particle.color_start",
            r.color_start.clone(),
            context,
        )?),
        color_mid: D::ColorMid::assign(native_fixed_color(
            values,
            "particle.color_mid",
            r.color_mid.clone(),
            context,
        )?),
        color_end: D::ColorEnd::assign(native_fixed_color(
            values,
            "particle.color_end",
            r.color_end.clone(),
            context,
        )?),
        drag: r.drag,
        legacy_mass_randomize: D::LegacyMassRandomize::assign(
            values
                .get("m3.particle.legacy_mass_randomize")
                .copied()
                .unwrap_or(0) as u32,
        ),
        mass: r.mass,
        mass_random: r.mass_random,
        mass_size_multiplier: D::MassSizeMultiplier::assign(r.mass_size_multiplier),
        legacy_world_space: D::LegacyWorldSpace::assign(
            values
                .get("m3.particle.legacy_world_space")
                .copied()
                .unwrap_or(0) as u32,
        ),
        local_forces: r.local_forces,
        world_forces: r.world_forces,
        local_forces_fallback: r.local_forces_fallback,
        world_forces_fallback: r.world_forces_fallback,
        world_forces_mass_multiplier: D::WorldForcesMassMultiplier::assign(
            r.world_forces_mass_multiplier,
        ),
        noise_amplitude: r.noise_amplitude,
        noise_frequency: r.noise_frequency,
        noise_coherence: r.noise_coherence,
        noise_edge: r.noise_edge,
        deprecated_noise_smoothness: D::DeprecatedNoiseSmoothness::assign(r.noise_smoothness),
        index_plus_length: D::IndexPlusLength::assign(r.index_plus_length),
        max_particles: r.max_particles,
        emission_rate: native_property(values, "particle.emission_rate", r.emission_rate.clone()),
        emitter_shape: r.emitter_shape.0,
        shape_outer: native_property(values, "particle.shape_outer", r.shape_outer.clone()),
        shape_inner: native_property(values, "particle.shape_inner", r.shape_inner.clone()),
        outer_radius: native_property(values, "particle.outer_radius", r.outer_radius.clone()),
        inner_radius: native_property(values, "particle.inner_radius", r.inner_radius.clone()),
        shape_regions: D::ShapeRegions::assign(native_shape_sections(
            source,
            doc,
            r.shape_sections,
        )?),
        velocity_type: r.velocity_type,
        size_random_enable: r.size_random_enable,
        size_random_animation: native_property(
            values,
            "particle.size_random_animation",
            r.size_random_animation.clone(),
        ),
        rotation_random_enable: r.rotation_random_enable,
        rotation_random_animation: native_property(
            values,
            "particle.rotation_random_animation",
            r.rotation_random_animation.clone(),
        ),
        color_random_enable: r.color_random_enable,
        color_start_random: native_color(
            values,
            "particle.color_start_random",
            r.color_start_random.clone(),
            context,
        )?,
        color_mid_random: native_color(
            values,
            "particle.color_mid_random",
            r.color_mid_random.clone(),
            context,
        )?,
        color_end_random: native_color(
            values,
            "particle.color_end_random",
            r.color_end_random.clone(),
            context,
        )?,
        alpha_random_enable: r.alpha_random_enable,
        squirt_amount: native_squirt(values, "particle.squirt_amount", r.squirt_amount, context)?,
        flipbook_start_init_index: r.flipbook_start_init_index,
        flipbook_start_stop_index: r.flipbook_start_stop_index,
        flipbook_end_init_index: r.flipbook_end_init_index,
        flipbook_end_stop_index: r.flipbook_end_stop_index,
        flipbook_mid_time: r.flipbook_mid_time,
        flipbook_columns: r.flipbook_columns,
        flipbook_rows: r.flipbook_rows,
        flipbook_column_fraction: D::FlipbookColumnFraction::assign(r.flipbook_column_fraction),
        flipbook_row_fraction: D::FlipbookRowFraction::assign(r.flipbook_row_fraction),
        bounce: r.bounce,
        friction: r.friction,
        collision_spawn_index: r.collision_spawn as i32,
        collision_spawn_min: r.collision_spawn_min,
        collision_spawn_max: r.collision_spawn_max,
        collision_spawn_chance: r.collision_spawn_chance,
        collision_spawn_energy: r.collision_spawn_energy,
        collision_die_bounce: r.collision_die_bounce,
        instance_type: r.instance_type.0,
        tail_length: r.tail_length,
        instance_angle: r.instance_angle,
        instance_distance: D::InstanceDistance::assign(r.instance_distance),
        pitch_type: r.pitch.type_,
        pitch_amplitude: native_property(
            values,
            "particle.pitch_amplitude",
            r.pitch.amplitude.clone(),
        ),
        pitch_frequency: native_property(
            values,
            "particle.pitch_frequency",
            r.pitch.frequency.clone(),
        ),
        yaw_type: r.yaw.type_,
        yaw_amplitude: native_property(values, "particle.yaw_amplitude", r.yaw.amplitude.clone()),
        yaw_frequency: native_property(values, "particle.yaw_frequency", r.yaw.frequency.clone()),
        speed_type: r.speed.type_,
        speed_amplitude: native_property(
            values,
            "particle.speed_amplitude",
            r.speed.amplitude.clone(),
        ),
        speed_frequency: native_property(
            values,
            "particle.speed_frequency",
            r.speed.frequency.clone(),
        ),
        size_type: r.size.type_,
        size_amplitude: native_property(
            values,
            "particle.size_amplitude",
            r.size.amplitude.clone(),
        ),
        size_frequency: native_property(
            values,
            "particle.size_frequency",
            r.size.frequency.clone(),
        ),
        alpha_type: r.alpha.type_,
        alpha_amplitude: native_property(
            values,
            "particle.alpha_amplitude",
            r.alpha.amplitude.clone(),
        ),
        alpha_frequency: native_property(
            values,
            "particle.alpha_frequency",
            r.alpha.frequency.clone(),
        ),
        color_type: r.color.type_,
        color_amplitude: native_property(
            values,
            "particle.color_amplitude",
            r.color.amplitude.clone(),
        ),
        color_frequency: native_property(
            values,
            "particle.color_frequency",
            r.color.frequency.clone(),
        ),
        rotation_type: r.rotation.type_,
        rotation_amplitude: native_property(
            values,
            "particle.rotation_amplitude",
            r.rotation.amplitude.clone(),
        ),
        rotation_frequency: native_property(
            values,
            "particle.rotation_frequency",
            r.rotation.frequency.clone(),
        ),
        horizontal_type: r.horizontal.type_,
        horizontal_amplitude: native_property(
            values,
            "particle.horizontal_amplitude",
            r.horizontal.amplitude.clone(),
        ),
        horizontal_frequency: native_property(
            values,
            "particle.horizontal_frequency",
            r.horizontal.frequency.clone(),
        ),
        vertical_type: r.vertical.type_,
        vertical_amplitude: native_property(
            values,
            "particle.vertical_amplitude",
            r.vertical.amplitude.clone(),
        ),
        vertical_frequency: native_property(
            values,
            "particle.vertical_frequency",
            r.vertical.frequency.clone(),
        ),
        particle_velocity: native_property(
            values,
            "particle.particle_velocity",
            r.particle_velocity.clone(),
        ),
        phase_shift: D::PhaseShift::assign(native_property(
            values,
            "particle.phase_shift",
            r.phase_shift.clone(),
        )),
        flags: r.flags,
        rotation_flags: D::RotationFlags::assign(r.rotation_flags),
        color_smoothing: D::ColorSmoothing::assign(r.color_smoothing.0),
        size_smoothing: D::SizeSmoothing::assign(r.size_smoothing.0),
        rotation_smoothing: D::RotationSmoothing::assign(r.rotation_smoothing.0),
        alpha_threshold: D::AlphaThreshold::assign(native_property(
            values,
            "particle.alpha_threshold",
            r.alpha_threshold.clone(),
        )),
        uv_offset: D::UvOffset::assign(native_property(
            values,
            "particle.uv_offset",
            r.uv_offset.clone(),
        )),
        uv_angle: D::UvAngle::assign(native_property(
            values,
            "particle.uv_angle",
            r.uv_angle.clone(),
        )),
        uv_tiling: D::UvTiling::assign(native_property(
            values,
            "particle.uv_tiling",
            r.uv_tiling.clone(),
        )),
        spline_line_data: native_spline_points(source, doc, r.spline_points, values)?,
        wind_multiplier: r.wind_multiplier,
        lod_reduce: r.lod_reduce,
        lod_cut: r.lod_cut,
        lower_bound: native_property(values, "particle.lower_bound", r.lower_bound.clone()),
        upper_bound: native_property(values, "particle.upper_bound", r.upper_bound.clone()),
        trail_link_index: r.trail_link as i32,
        trail_chance: r.trail_chance,
        trail_emission_rate: native_property(
            values,
            "particle.trail_emission_rate",
            r.trail_emission_rate.clone(),
        ),
        splat_projection_index: r.splat_projection_index,
        splat_chance: r.splat_chance,
        model_paths: native_models(source, doc, r.models)?,
        copy_indices: copies,
        spawn_ribbon_on_bounce_chance: D::SpawnRibbonOnBounceChance::assign(
            r.spawn_ribbon_on_bounce_chance,
        ),
        ribbon_link_index: D::RibbonLinkIndex::assign(r.ribbon_link as i32),
    })
}
