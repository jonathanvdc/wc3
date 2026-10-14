//! Rebuild each emitter field from independently editable logical controls.
use super::super::{
    slots::Assign,
    write::{animated, color, fixed_color},
};
use super::writer::Writer;
use crate::model::conversion::Error;
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::profiles::m3 as surface;
use native::{Envelope, Ref};
pub(super) fn encode<E: Envelope, D: nr::ParticleEmitterDialect>(
    r: &surface::Particle<'_>,
    writer: &mut Writer<'_, '_, E>,
    bone: u32,
    copies: Ref<nr::targets::U32>,
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
    if !D::AdditionalFlags::PRESENT && r.additional_flags.bits() & !0xF != 0 {
        writer.context.unsupported(
            "particle.additional_flags",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::DeprecatedNoiseSmoothness::PRESENT && r.noise_smoothness != 0.0 {
        writer.context.unsupported(
            "m3.particle.noise_smoothness",
            "selected native particle layout has no legacy noise smoothing control",
        )?;
    }
    if !D::WorldForcesMassMultiplier::PRESENT && r.world_forces_mass_multiplier != f32::default() {
        writer.context.unsupported(
            "particle.world_forces_mass_multiplier",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::IndexPlusLength::PRESENT && r.index_plus_length != u32::default() {
        writer.context.unsupported(
            "particle.index_plus_length",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SizeMidHoldTime::PRESENT && r.size_mid_hold_time != f32::default() {
        writer.context.unsupported(
            "particle.size_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::ColorMidHoldTime::PRESENT && r.color_mid_hold_time != f32::default() {
        writer.context.unsupported(
            "particle.color_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::AlphaMidHoldTime::PRESENT && r.alpha_mid_hold_time != f32::default() {
        writer.context.unsupported(
            "particle.alpha_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::RotationMidHoldTime::PRESENT && r.rotation_mid_hold_time != f32::default() {
        writer.context.unsupported(
            "particle.rotation_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SizeSmoothing::PRESENT && r.size_smoothing != Default::default() {
        writer.context.unsupported(
            "particle.size_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::ColorSmoothing::PRESENT && r.color_smoothing != Default::default() {
        writer.context.unsupported(
            "particle.color_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::MassSizeMultiplier::PRESENT && r.mass_size_multiplier != f32::default() {
        writer.context.unsupported(
            "particle.mass_size_multiplier",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::FlipbookColumnFraction::PRESENT && r.flipbook_column_fraction != Default::default() {
        writer.context.unsupported(
            "particle.flipbook_column_fraction",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::FlipbookRowFraction::PRESENT && r.flipbook_row_fraction != Default::default() {
        writer.context.unsupported(
            "particle.flipbook_row_fraction",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::RotationSmoothing::PRESENT && r.rotation_smoothing != Default::default() {
        writer.context.unsupported(
            "particle.rotation_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::AlphaThreshold::PRESENT && writer.property_is_set(&r.alpha_threshold) {
        writer.context.unsupported(
            "particle.alpha_threshold.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::UvOffset::PRESENT && writer.property_is_set(&r.uv_offset) {
        writer.context.unsupported(
            "particle.uv_offset.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::UvAngle::PRESENT && writer.property_is_set(&r.uv_angle) {
        writer.context.unsupported(
            "particle.uv_angle.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::UvTiling::PRESENT && writer.property_is_set(&r.uv_tiling) {
        writer.context.unsupported(
            "particle.uv_tiling.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::InstanceDistance::PRESENT && r.instance_distance != f32::default() {
        writer.context.unsupported(
            "particle.instance_distance",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::RotationFlags::PRESENT && r.rotation_flags != Default::default() {
        writer.context.unsupported(
            "particle.rotation_flags",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::PhaseShift::PRESENT && writer.property_is_set(&r.phase_shift) {
        writer.context.unsupported(
            "particle.phase_shift.",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SpawnRibbonOnBounceChance::PRESENT && r.spawn_ribbon_on_bounce_chance != f32::default() {
        writer.context.unsupported(
            "particle.spawn_ribbon_on_bounce_chance",
            "selected source revision omits this emitter control",
        )?;
    }
    Ok(nr::ParticleEmitter {
        bone_index: bone,
        material_index: writer.material(r.material)?,
        additional_flags: D::AdditionalFlags::assign((r.additional_flags).bits()),
        initial_speed: animated(&r.initial_speed, writer.ids),
        initial_speed_random: animated(&r.initial_speed_random, writer.ids),
        legacy_speed_randomize: D::LegacySpeedRandomize::assign(r.additional_flags.bits() & 1),
        initial_yaw: animated(&r.initial_yaw, writer.ids),
        initial_pitch: animated(&r.initial_pitch, writer.ids),
        initial_horizontal: animated(&r.initial_horizontal, writer.ids),
        initial_vertical: animated(&r.initial_vertical, writer.ids),
        lifetime: animated(&r.lifetime, writer.ids),
        lifetime_random: animated(&r.lifetime_random, writer.ids),
        legacy_lifespan_randomize: D::LegacyLifespanRandomize::assign(
            (r.additional_flags.bits() >> 1) & 1,
        ),
        kill_radius: r.kill_radius,
        gravity_x: r.gravity_x,
        gravity_y: r.gravity_y,
        gravity: r.gravity,
        legacy_size_animation: D::LegacySizeAnimation::assign(animated(
            &r.size_animation,
            writer.ids,
        )),
        size_mid_time: D::SizeMidTime::assign(r.size_mid_time),
        legacy_size_mid_time: D::LegacySizeMidTime::assign(r.size_mid_time),
        color_mid_time: D::ColorMidTime::assign(r.color_mid_time),
        legacy_rotation_animation: D::LegacyRotationAnimation::assign(animated(
            &r.rotation_animation,
            writer.ids,
        )),
        alpha_mid_time: D::AlphaMidTime::assign(r.alpha_mid_time),
        legacy_rotation_mid_time: D::LegacyRotationMidTime::assign(r.rotation_mid_time),
        rotation_mid_time: D::RotationMidTime::assign(r.rotation_mid_time),
        legacy_color_start: D::LegacyColorStart::assign(fixed_color(
            &r.color_start,
            writer.ids,
            writer.context,
        )?),
        size_mid_hold_time: D::SizeMidHoldTime::assign(r.size_mid_hold_time),
        legacy_color_mid: D::LegacyColorMid::assign(fixed_color(
            &r.color_mid,
            writer.ids,
            writer.context,
        )?),
        color_mid_hold_time: D::ColorMidHoldTime::assign(r.color_mid_hold_time),
        legacy_color_end: D::LegacyColorEnd::assign(fixed_color(
            &r.color_end,
            writer.ids,
            writer.context,
        )?),
        alpha_mid_hold_time: D::AlphaMidHoldTime::assign(r.alpha_mid_hold_time),
        legacy_color_mid_time: D::LegacyColorMidTime::assign(r.color_mid_time),
        rotation_mid_hold_time: D::RotationMidHoldTime::assign(r.rotation_mid_hold_time),
        legacy_alpha_mid_time: D::LegacyAlphaMidTime::assign(r.alpha_mid_time),
        size_animation: D::SizeAnimation::assign(animated(&r.size_animation, writer.ids)),
        rotation_animation: D::RotationAnimation::assign(animated(
            &r.rotation_animation,
            writer.ids,
        )),
        color_start: D::ColorStart::assign(fixed_color(
            &r.color_start,
            writer.ids,
            writer.context,
        )?),
        color_mid: D::ColorMid::assign(fixed_color(&r.color_mid, writer.ids, writer.context)?),
        color_end: D::ColorEnd::assign(fixed_color(&r.color_end, writer.ids, writer.context)?),
        drag: r.drag,
        legacy_mass_randomize: D::LegacyMassRandomize::assign((r.additional_flags.bits() >> 2) & 1),
        mass: r.mass,
        mass_random: r.mass_random,
        mass_size_multiplier: D::MassSizeMultiplier::assign(r.mass_size_multiplier),
        legacy_world_space: D::LegacyWorldSpace::assign((r.additional_flags.bits() >> 3) & 1),
        local_forces: nr::LocalForces((r.local_forces).bits()),
        world_forces: nr::WorldForces((r.world_forces).bits()),
        local_forces_fallback: nr::LocalForces((r.local_forces_fallback).bits()),
        world_forces_fallback: nr::WorldForces((r.world_forces_fallback).bits()),
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
        emission_rate: animated(&r.emission_rate, writer.ids),
        emitter_shape: nr::EmitterShape::from_raw(r.emitter_shape.raw()),
        shape_outer: animated(&r.shape_outer, writer.ids),
        shape_inner: animated(&r.shape_inner, writer.ids),
        outer_radius: animated(&r.outer_radius, writer.ids),
        inner_radius: animated(&r.inner_radius, writer.ids),
        shape_regions: D::ShapeRegions::assign(writer.sections(&r.shape_sections)?),
        velocity_type: nr::VelocityKind::from_raw(r.velocity_type.raw()),
        size_random_enable: r.size_random_enable,
        size_random_animation: animated(&r.size_random_animation, writer.ids),
        rotation_random_enable: r.rotation_random_enable,
        rotation_random_animation: animated(&r.rotation_random_animation, writer.ids),
        color_random_enable: r.color_random_enable,
        color_start_random: color(&r.color_start_random, writer.ids, writer.context)?,
        color_mid_random: color(&r.color_mid_random, writer.ids, writer.context)?,
        color_end_random: color(&r.color_end_random, writer.ids, writer.context)?,
        alpha_random_enable: r.alpha_random_enable,
        squirt_amount: animated(&r.squirt_amount, writer.ids),
        flipbook_start_init_index: r.flipbook_start_init_index,
        flipbook_start_stop_index: r.flipbook_start_stop_index,
        flipbook_end_init_index: r.flipbook_end_init_index,
        flipbook_end_stop_index: r.flipbook_end_stop_index,
        flipbook_mid_time: r.flipbook_mid_time,
        flipbook_columns: r.flipbook_columns,
        flipbook_rows: r.flipbook_rows,
        flipbook_column_fraction: D::FlipbookColumnFraction::assign(
            r.flipbook_column_fraction.unwrap_or(f32::INFINITY),
        ),
        flipbook_row_fraction: D::FlipbookRowFraction::assign(
            r.flipbook_row_fraction.unwrap_or(f32::INFINITY),
        ),
        bounce: r.bounce,
        friction: r.friction,
        collision_spawn_index: writer.particle(r.collision_spawn)?,
        collision_spawn_min: r.collision_spawn_min,
        collision_spawn_max: r.collision_spawn_max,
        collision_spawn_chance: r.collision_spawn_chance,
        collision_spawn_energy: r.collision_spawn_energy,
        collision_die_bounce: r.collision_die_bounce,
        instance_type: nr::ParticleInstance::from_raw(r.instance_type.raw()),
        tail_length: r.tail_length,
        instance_angle: r.instance_angle,
        instance_distance: D::InstanceDistance::assign(r.instance_distance),
        pitch_type: nr::Variation::from_raw(r.pitch.kind.raw()),
        pitch_amplitude: animated(&r.pitch.amplitude, writer.ids),
        pitch_frequency: animated(&r.pitch.frequency, writer.ids),
        yaw_type: nr::Variation::from_raw(r.yaw.kind.raw()),
        yaw_amplitude: animated(&r.yaw.amplitude, writer.ids),
        yaw_frequency: animated(&r.yaw.frequency, writer.ids),
        speed_type: nr::Variation::from_raw(r.speed.kind.raw()),
        speed_amplitude: animated(&r.speed.amplitude, writer.ids),
        speed_frequency: animated(&r.speed.frequency, writer.ids),
        size_type: nr::Variation::from_raw(r.size.kind.raw()),
        size_amplitude: animated(&r.size.amplitude, writer.ids),
        size_frequency: animated(&r.size.frequency, writer.ids),
        alpha_type: nr::Variation::from_raw(r.alpha.kind.raw()),
        alpha_amplitude: animated(&r.alpha.amplitude, writer.ids),
        alpha_frequency: animated(&r.alpha.frequency, writer.ids),
        color_type: nr::Variation::from_raw(r.color.kind.raw()),
        color_amplitude: animated(&r.color.amplitude, writer.ids),
        color_frequency: animated(&r.color.frequency, writer.ids),
        rotation_type: nr::Variation::from_raw(r.rotation.kind.raw()),
        rotation_amplitude: animated(&r.rotation.amplitude, writer.ids),
        rotation_frequency: animated(&r.rotation.frequency, writer.ids),
        horizontal_type: nr::Variation::from_raw(r.horizontal.kind.raw()),
        horizontal_amplitude: animated(&r.horizontal.amplitude, writer.ids),
        horizontal_frequency: animated(&r.horizontal.frequency, writer.ids),
        vertical_type: nr::Variation::from_raw(r.vertical.kind.raw()),
        vertical_amplitude: animated(&r.vertical.amplitude, writer.ids),
        vertical_frequency: animated(&r.vertical.frequency, writer.ids),
        particle_velocity: animated(&r.particle_velocity, writer.ids),
        phase_shift: D::PhaseShift::assign(animated(&r.phase_shift, writer.ids)),
        flags: nr::ParticleFlags((r.flags).bits()),
        rotation_flags: D::RotationFlags::assign((r.rotation_flags).bits()),
        color_smoothing: D::ColorSmoothing::assign(r.color_smoothing.raw()),
        size_smoothing: D::SizeSmoothing::assign(r.size_smoothing.raw()),
        rotation_smoothing: D::RotationSmoothing::assign(r.rotation_smoothing.raw()),
        alpha_threshold: D::AlphaThreshold::assign(animated(&r.alpha_threshold, writer.ids)),
        uv_offset: D::UvOffset::assign(animated(&r.uv_offset, writer.ids)),
        uv_angle: D::UvAngle::assign(animated(&r.uv_angle, writer.ids)),
        uv_tiling: D::UvTiling::assign(animated(&r.uv_tiling, writer.ids)),
        spline_line_data: writer.spline(&r.spline_points)?,
        wind_multiplier: r.wind_multiplier,
        lod_reduce: r.lod_reduce,
        lod_cut: r.lod_cut,
        lower_bound: animated(&r.lower_bound, writer.ids),
        upper_bound: animated(&r.upper_bound, writer.ids),
        trail_link_index: writer.particle(r.trail)?,
        trail_chance: r.trail_chance,
        trail_emission_rate: animated(&r.trail_emission_rate, writer.ids),
        splat_projection_index: writer.projector(r.projection)?,
        splat_chance: r.splat_chance,
        model_paths: writer.models(&r.models)?,
        copy_indices: copies,
        spawn_ribbon_on_bounce_chance: D::SpawnRibbonOnBounceChance::assign(
            r.spawn_ribbon_on_bounce_chance,
        ),
        ribbon_link_index: D::RibbonLinkIndex::assign(writer.ribbon(r.ribbon)?),
    })
}
