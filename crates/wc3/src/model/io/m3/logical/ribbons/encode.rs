//! Mechanical ribbon encoding from complete logical emitter controls.
use super::super::{
    particles::writer::Writer,
    slots::Assign,
    write::{animated, fixed_color},
};
use super::spline;
use crate::model::conversion::Error;
use crate::model::formats::m3::{self as native, records as nr};
use crate::model::{profiles::m3 as surface, NodeId};
use native::Envelope;
use std::collections::BTreeMap;
pub(in crate::model::io::m3::logical) fn encode<E: Envelope, D: nr::RibbonEmitterDialect>(
    r: &surface::Ribbon<'_>,
    writer: &mut Writer<'_, '_, E>,
    bone: u32,
    bones: &BTreeMap<NodeId, u32>,
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
    if !D::DeprecatedUnknown3fbae7d6::PRESENT && r.deprecated_unknown != 0 {
        writer.context.unsupported(
            "m3.ribbon.deprecated_unknown",
            "selected native ribbon layout omits this legacy control word",
        )?;
    }
    if !D::AdditionalFlags::PRESENT && r.additional_flags.bits() & !0xF != 0 {
        writer.context.unsupported(
            "ribbon.additional_flags",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::WorldForcesMassMultiplier::PRESENT
        && r.forces.world_forces_mass_multiplier != f32::default()
    {
        writer.context.unsupported(
            "ribbon.world_forces_mass_multiplier",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::IndexPlusLength::PRESENT && r.index_plus_length != u32::default() {
        writer.context.unsupported(
            "ribbon.index_plus_length",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SizeMidHoldTime::PRESENT && r.appearance.size_mid_hold_time != f32::default() {
        writer.context.unsupported(
            "ribbon.size_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::ColorMidHoldTime::PRESENT && r.appearance.color_mid_hold_time != f32::default() {
        writer.context.unsupported(
            "ribbon.color_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::AlphaMidHoldTime::PRESENT && r.appearance.alpha_mid_hold_time != f32::default() {
        writer.context.unsupported(
            "ribbon.alpha_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::RotationMidHoldTime::PRESENT && r.appearance.rotation_mid_hold_time != f32::default() {
        writer.context.unsupported(
            "ribbon.rotation_mid_hold_time",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::SizeSmoothing::PRESENT && r.size_smoothing != Default::default() {
        writer.context.unsupported(
            "ribbon.size_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    if !D::ColorSmoothing::PRESENT && r.color_smoothing != Default::default() {
        writer.context.unsupported(
            "ribbon.color_smoothing",
            "selected source revision omits this emitter control",
        )?;
    }
    Ok(nr::RibbonEmitter {
        bone_index: u16::try_from(bone)
            .map_err(|_| Error::new("ribbon.bone", "bone index exceeds ribbon width"))?,
        bone_index_fallback: u16::try_from(
            *bones.get(&r.fallback_node).ok_or_else(|| {
                Error::new("m3.ribbon.fallback_node", "missing fallback attachment")
            })?,
        )
        .map_err(|_| {
            Error::new(
                "m3.ribbon.fallback_node",
                "fallback attachment exceeds native index width",
            )
        })?,
        material_index: writer.material(r.material)?,
        additional_flags: D::AdditionalFlags::assign((r.additional_flags).bits()),
        initial_speed: animated(&r.motion.speed, writer.ids),
        initial_speed_random: animated(&r.motion.variation, writer.ids),
        legacy_speed_randomize: D::LegacySpeedRandomize::assign(r.additional_flags.bits() & 1),
        legacy_initial_pitch: D::LegacyInitialPitch::assign(animated(
            &r.direction.initial_pitch,
            writer.ids,
        )),
        initial_yaw: D::InitialYaw::assign(animated(&r.direction.initial_yaw, writer.ids)),
        legacy_initial_yaw: D::LegacyInitialYaw::assign(animated(
            &r.direction.initial_yaw,
            writer.ids,
        )),
        initial_pitch: D::InitialPitch::assign(animated(&r.direction.initial_pitch, writer.ids)),
        initial_horizontal: animated(&r.direction.initial_horizontal, writer.ids),
        initial_vertical: animated(&r.direction.initial_vertical, writer.ids),
        lifetime: animated(&r.lifetime.duration, writer.ids),
        lifetime_random: animated(&r.lifetime.variation, writer.ids),
        legacy_lifespan_randomize: D::LegacyLifespanRandomize::assign(
            (r.additional_flags.bits() >> 1) & 1,
        ),
        kill_radius: r.kill_radius,
        gravity_x: r.motion.gravity.x,
        gravity_y: r.motion.gravity.y,
        gravity: r.motion.gravity.magnitude,
        legacy_size_animation: D::LegacySizeAnimation::assign(animated(
            &r.appearance.size_animation,
            writer.ids,
        )),
        size_mid_time: D::SizeMidTime::assign(r.appearance.size_mid_time),
        legacy_size_mid_time: D::LegacySizeMidTime::assign(r.appearance.size_mid_time),
        color_mid_time: D::ColorMidTime::assign(r.appearance.color_mid_time),
        legacy_rotation_animation: D::LegacyRotationAnimation::assign(animated(
            &r.appearance.rotation_animation,
            writer.ids,
        )),
        alpha_mid_time: D::AlphaMidTime::assign(r.appearance.alpha_mid_time),
        legacy_rotation_mid_time: D::LegacyRotationMidTime::assign(r.appearance.rotation_mid_time),
        rotation_mid_time: D::RotationMidTime::assign(r.appearance.rotation_mid_time),
        legacy_color_start: D::LegacyColorStart::assign(fixed_color(
            &r.appearance.color_start,
            writer.ids,
            writer.context,
        )?),
        size_mid_hold_time: D::SizeMidHoldTime::assign(r.appearance.size_mid_hold_time),
        legacy_color_mid: D::LegacyColorMid::assign(fixed_color(
            &r.appearance.color_mid,
            writer.ids,
            writer.context,
        )?),
        color_mid_hold_time: D::ColorMidHoldTime::assign(r.appearance.color_mid_hold_time),
        legacy_color_end: D::LegacyColorEnd::assign(fixed_color(
            &r.appearance.color_end,
            writer.ids,
            writer.context,
        )?),
        alpha_mid_hold_time: D::AlphaMidHoldTime::assign(r.appearance.alpha_mid_hold_time),
        legacy_color_mid_time: D::LegacyColorMidTime::assign(r.appearance.color_mid_time),
        rotation_mid_hold_time: D::RotationMidHoldTime::assign(r.appearance.rotation_mid_hold_time),
        legacy_alpha_mid_time: D::LegacyAlphaMidTime::assign(r.appearance.alpha_mid_time),
        size_animation: D::SizeAnimation::assign(animated(
            &r.appearance.size_animation,
            writer.ids,
        )),
        rotation_animation: D::RotationAnimation::assign(animated(
            &r.appearance.rotation_animation,
            writer.ids,
        )),
        color_start: D::ColorStart::assign(fixed_color(
            &r.appearance.color_start,
            writer.ids,
            writer.context,
        )?),
        color_mid: D::ColorMid::assign(fixed_color(
            &r.appearance.color_mid,
            writer.ids,
            writer.context,
        )?),
        color_end: D::ColorEnd::assign(fixed_color(
            &r.appearance.color_end,
            writer.ids,
            writer.context,
        )?),
        drag: r.direction.drag,
        legacy_mass_randomize: D::LegacyMassRandomize::assign((r.additional_flags.bits() >> 2) & 1),
        mass: r.forces.mass,
        mass_random: r.forces.mass_random,
        mass_size_multiplier: r.forces.mass_size_multiplier,
        legacy_world_space: D::LegacyWorldSpace::assign((r.additional_flags.bits() >> 3) & 1),
        local_forces: nr::LocalForces((r.forces.local_forces).bits()),
        world_forces: nr::WorldForces((r.forces.world_forces).bits()),
        local_forces_fallback: nr::LocalForces((r.forces.local_forces_fallback).bits()),
        world_forces_fallback: nr::WorldForces((r.forces.world_forces_fallback).bits()),
        world_forces_mass_multiplier: D::WorldForcesMassMultiplier::assign(
            r.forces.world_forces_mass_multiplier,
        ),
        noise_amplitude: r.noise.noise_amplitude,
        noise_frequency: r.noise.noise_frequency,
        noise_coherence: r.noise.noise_coherence,
        noise_edge: r.noise.noise_edge,
        index_plus_length: D::IndexPlusLength::assign(r.index_plus_length),
        emitter_shape: nr::EmitterShape::from_raw(r.emitter_shape),
        ribbon_type: nr::RibbonKind::from_raw(r.ribbon_type.raw()),
        divisions: r.divisions,
        edges: r.edges,
        inner_radius: r.inner_radius,
        max_length: animated(&r.max_length, writer.ids),
        deprecated_unknown3fbae7d6: D::DeprecatedUnknown3fbae7d6::assign(r.deprecated_unknown),
        spline_ribbons: spline::write(&r.spline_points, writer, bones)?,
        active: animated(&r.active, writer.ids),
        flags: nr::RibbonFlags((r.flags).bits()),
        size_smoothing: D::SizeSmoothing::assign(r.size_smoothing.raw()),
        color_smoothing: D::ColorSmoothing::assign(r.color_smoothing.raw()),
        friction: r.friction,
        bounce: r.bounce,
        lod_reduce: r.lod_reduce,
        lod_cut: r.lod_cut,
        legacy_pitch_type: D::LegacyPitchType::assign(r.pitch.kind.raw()),
        yaw_type: D::YawType::assign(r.yaw.kind.raw()),
        legacy_pitch_amplitude: D::LegacyPitchAmplitude::assign(animated(
            &r.pitch.amplitude,
            writer.ids,
        )),
        yaw_amplitude: D::YawAmplitude::assign(animated(&r.yaw.amplitude, writer.ids)),
        legacy_pitch_frequency: D::LegacyPitchFrequency::assign(animated(
            &r.pitch.frequency,
            writer.ids,
        )),
        yaw_frequency: D::YawFrequency::assign(animated(&r.yaw.frequency, writer.ids)),
        legacy_yaw_type: D::LegacyYawType::assign(r.yaw.kind.raw()),
        pitch_type: D::PitchType::assign(r.pitch.kind.raw()),
        legacy_yaw_amplitude: D::LegacyYawAmplitude::assign(animated(&r.yaw.amplitude, writer.ids)),
        pitch_amplitude: D::PitchAmplitude::assign(animated(&r.pitch.amplitude, writer.ids)),
        legacy_yaw_frequency: D::LegacyYawFrequency::assign(animated(&r.yaw.frequency, writer.ids)),
        pitch_frequency: D::PitchFrequency::assign(animated(&r.pitch.frequency, writer.ids)),
        speed_type: nr::Variation::from_raw(r.speed.kind.raw()),
        speed_amplitude: animated(&r.speed.amplitude, writer.ids),
        speed_frequency: animated(&r.speed.frequency, writer.ids),
        size_type: nr::Variation::from_raw(r.size.kind.raw()),
        size_amplitude: animated(&r.size.amplitude, writer.ids),
        size_frequency: animated(&r.size.frequency, writer.ids),
        alpha_type: nr::Variation::from_raw(r.alpha.kind.raw()),
        alpha_amplitude: animated(&r.alpha.amplitude, writer.ids),
        alpha_frequency: animated(&r.alpha.frequency, writer.ids),
        particle_velocity: animated(&r.particle_velocity, writer.ids),
        overlay: animated(&r.overlay, writer.ids),
    })
}
