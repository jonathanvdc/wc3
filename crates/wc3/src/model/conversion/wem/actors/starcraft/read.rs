use super::super::read_asset;
use super::codec::{lookup, read_error, read_words};
use super::{control, feature, node, slot, variation, ReadContext};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{actors::AssetAddress, SectionId};
use crate::model::semantic::{profiles::wem::actors::starcraft::*, NodeId, Values};
use crate::model::wem::{records, Ref};
pub(super) fn read_particle<'a>(
    context: &ReadContext<'a, '_, '_>,
    value: &records::Sc2ParticleEmitterPayload,
) -> Result<Particle<'a>, Error> {
    Ok(Particle {
        material_slot: slot(value.material_slot, context.slots)?,
        additional_flags: ParticleAdditionalFlags::from_bits_retain(value.additional_flags.bits()),
        initial_speed: control(&value.initial_speed),
        initial_speed_random: control(&value.initial_speed_random),
        initial_yaw: control(&value.initial_yaw),
        initial_pitch: control(&value.initial_pitch),
        initial_horizontal: control(&value.initial_horizontal),
        initial_vertical: control(&value.initial_vertical),
        lifetime: control(&value.lifetime),
        lifetime_random: control(&value.lifetime_random),
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
        size_animation: control(&value.size_animation),
        rotation_animation: control(&value.rotation_animation),
        color_start: control(&value.color_start),
        color_mid: control(&value.color_mid),
        color_end: control(&value.color_end),
        drag: value.drag,
        mass: value.mass,
        mass_random: value.mass_random,
        mass_size_multiplier: value.mass_size_multiplier,
        local_forces: LocalForceChannels::from_bits_retain(value.local_forces.bits()),
        world_forces: WorldForceChannels::from_bits_retain(value.world_forces.bits()),
        local_forces_fallback: LocalForceChannels::from_bits_retain(
            value.local_forces_fallback.bits(),
        ),
        world_forces_fallback: WorldForceChannels::from_bits_retain(
            value.world_forces_fallback.bits(),
        ),
        world_forces_mass_multiplier: value.world_forces_mass_multiplier,
        noise_amplitude: value.noise_amplitude,
        noise_frequency: value.noise_frequency,
        noise_coherence: value.noise_coherence,
        noise_edge: value.noise_edge,
        index_plus_length: value.index_plus_length,
        max_particles: value.max_particles,
        emission_rate: control(&value.emission_rate),
        emitter_shape: EmitterShape::from_raw(value.emitter_shape.raw()),
        shape_outer: control(&value.shape_outer),
        shape_inner: control(&value.shape_inner),
        outer_radius: control(&value.outer_radius),
        inner_radius: control(&value.inner_radius),
        shape_sections: read_sections(context, value.shape_sections)?,
        velocity_type: EmissionDirection::from_raw(value.velocity_type.raw()),
        size_random_enable: value.size_random_enable,
        size_random_animation: control(&value.size_random_animation),
        rotation_random_enable: value.rotation_random_enable,
        rotation_random_animation: control(&value.rotation_random_animation),
        color_random_enable: value.color_random_enable,
        color_start_random: control(&value.color_start_random),
        color_mid_random: control(&value.color_mid_random),
        color_end_random: control(&value.color_end_random),
        alpha_random_enable: value.alpha_random_enable,
        squirt_amount: control(&value.squirt_amount),
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
        collision_spawn: feature(value.collision_spawn, context.features)?,
        collision_spawn_min: value.collision_spawn_min,
        collision_spawn_max: value.collision_spawn_max,
        collision_spawn_chance: value.collision_spawn_chance,
        collision_spawn_energy: value.collision_spawn_energy,
        collision_die_bounce: value.collision_die_bounce,
        instance_type: ParticleInstance::from_raw(value.instance_type.raw()),
        tail_length: value.tail_length,
        instance_angle: value.instance_angle,
        instance_distance: value.instance_distance,
        pitch: variation(&value.pitch),
        yaw: variation(&value.yaw),
        speed: variation(&value.speed),
        size: variation(&value.size),
        alpha: variation(&value.alpha),
        color: variation(&value.color),
        rotation: variation(&value.rotation),
        horizontal: variation(&value.horizontal),
        vertical: variation(&value.vertical),
        particle_velocity: control(&value.particle_velocity),
        phase_shift: control(&value.phase_shift),
        flags: ParticleFlags::from_bits_retain(value.flags.bits()),
        rotation_flags: ParticleRotationFlags::from_bits_retain(value.rotation_flags.bits()),
        color_smoothing: Smoothing::from_raw(value.color_smoothing.raw()),
        size_smoothing: Smoothing::from_raw(value.size_smoothing.raw()),
        rotation_smoothing: Smoothing::from_raw(value.rotation_smoothing.raw()),
        alpha_threshold: control(&value.alpha_threshold),
        uv_offset: control(&value.uv_offset),
        uv_angle: control(&value.uv_angle),
        uv_tiling: control(&value.uv_tiling),
        spline_points: read_positions(context, value.spline_points)?,
        wind_multiplier: value.wind_multiplier,
        lod_reduce: value.lod_reduce,
        lod_cut: value.lod_cut,
        lower_bound: control(&value.lower_bound),
        upper_bound: control(&value.upper_bound),
        trail_link: feature(value.trail_link, context.features)?,
        trail_chance: value.trail_chance,
        trail_emission_rate: control(&value.trail_emission_rate),
        splat_projection_index: value.splat_projection_index,
        splat_chance: value.splat_chance,
        models: read_models(context, value.models)?,
        spawn_ribbon_on_bounce_chance: value.spawn_ribbon_on_bounce_chance,
        ribbon_link: feature(value.ribbon_link, context.features)?,
        noise_smoothness: value.noise_smoothness,
    })
}
pub(super) fn read_ribbon<'a>(
    context: &ReadContext<'a, '_, '_>,
    value: &records::Sc2RibbonEmitterPayload,
) -> Result<Ribbon<'a>, Error> {
    Ok(Ribbon {
        material_slot: slot(value.material_slot, context.slots)?,
        additional_flags: RibbonAdditionalFlags::from_bits_retain(value.additional_flags.bits()),
        initial_speed: control(&value.initial_speed),
        initial_speed_random: control(&value.initial_speed_random),
        initial_yaw: control(&value.initial_yaw),
        initial_pitch: control(&value.initial_pitch),
        initial_horizontal: control(&value.initial_horizontal),
        initial_vertical: control(&value.initial_vertical),
        lifetime: control(&value.lifetime),
        lifetime_random: control(&value.lifetime_random),
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
        size_animation: control(&value.size_animation),
        rotation_animation: control(&value.rotation_animation),
        color_start: control(&value.color_start),
        color_mid: control(&value.color_mid),
        color_end: control(&value.color_end),
        drag: value.drag,
        mass: value.mass,
        mass_random: value.mass_random,
        mass_size_multiplier: value.mass_size_multiplier,
        local_forces: LocalForceChannels::from_bits_retain(value.local_forces.bits()),
        world_forces: WorldForceChannels::from_bits_retain(value.world_forces.bits()),
        local_forces_fallback: LocalForceChannels::from_bits_retain(
            value.local_forces_fallback.bits(),
        ),
        world_forces_fallback: WorldForceChannels::from_bits_retain(
            value.world_forces_fallback.bits(),
        ),
        world_forces_mass_multiplier: value.world_forces_mass_multiplier,
        noise_amplitude: value.noise_amplitude,
        noise_frequency: value.noise_frequency,
        noise_coherence: value.noise_coherence,
        noise_edge: value.noise_edge,
        index_plus_length: value.index_plus_length,
        emitter_shape: value.emitter_shape,
        ribbon_type: RibbonType::from_raw(value.ribbon_type.raw()),
        divisions: value.divisions,
        edges: value.edges,
        inner_radius: value.inner_radius,
        max_length: control(&value.max_length),
        spline_points: read_points(context, value.spline_points)?,
        active: control(&value.active),
        flags: RibbonFlags::from_bits_retain(value.flags.bits()),
        size_smoothing: Smoothing::from_raw(value.size_smoothing.raw()),
        color_smoothing: Smoothing::from_raw(value.color_smoothing.raw()),
        friction: value.friction,
        bounce: value.bounce,
        lod_reduce: value.lod_reduce,
        lod_cut: value.lod_cut,
        yaw: variation(&value.yaw),
        pitch: variation(&value.pitch),
        speed: variation(&value.speed),
        size: variation(&value.size),
        alpha: variation(&value.alpha),
        particle_velocity: control(&value.particle_velocity),
        overlay: control(&value.overlay),
        deprecated_unknown: value.deprecated_unknown,
    })
}
pub(super) fn read_ribbonpoint(
    nodes: &[NodeId],
    value: &records::Sc2RibbonSplinePoint,
) -> Result<RibbonPoint, Error> {
    Ok(RibbonPoint {
        node: node(value.node, nodes)?,
        emission_offset: value.emission_offset,
        emission_vector: value.emission_vector,
        velocity: control(&value.velocity),
        unresolved_control: value.reserved,
        velocity_base_factor: control(&value.velocity_base_factor),
        velocity_end_factor: control(&value.velocity_end_factor),
        yaw_variation: variation(&value.yaw_variation),
        pitch_variation: variation(&value.pitch_variation),
        velocity_variation: variation(&value.velocity_variation),
        yaw: control(&value.yaw),
        pitch: control(&value.pitch),
        emission_vector_norm_factor: value.emission_vector_norm_factor,
        velocity_norm_factor: value.velocity_norm_factor,
    })
}

