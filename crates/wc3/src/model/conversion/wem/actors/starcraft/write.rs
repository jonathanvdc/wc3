use super::super::write_asset;
use super::codec::{append, mapped, words};
use super::{feature_index, node_index, slot_index, write_control, write_variation, WriteContext};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::actors::AssetAddress;
use crate::model::semantic::{profiles::wem::actors::starcraft::*, Values};
use crate::model::wem::{records, Ref};
pub(super) fn write_particle(
    context: &mut WriteContext<'_, '_>,
    value: &Particle<'_>,
) -> Result<records::Sc2ParticleEmitterPayload, Error> {
    Ok(records::Sc2ParticleEmitterPayload {
        copy_of: u32::MAX,
        material_slot: slot_index(value.material_slot, context.slots)?,
        additional_flags: records::Sc2ParticleAdditionalFlags::from_bits_retain(
            value.additional_flags.bits(),
        ),
        initial_speed: write_control(&value.initial_speed),
        initial_speed_random: write_control(&value.initial_speed_random),
        initial_yaw: write_control(&value.initial_yaw),
        initial_pitch: write_control(&value.initial_pitch),
        initial_horizontal: write_control(&value.initial_horizontal),
        initial_vertical: write_control(&value.initial_vertical),
        lifetime: write_control(&value.lifetime),
        lifetime_random: write_control(&value.lifetime_random),
        kill_radius: value.kill_radius,
        gravity_x: value.gravity_x,
        gravity_y: value.gravity_y,
        gravity: value.gravity,
        size_mid_time: value.size_mid_time,
        color_mid_time: value.color_mid_time,
        alpha_mid_time: value.alpha_mid_time,
        rotation_mid_time: value.rotation_mid_time,
        size_mid_hold_time: value.size_mid_hold_time,
        color_mid_hold_time: value.color_mid_hold_time,
        alpha_mid_hold_time: value.alpha_mid_hold_time,
        rotation_mid_hold_time: value.rotation_mid_hold_time,
        size_animation: write_control(&value.size_animation),
        rotation_animation: write_control(&value.rotation_animation),
        color_start: write_control(&value.color_start),
        color_mid: write_control(&value.color_mid),
        color_end: write_control(&value.color_end),
        drag: value.drag,
        mass: value.mass,
        mass_random: value.mass_random,
        mass_size_multiplier: value.mass_size_multiplier,
        local_forces: records::LocalForceChannels::from_bits_retain(value.local_forces.bits()),
        world_forces: records::WorldForceChannels::from_bits_retain(value.world_forces.bits()),
        local_forces_fallback: records::LocalForceChannels::from_bits_retain(
            value.local_forces_fallback.bits(),
        ),
        world_forces_fallback: records::WorldForceChannels::from_bits_retain(
            value.world_forces_fallback.bits(),
        ),
        world_forces_mass_multiplier: value.world_forces_mass_multiplier,
        noise_amplitude: value.noise_amplitude,
        noise_frequency: value.noise_frequency,
        noise_coherence: value.noise_coherence,
        noise_edge: value.noise_edge,
        index_plus_length: value.index_plus_length,
        max_particles: value.max_particles,
        emission_rate: write_control(&value.emission_rate),
        emitter_shape: records::Sc2EmitterShape::from_raw(value.emitter_shape.raw()),
        shape_outer: write_control(&value.shape_outer),
        shape_inner: write_control(&value.shape_inner),
        outer_radius: write_control(&value.outer_radius),
        inner_radius: write_control(&value.inner_radius),
        shape_sections: words(
            context.document,
            value
                .shape_sections
                .iter()
                .map(|id| mapped(context.sections, id, "StarCraft emitter section"))
                .collect::<Result<Vec<_>, _>>()?,
        )?,
        velocity_type: records::Sc2EmissionDirection::from_raw(value.velocity_type.raw()),
        size_random_enable: value.size_random_enable,
        size_random_animation: write_control(&value.size_random_animation),
        rotation_random_enable: value.rotation_random_enable,
        rotation_random_animation: write_control(&value.rotation_random_animation),
        color_random_enable: value.color_random_enable,
        color_start_random: write_control(&value.color_start_random),
        color_mid_random: write_control(&value.color_mid_random),
        color_end_random: write_control(&value.color_end_random),
        alpha_random_enable: value.alpha_random_enable,
        squirt_amount: write_control(&value.squirt_amount),
        flipbook_start_init_index: value.flipbook_start_init_index,
        flipbook_start_stop_index: value.flipbook_start_stop_index,
        flipbook_end_init_index: value.flipbook_end_init_index,
        flipbook_end_stop_index: value.flipbook_end_stop_index,
        flipbook_mid_time: value.flipbook_mid_time,
        flipbook_columns: value.flipbook_columns,
        flipbook_rows: value.flipbook_rows,
        flipbook_column_fraction: value.flipbook_column_fraction,
        flipbook_row_fraction: value.flipbook_row_fraction,
        bounce: value.bounce,
        friction: value.friction,
        collision_spawn: feature_index(value.collision_spawn, context.features)?,
        collision_spawn_min: value.collision_spawn_min,
        collision_spawn_max: value.collision_spawn_max,
        collision_spawn_chance: value.collision_spawn_chance,
        collision_spawn_energy: value.collision_spawn_energy,
        collision_die_bounce: value.collision_die_bounce,
        instance_type: records::Sc2ParticleInstance::from_raw(value.instance_type.raw()),
        tail_length: value.tail_length,
        instance_angle: value.instance_angle,
        instance_distance: value.instance_distance,
        pitch: write_variation(&value.pitch),
        yaw: write_variation(&value.yaw),
        speed: write_variation(&value.speed),
        size: write_variation(&value.size),
        alpha: write_variation(&value.alpha),
        color: write_variation(&value.color),
        rotation: write_variation(&value.rotation),
        horizontal: write_variation(&value.horizontal),
        vertical: write_variation(&value.vertical),
        particle_velocity: write_control(&value.particle_velocity),
        phase_shift: write_control(&value.phase_shift),
        flags: records::Sc2ParticleFlags::from_bits_retain(value.flags.bits()),
        rotation_flags: records::Sc2ParticleRotationFlags::from_bits_retain(
            value.rotation_flags.bits(),
        ),
        color_smoothing: records::Sc2Smoothing::from_raw(value.color_smoothing.raw()),
        size_smoothing: records::Sc2Smoothing::from_raw(value.size_smoothing.raw()),
        rotation_smoothing: records::Sc2Smoothing::from_raw(value.rotation_smoothing.raw()),
        alpha_threshold: write_control(&value.alpha_threshold),
        uv_offset: write_control(&value.uv_offset),
        uv_angle: write_control(&value.uv_angle),
        uv_tiling: write_control(&value.uv_tiling),
        spline_points: append(
            context.document,
            &value
                .spline_points
                .iter()
                .map(|v| write_control(&v))
                .collect::<Vec<_>>(),
        )?
        .cast(),
        wind_multiplier: value.wind_multiplier,
        lod_reduce: value.lod_reduce,
        lod_cut: value.lod_cut,
        lower_bound: write_control(&value.lower_bound),
        upper_bound: write_control(&value.upper_bound),
        trail_link: feature_index(value.trail_link, context.features)?,
        trail_chance: value.trail_chance,
        trail_emission_rate: write_control(&value.trail_emission_rate),
        splat_projection_index: value.splat_projection_index,
        splat_chance: value.splat_chance,
        models: write_models(context, &value.models)?,
        spawn_ribbon_on_bounce_chance: value.spawn_ribbon_on_bounce_chance,
        ribbon_link: feature_index(value.ribbon_link, context.features)?,
        noise_smoothness: value.noise_smoothness,
    })
}
pub(super) fn write_ribbon(
    context: &mut WriteContext<'_, '_>,
    value: &Ribbon<'_>,
) -> Result<records::Sc2RibbonEmitterPayload, Error> {
    Ok(records::Sc2RibbonEmitterPayload {
        material_slot: slot_index(value.material_slot, context.slots)?,
        additional_flags: records::Sc2RibbonAdditionalFlags::from_bits_retain(
            value.additional_flags.bits(),
        ),
        initial_speed: write_control(&value.initial_speed),
        initial_speed_random: write_control(&value.initial_speed_random),
        initial_yaw: write_control(&value.initial_yaw),
        initial_pitch: write_control(&value.initial_pitch),
        initial_horizontal: write_control(&value.initial_horizontal),
        initial_vertical: write_control(&value.initial_vertical),
        lifetime: write_control(&value.lifetime),
        lifetime_random: write_control(&value.lifetime_random),
        kill_radius: value.kill_radius,
        gravity_x: value.gravity_x,
        gravity_y: value.gravity_y,
        gravity: value.gravity,
        size_mid_time: value.size_mid_time,
        color_mid_time: value.color_mid_time,
        alpha_mid_time: value.alpha_mid_time,
        rotation_mid_time: value.rotation_mid_time,
        size_mid_hold_time: value.size_mid_hold_time,
        color_mid_hold_time: value.color_mid_hold_time,
        alpha_mid_hold_time: value.alpha_mid_hold_time,
        rotation_mid_hold_time: value.rotation_mid_hold_time,
        size_animation: write_control(&value.size_animation),
        rotation_animation: write_control(&value.rotation_animation),
        color_start: write_control(&value.color_start),
        color_mid: write_control(&value.color_mid),
        color_end: write_control(&value.color_end),
        drag: value.drag,
        mass: value.mass,
        mass_random: value.mass_random,
        mass_size_multiplier: value.mass_size_multiplier,
        local_forces: records::LocalForceChannels::from_bits_retain(value.local_forces.bits()),
        world_forces: records::WorldForceChannels::from_bits_retain(value.world_forces.bits()),
        local_forces_fallback: records::LocalForceChannels::from_bits_retain(
            value.local_forces_fallback.bits(),
        ),
        world_forces_fallback: records::WorldForceChannels::from_bits_retain(
            value.world_forces_fallback.bits(),
        ),
        world_forces_mass_multiplier: value.world_forces_mass_multiplier,
        noise_amplitude: value.noise_amplitude,
        noise_frequency: value.noise_frequency,
        noise_coherence: value.noise_coherence,
        noise_edge: value.noise_edge,
        index_plus_length: value.index_plus_length,
        emitter_shape: value.emitter_shape,
        ribbon_type: records::Sc2RibbonType::from_raw(value.ribbon_type.raw()),
        divisions: value.divisions,
        edges: value.edges,
        inner_radius: value.inner_radius,
        max_length: write_control(&value.max_length),
        spline_points: write_points(context, &value.spline_points)?,
        active: write_control(&value.active),
        flags: records::Sc2RibbonFlags::from_bits_retain(value.flags.bits()),
        size_smoothing: records::Sc2Smoothing::from_raw(value.size_smoothing.raw()),
        color_smoothing: records::Sc2Smoothing::from_raw(value.color_smoothing.raw()),
        friction: value.friction,
        bounce: value.bounce,
        lod_reduce: value.lod_reduce,
        lod_cut: value.lod_cut,
        yaw: write_variation(&value.yaw),
        pitch: write_variation(&value.pitch),
        speed: write_variation(&value.speed),
        size: write_variation(&value.size),
        alpha: write_variation(&value.alpha),
        particle_velocity: write_control(&value.particle_velocity),
        overlay: write_control(&value.overlay),
        deprecated_unknown: value.deprecated_unknown,
    })
}
pub(super) fn write_ribbonpoint(
    context: &mut WriteContext<'_, '_>,
    value: &RibbonPoint,
) -> Result<records::Sc2RibbonSplinePoint, Error> {
    Ok(records::Sc2RibbonSplinePoint {
        node: node_index(value.node, context.nodes)?,
        emission_offset: value.emission_offset,
        emission_vector: value.emission_vector,
        velocity: write_control(&value.velocity),
        reserved: value.unresolved_control,
        velocity_base_factor: write_control(&value.velocity_base_factor),
        velocity_end_factor: write_control(&value.velocity_end_factor),
        yaw_variation: write_variation(&value.yaw_variation),
        pitch_variation: write_variation(&value.pitch_variation),
        velocity_variation: write_variation(&value.velocity_variation),
        yaw: write_control(&value.yaw),
        pitch: write_control(&value.pitch),
        emission_vector_norm_factor: value.emission_vector_norm_factor,
        velocity_norm_factor: value.velocity_norm_factor,
    })
}
fn write_points(
    context: &mut WriteContext<'_, '_>,
    points: &Values<'_, RibbonPoint>,
) -> Result<Ref<records::targets::Sc2RibbonSplinePoint>, Error> {
    let rows = points
        .iter()
        .map(|p| write_ribbonpoint(context, &p))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(append(context.document, &rows)?.cast())
}

fn write_models(
    context: &mut WriteContext<'_, '_>,
    values: &Values<'_, AssetAddress<'_>>,
) -> Result<Ref<records::targets::AssetKey>, Error> {
    let rows = values
        .iter()
        .map(|value| write_asset(context.document, &value))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(append(context.document, &rows)?.cast())
}
