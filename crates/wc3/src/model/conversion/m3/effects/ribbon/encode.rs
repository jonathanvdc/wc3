//! Field mapping for the ribbon encode path.
use super::super::emitter_fields::{native_fixed_color, Assign};
use super::super::metadata::native_property;
use super::native_spline_points;
use super::Context;
use super::{native, nr, wem, wr, Envelope, Error};
use std::collections::BTreeMap;
pub(super) fn encode<E: Envelope, D: nr::RibbonEmitterDialect>(
    r: wr::Sc2RibbonEmitterPayload,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    values: &BTreeMap<String, i64>,
    bone: u32,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<nr::RibbonEmitter<D>, Error>
where
    D::LegacySpeedRandomize: Assign<u32>,
    D::LegacyInitialPitch: Assign<nr::AnimationValue<f32>>,
    D::LegacyInitialYaw: Assign<nr::AnimationValue<f32>>,
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
    D::DeprecatedUnknown3fbae7d6: Assign<i32>,
    D::LegacyPitchType: Assign<u32>,
    D::LegacyPitchAmplitude: Assign<nr::AnimationValue<f32>>,
    D::LegacyPitchFrequency: Assign<nr::AnimationValue<f32>>,
    D::LegacyYawType: Assign<u32>,
    D::LegacyYawAmplitude: Assign<nr::AnimationValue<f32>>,
    D::LegacyYawFrequency: Assign<nr::AnimationValue<f32>>,
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
    D::InitialYaw: Assign<nr::AnimationValue<f32>>,
    D::InitialPitch: Assign<nr::AnimationValue<f32>>,
    D::SizeMidHoldTime: Assign<f32>,
    D::ColorMidHoldTime: Assign<f32>,
    D::AlphaMidHoldTime: Assign<f32>,
    D::RotationMidHoldTime: Assign<f32>,
    D::SizeSmoothing: Assign<u32>,
    D::ColorSmoothing: Assign<u32>,
    D::YawType: Assign<u32>,
    D::YawAmplitude: Assign<nr::AnimationValue<f32>>,
    D::YawFrequency: Assign<nr::AnimationValue<f32>>,
    D::PitchType: Assign<u32>,
    D::PitchAmplitude: Assign<nr::AnimationValue<f32>>,
    D::PitchFrequency: Assign<nr::AnimationValue<f32>>,
    D::AdditionalFlags: Assign<u32>,
    D::WorldForcesMassMultiplier: Assign<f32>,
{
    if !D::AdditionalFlags::PRESENT && r.additional_flags != Default::default() {
        context.unsupported(
            "ribbon.additional_flags",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::WorldForcesMassMultiplier::PRESENT
        && r.world_forces_mass_multiplier != Default::default()
    {
        context.unsupported(
            "ribbon.world_forces_mass_multiplier",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::IndexPlusLength::PRESENT && r.index_plus_length != Default::default() {
        context.unsupported(
            "ribbon.index_plus_length",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SizeMidHoldTime::PRESENT && r.size_mid_hold_time != Default::default() {
        context.unsupported(
            "ribbon.size_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::ColorMidHoldTime::PRESENT && r.color_mid_hold_time != Default::default() {
        context.unsupported(
            "ribbon.color_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::AlphaMidHoldTime::PRESENT && r.alpha_mid_hold_time != Default::default() {
        context.unsupported(
            "ribbon.alpha_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::RotationMidHoldTime::PRESENT && r.rotation_mid_hold_time != Default::default() {
        context.unsupported(
            "ribbon.rotation_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SizeSmoothing::PRESENT && r.size_smoothing != Default::default() {
        context.unsupported(
            "ribbon.size_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::ColorSmoothing::PRESENT && r.color_smoothing != Default::default() {
        context.unsupported(
            "ribbon.color_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    Ok(nr::RibbonEmitter {
        bone_index: u16::try_from(bone)
            .map_err(|_| Error::new("ribbon.bone", "bone index exceeds ribbon width"))?,
        bone_index_fallback: values
            .get("m3.ribbon.bone_index_fallback")
            .copied()
            .unwrap_or(i64::from(bone)) as u16,
        material_index: r.material_slot,
        additional_flags: D::AdditionalFlags::assign(r.additional_flags),
        initial_speed: native_property(values, "ribbon.initial_speed", r.initial_speed.clone()),
        initial_speed_random: native_property(
            values,
            "ribbon.initial_speed_random",
            r.initial_speed_random.clone(),
        ),
        legacy_speed_randomize: D::LegacySpeedRandomize::assign(
            values
                .get("m3.ribbon.legacy_speed_randomize")
                .copied()
                .unwrap_or(0) as u32,
        ),
        legacy_initial_pitch: D::LegacyInitialPitch::assign(native_property(
            values,
            "ribbon.initial_pitch",
            r.initial_pitch.clone(),
        )),
        initial_yaw: D::InitialYaw::assign(native_property(
            values,
            "ribbon.initial_yaw",
            r.initial_yaw.clone(),
        )),
        legacy_initial_yaw: D::LegacyInitialYaw::assign(native_property(
            values,
            "ribbon.initial_yaw",
            r.initial_yaw.clone(),
        )),
        initial_pitch: D::InitialPitch::assign(native_property(
            values,
            "ribbon.initial_pitch",
            r.initial_pitch.clone(),
        )),
        initial_horizontal: native_property(
            values,
            "ribbon.initial_horizontal",
            r.initial_horizontal.clone(),
        ),
        initial_vertical: native_property(
            values,
            "ribbon.initial_vertical",
            r.initial_vertical.clone(),
        ),
        lifetime: native_property(values, "ribbon.lifetime", r.lifetime.clone()),
        lifetime_random: native_property(
            values,
            "ribbon.lifetime_random",
            r.lifetime_random.clone(),
        ),
        legacy_lifespan_randomize: D::LegacyLifespanRandomize::assign(
            values
                .get("m3.ribbon.legacy_lifespan_randomize")
                .copied()
                .unwrap_or(0) as u32,
        ),
        kill_radius: r.kill_radius,
        gravity_x: r.gravity_x,
        gravity_y: r.gravity_y,
        gravity: r.gravity,
        legacy_size_animation: D::LegacySizeAnimation::assign(native_property(
            values,
            "ribbon.size_animation",
            r.size_animation.clone(),
        )),
        size_mid_time: D::SizeMidTime::assign(r.size_mid_time),
        legacy_size_mid_time: D::LegacySizeMidTime::assign(r.size_mid_time),
        color_mid_time: D::ColorMidTime::assign(r.color_mid_time),
        legacy_rotation_animation: D::LegacyRotationAnimation::assign(native_property(
            values,
            "ribbon.rotation_animation",
            r.rotation_animation.clone(),
        )),
        alpha_mid_time: D::AlphaMidTime::assign(r.alpha_mid_time),
        legacy_rotation_mid_time: D::LegacyRotationMidTime::assign(r.rotation_mid_time),
        rotation_mid_time: D::RotationMidTime::assign(r.rotation_mid_time),
        legacy_color_start: D::LegacyColorStart::assign(native_fixed_color(
            values,
            "ribbon.color_start",
            r.color_start.clone(),
            context,
        )?),
        size_mid_hold_time: D::SizeMidHoldTime::assign(r.size_mid_hold_time),
        legacy_color_mid: D::LegacyColorMid::assign(native_fixed_color(
            values,
            "ribbon.color_mid",
            r.color_mid.clone(),
            context,
        )?),
        color_mid_hold_time: D::ColorMidHoldTime::assign(r.color_mid_hold_time),
        legacy_color_end: D::LegacyColorEnd::assign(native_fixed_color(
            values,
            "ribbon.color_end",
            r.color_end.clone(),
            context,
        )?),
        alpha_mid_hold_time: D::AlphaMidHoldTime::assign(r.alpha_mid_hold_time),
        legacy_color_mid_time: D::LegacyColorMidTime::assign(r.color_mid_time),
        rotation_mid_hold_time: D::RotationMidHoldTime::assign(r.rotation_mid_hold_time),
        legacy_alpha_mid_time: D::LegacyAlphaMidTime::assign(r.alpha_mid_time),
        size_animation: D::SizeAnimation::assign(native_property(
            values,
            "ribbon.size_animation",
            r.size_animation.clone(),
        )),
        rotation_animation: D::RotationAnimation::assign(native_property(
            values,
            "ribbon.rotation_animation",
            r.rotation_animation.clone(),
        )),
        color_start: D::ColorStart::assign(native_fixed_color(
            values,
            "ribbon.color_start",
            r.color_start.clone(),
            context,
        )?),
        color_mid: D::ColorMid::assign(native_fixed_color(
            values,
            "ribbon.color_mid",
            r.color_mid.clone(),
            context,
        )?),
        color_end: D::ColorEnd::assign(native_fixed_color(
            values,
            "ribbon.color_end",
            r.color_end.clone(),
            context,
        )?),
        drag: r.drag,
        legacy_mass_randomize: D::LegacyMassRandomize::assign(
            values
                .get("m3.ribbon.legacy_mass_randomize")
                .copied()
                .unwrap_or(0) as u32,
        ),
        mass: r.mass,
        mass_random: r.mass_random,
        mass_size_multiplier: r.mass_size_multiplier,
        legacy_world_space: D::LegacyWorldSpace::assign(
            values
                .get("m3.ribbon.legacy_world_space")
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
        index_plus_length: D::IndexPlusLength::assign(r.index_plus_length),
        emitter_shape: r.emitter_shape,
        ribbon_type: r.ribbon_type.0,
        divisions: r.divisions,
        edges: r.edges,
        inner_radius: r.inner_radius,
        max_length: native_property(values, "ribbon.max_length", r.max_length.clone()),
        deprecated_unknown3fbae7d6: D::DeprecatedUnknown3fbae7d6::assign(r.deprecated_unknown),
        spline_ribbons: native_spline_points(source, doc, r.spline_points, values, bone_map)?,
        active: native_property(values, "ribbon.active", r.active.clone()),
        flags: r.flags,
        size_smoothing: D::SizeSmoothing::assign(r.size_smoothing.0),
        color_smoothing: D::ColorSmoothing::assign(r.color_smoothing.0),
        friction: r.friction,
        bounce: r.bounce,
        lod_reduce: r.lod_reduce,
        lod_cut: r.lod_cut,
        legacy_pitch_type: D::LegacyPitchType::assign(r.pitch.type_),
        yaw_type: D::YawType::assign(r.yaw.type_),
        legacy_pitch_amplitude: D::LegacyPitchAmplitude::assign(native_property(
            values,
            "ribbon.pitch_amplitude",
            r.pitch.amplitude.clone(),
        )),
        yaw_amplitude: D::YawAmplitude::assign(native_property(
            values,
            "ribbon.yaw_amplitude",
            r.yaw.amplitude.clone(),
        )),
        legacy_pitch_frequency: D::LegacyPitchFrequency::assign(native_property(
            values,
            "ribbon.pitch_frequency",
            r.pitch.frequency.clone(),
        )),
        yaw_frequency: D::YawFrequency::assign(native_property(
            values,
            "ribbon.yaw_frequency",
            r.yaw.frequency.clone(),
        )),
        legacy_yaw_type: D::LegacyYawType::assign(r.yaw.type_),
        pitch_type: D::PitchType::assign(r.pitch.type_),
        legacy_yaw_amplitude: D::LegacyYawAmplitude::assign(native_property(
            values,
            "ribbon.yaw_amplitude",
            r.yaw.amplitude.clone(),
        )),
        pitch_amplitude: D::PitchAmplitude::assign(native_property(
            values,
            "ribbon.pitch_amplitude",
            r.pitch.amplitude.clone(),
        )),
        legacy_yaw_frequency: D::LegacyYawFrequency::assign(native_property(
            values,
            "ribbon.yaw_frequency",
            r.yaw.frequency.clone(),
        )),
        pitch_frequency: D::PitchFrequency::assign(native_property(
            values,
            "ribbon.pitch_frequency",
            r.pitch.frequency.clone(),
        )),
        speed_type: r.speed.type_,
        speed_amplitude: native_property(
            values,
            "ribbon.speed_amplitude",
            r.speed.amplitude.clone(),
        ),
        speed_frequency: native_property(
            values,
            "ribbon.speed_frequency",
            r.speed.frequency.clone(),
        ),
        size_type: r.size.type_,
        size_amplitude: native_property(values, "ribbon.size_amplitude", r.size.amplitude.clone()),
        size_frequency: native_property(values, "ribbon.size_frequency", r.size.frequency.clone()),
        alpha_type: r.alpha.type_,
        alpha_amplitude: native_property(
            values,
            "ribbon.alpha_amplitude",
            r.alpha.amplitude.clone(),
        ),
        alpha_frequency: native_property(
            values,
            "ribbon.alpha_frequency",
            r.alpha.frequency.clone(),
        ),
        particle_velocity: native_property(
            values,
            "ribbon.particle_velocity",
            r.particle_velocity.clone(),
        ),
        overlay: native_property(values, "ribbon.overlay", r.overlay.clone()),
    })
}