fn read_sections<'a>(
    context: &ReadContext<'a, '_, '_>,
    reference: Ref<records::targets::U32>,
) -> Result<Values<'a, SectionId>, Error> {
    let values = read_words(context.document, reference)?;
    for value in values.iter() {
        lookup(context.sections, value, "StarCraft emission section")?;
    }
    let sections = context.sections.to_vec();
    Ok(values.map(move |value| sections[value as usize]))
}
fn read_positions<'a>(
    context: &ReadContext<'a, '_, '_>,
    reference: Ref<records::targets::Sc2PropertyVector3>,
) -> Result<Values<'a, Control<[f32; 3]>>, Error> {
    let rows = context
        .document
        .resolve::<records::Sc2Property<[f32; 3]>>(reference.cast())
        .map_err(read_error)?;
    for row in rows.iter() {
        row.map_err(read_error)?;
    }
    Ok(Values::view(rows.len(), move |index| {
        control(
            &rows
                .get(index)
                .expect("checked spline position")
                .expect("validated spline position"),
        )
    }))
}
fn read_models<'a>(
    context: &ReadContext<'a, '_, '_>,
    reference: Ref<records::targets::AssetKey>,
) -> Result<Values<'a, AssetAddress<'a>>, Error> {
    let rows = context
        .document
        .resolve::<records::AssetKey>(reference.cast())
        .map_err(read_error)?;
    let paths = rows
        .iter()
        .map(|row| read_asset(context.document, &row.map_err(read_error)?).map(|value| value.path))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Values::view(rows.len(), move |index| {
        let row = rows
            .get(index)
            .expect("checked model particle")
            .expect("validated model particle");
        AssetAddress {
            namespace: row.group,
            numeric: (row.id != u32::MAX).then_some(row.id),
            path: paths[index].clone(),
        }
    }))
}

fn read_points<'a>(
    context: &ReadContext<'a, '_, '_>,
    reference: Ref<records::targets::Sc2RibbonSplinePoint>,
) -> Result<Values<'a, RibbonPoint>, Error> {
    let rows = context
        .document
        .resolve::<records::Sc2RibbonSplinePoint>(reference.cast())
        .map_err(read_error)?;
    for row in rows.iter() {
        read_ribbonpoint(context.nodes, &row.map_err(read_error)?)?;
    }
    let nodes = context.nodes.to_vec();
    Ok(Values::view(rows.len(), move |index| {
        read_ribbonpoint(
            &nodes,
            &rows
                .get(index)
                .expect("checked ribbon point")
                .expect("validated ribbon point"),
        )
        .expect("validated ribbon attachment")
    }))
}
