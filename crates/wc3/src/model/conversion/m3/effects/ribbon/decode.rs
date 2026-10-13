//! Field mapping for the ribbon decode path.
use super::super::emitter_fields::{fixed_color, property, Slot};
use super::spline_points;
use super::{native, nr, wem, wr, Envelope, Error};
use std::collections::BTreeMap;
pub(super) fn decode<E: Envelope, D: nr::RibbonEmitterDialect>(
    r: nr::RibbonEmitter<D>,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    values: &mut BTreeMap<String, i64>,
) -> Result<wr::Sc2RibbonEmitterPayload, Error>
where
    D::LegacySpeedRandomize: Slot<u32>,
    D::LegacyInitialPitch: Slot<nr::AnimationValue<f32>>,
    D::LegacyInitialYaw: Slot<nr::AnimationValue<f32>>,
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
    D::DeprecatedUnknown3fbae7d6: Slot<i32>,
    D::LegacyPitchType: Slot<u32>,
    D::LegacyPitchAmplitude: Slot<nr::AnimationValue<f32>>,
    D::LegacyPitchFrequency: Slot<nr::AnimationValue<f32>>,
    D::LegacyYawType: Slot<u32>,
    D::LegacyYawAmplitude: Slot<nr::AnimationValue<f32>>,
    D::LegacyYawFrequency: Slot<nr::AnimationValue<f32>>,
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
    D::InitialYaw: Slot<nr::AnimationValue<f32>>,
    D::InitialPitch: Slot<nr::AnimationValue<f32>>,
    D::SizeMidHoldTime: Slot<f32>,
    D::ColorMidHoldTime: Slot<f32>,
    D::AlphaMidHoldTime: Slot<f32>,
    D::RotationMidHoldTime: Slot<f32>,
    D::SizeSmoothing: Slot<u32>,
    D::ColorSmoothing: Slot<u32>,
    D::YawType: Slot<u32>,
    D::YawAmplitude: Slot<nr::AnimationValue<f32>>,
    D::YawFrequency: Slot<nr::AnimationValue<f32>>,
    D::PitchType: Slot<u32>,
    D::PitchAmplitude: Slot<nr::AnimationValue<f32>>,
    D::PitchFrequency: Slot<nr::AnimationValue<f32>>,
    D::AdditionalFlags: Slot<u32>,
    D::WorldForcesMassMultiplier: Slot<f32>,
{
    let bone_index_fallback = r.bone_index_fallback;
    let material_index = r.material_index;
    let additional_flags = r.additional_flags.slot().unwrap_or_default();
    let initial_speed = r.initial_speed;
    let initial_speed_random = r.initial_speed_random;
    let legacy_speed_randomize = r.legacy_speed_randomize.slot().unwrap_or_default();
    let initial_yaw = r
        .initial_yaw
        .slot()
        .or_else(|| r.legacy_initial_yaw.slot())
        .unwrap_or_default();
    let initial_pitch = r
        .initial_pitch
        .slot()
        .or_else(|| r.legacy_initial_pitch.slot())
        .unwrap_or_default();
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
    let mass_size_multiplier = r.mass_size_multiplier;
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
    let index_plus_length = r.index_plus_length.slot().unwrap_or_default();
    let emitter_shape = r.emitter_shape;
    let ribbon_type = r.ribbon_type;
    let divisions = r.divisions;
    let edges = r.edges;
    let inner_radius = r.inner_radius;
    let max_length = r.max_length;
    let deprecated_unknown3fbae7d6 = r.deprecated_unknown3fbae7d6.slot().unwrap_or_default();
    let spline_ribbons = r.spline_ribbons;
    let active = r.active;
    let flags = r.flags;
    let size_smoothing = r.size_smoothing.slot().unwrap_or_default();
    let color_smoothing = r.color_smoothing.slot().unwrap_or_default();
    let friction = r.friction;
    let bounce = r.bounce;
    let lod_reduce = r.lod_reduce;
    let lod_cut = r.lod_cut;
    let legacy_pitch_type = r.legacy_pitch_type.slot().unwrap_or_default();
    let yaw_type = r
        .yaw_type
        .slot()
        .or_else(|| r.legacy_yaw_type.slot())
        .unwrap_or_default();
    let yaw_amplitude = r
        .yaw_amplitude
        .slot()
        .or_else(|| r.legacy_yaw_amplitude.slot())
        .unwrap_or_default();
    let yaw_frequency = r
        .yaw_frequency
        .slot()
        .or_else(|| r.legacy_yaw_frequency.slot())
        .unwrap_or_default();
    let legacy_yaw_type = r.legacy_yaw_type.slot().unwrap_or_default();
    let pitch_type = r
        .pitch_type
        .slot()
        .or_else(|| r.legacy_pitch_type.slot())
        .unwrap_or_default();
    let pitch_amplitude = r
        .pitch_amplitude
        .slot()
        .or_else(|| r.legacy_pitch_amplitude.slot())
        .unwrap_or_default();
    let pitch_frequency = r
        .pitch_frequency
        .slot()
        .or_else(|| r.legacy_pitch_frequency.slot())
        .unwrap_or_default();
    let speed_type = r.speed_type;
    let speed_amplitude = r.speed_amplitude;
    let speed_frequency = r.speed_frequency;
    let size_type = r.size_type;
    let size_amplitude = r.size_amplitude;
    let size_frequency = r.size_frequency;
    let alpha_type = r.alpha_type;
    let alpha_amplitude = r.alpha_amplitude;
    let alpha_frequency = r.alpha_frequency;
    let particle_velocity = r.particle_velocity;
    let overlay = r.overlay;
    values.insert(
        "m3.ribbon.bone_index_fallback".into(),
        i64::from(bone_index_fallback),
    );
    values.insert(
        "m3.ribbon.legacy_speed_randomize".into(),
        i64::from(legacy_speed_randomize),
    );
    values.insert(
        "m3.ribbon.legacy_lifespan_randomize".into(),
        i64::from(legacy_lifespan_randomize),
    );
    values.insert(
        "m3.ribbon.legacy_size_mid_time".into(),
        i64::from(legacy_size_mid_time.to_bits()),
    );
    values.insert(
        "m3.ribbon.legacy_rotation_mid_time".into(),
        i64::from(legacy_rotation_mid_time.to_bits()),
    );
    values.insert(
        "m3.ribbon.legacy_color_mid_time".into(),
        i64::from(legacy_color_mid_time.to_bits()),
    );
    values.insert(
        "m3.ribbon.legacy_alpha_mid_time".into(),
        i64::from(legacy_alpha_mid_time.to_bits()),
    );
    values.insert(
        "m3.ribbon.legacy_mass_randomize".into(),
        i64::from(legacy_mass_randomize),
    );
    values.insert(
        "m3.ribbon.legacy_world_space".into(),
        i64::from(legacy_world_space),
    );
    values.insert(
        "m3.ribbon.deprecated_unknown3fbae7d6".into(),
        i64::from(deprecated_unknown3fbae7d6),
    );
    values.insert(
        "m3.ribbon.legacy_pitch_type".into(),
        i64::from(legacy_pitch_type),
    );
    values.insert(
        "m3.ribbon.legacy_yaw_type".into(),
        i64::from(legacy_yaw_type),
    );
    values.insert("m3.ribbon.source_version".into(), i64::from(D::VERSION));
    Ok(wr::Sc2RibbonEmitterPayload {
        material_slot: material_index,
        additional_flags: wr::Sc2RibbonAdditionalFlags(additional_flags),
        initial_speed: property(values, "ribbon.initial_speed", initial_speed),
        initial_speed_random: property(values, "ribbon.initial_speed_random", initial_speed_random),
        initial_yaw: property(values, "ribbon.initial_yaw", initial_yaw),
        initial_pitch: property(values, "ribbon.initial_pitch", initial_pitch),
        initial_horizontal: property(values, "ribbon.initial_horizontal", initial_horizontal),
        initial_vertical: property(values, "ribbon.initial_vertical", initial_vertical),
        lifetime: property(values, "ribbon.lifetime", lifetime),
        lifetime_random: property(values, "ribbon.lifetime_random", lifetime_random),
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
        size_animation: property(values, "ribbon.size_animation", size_animation),
        rotation_animation: property(values, "ribbon.rotation_animation", rotation_animation),
        color_start: fixed_color(values, "ribbon.color_start", color_start),
        color_mid: fixed_color(values, "ribbon.color_mid", color_mid),
        color_end: fixed_color(values, "ribbon.color_end", color_end),
        drag,
        mass,
        mass_random,
        mass_size_multiplier,
        local_forces: wr::LocalForceChannels(local_forces.bits()),
        world_forces: wr::WorldForceChannels(world_forces.bits()),
        local_forces_fallback: wr::LocalForceChannels(local_forces_fallback.bits()),
        world_forces_fallback: wr::WorldForceChannels(world_forces_fallback.bits()),
        world_forces_mass_multiplier,
        noise_amplitude,
        noise_frequency,
        noise_coherence,
        noise_edge,
        index_plus_length,
        emitter_shape: emitter_shape.raw(),
        ribbon_type: wr::Sc2RibbonType::from_raw((ribbon_type).raw()),
        divisions,
        edges,
        inner_radius,
        max_length: property(values, "ribbon.max_length", max_length),
        spline_points: spline_points(source, doc, spline_ribbons, values)?,
        active: property(values, "ribbon.active", active),
        flags: wr::Sc2RibbonFlags(flags.bits()),
        size_smoothing: wr::Sc2Smoothing::from_raw(size_smoothing),
        color_smoothing: wr::Sc2Smoothing::from_raw(color_smoothing),
        friction,
        bounce,
        lod_reduce,
        lod_cut,
        yaw: wr::Sc2Variation {
            type_: wr::Sc2VariationKind::from_raw(yaw_type),
            amplitude: property(values, "ribbon.yaw_amplitude", yaw_amplitude),
            frequency: property(values, "ribbon.yaw_frequency", yaw_frequency),
        },
        pitch: wr::Sc2Variation {
            type_: wr::Sc2VariationKind::from_raw(pitch_type),
            amplitude: property(values, "ribbon.pitch_amplitude", pitch_amplitude),
            frequency: property(values, "ribbon.pitch_frequency", pitch_frequency),
        },
        speed: wr::Sc2Variation {
            type_: wr::Sc2VariationKind::from_raw((speed_type).raw()),
            amplitude: property(values, "ribbon.speed_amplitude", speed_amplitude),
            frequency: property(values, "ribbon.speed_frequency", speed_frequency),
        },
        size: wr::Sc2Variation {
            type_: wr::Sc2VariationKind::from_raw((size_type).raw()),
            amplitude: property(values, "ribbon.size_amplitude", size_amplitude),
            frequency: property(values, "ribbon.size_frequency", size_frequency),
        },
        alpha: wr::Sc2Variation {
            type_: wr::Sc2VariationKind::from_raw((alpha_type).raw()),
            amplitude: property(values, "ribbon.alpha_amplitude", alpha_amplitude),
            frequency: property(values, "ribbon.alpha_frequency", alpha_frequency),
        },
        particle_velocity: property(values, "ribbon.particle_velocity", particle_velocity),
        overlay: property(values, "ribbon.overlay", overlay),
        deprecated_unknown: deprecated_unknown3fbae7d6,
    })
}
