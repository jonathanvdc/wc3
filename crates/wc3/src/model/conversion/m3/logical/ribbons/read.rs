//! Ribbon controls resolve to actual skeleton and material identities.
use super::super::{particles::context::Reader, slots::Slot};
use super::spline;
use crate::model::conversion::Error;
use crate::model::m3::{self as native, records as nr};
use crate::model::semantic::profiles::m3::properties::{Field, FieldPath};
use crate::model::semantic::{profiles::m3 as surface, NodeId};
use crate::model::semantic::{
    ParticleLifetime, ParticleMotion, ProgrammedAge, ProgrammedDirection, ProgrammedForces,
    ProgrammedNoise, ProgrammedTrailGravity,
};
use native::Envelope;
pub(super) fn decode<'view, E: Envelope, D: nr::RibbonEmitterDialect>(
    r: nr::RibbonEmitter<D>,
    reader: &mut Reader<'view, '_, '_, E>,
    nodes: &[NodeId],
) -> Result<surface::Ribbon<'view>, Error>
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
    let fallback_node = *nodes.get(bone_index_fallback as usize).ok_or_else(|| {
        Error::new(
            "m3.ribbon.fallback_node",
            "fallback attachment exceeds skeleton",
        )
    })?;
    let mut additional_flags = additional_flags;
    for (word, bit, field) in [
        (legacy_speed_randomize, 0, "speed"),
        (legacy_lifespan_randomize, 1, "lifespan"),
        (legacy_mass_randomize, 2, "mass"),
        (legacy_world_space, 3, "world_space"),
    ] {
        if word > 1 {
            reader.context.unsupported(
                format!("m3.ribbon.legacy_{field}"),
                "legacy enable word is not a boolean",
            )?;
        }
        if word != 0 {
            additional_flags |= 1 << bit;
        }
    }
    Ok(surface::Ribbon {
        fallback_node,
        material: Some(reader.material(material_index)?),
        additional_flags: surface::values::RibbonAdditionalFlags(additional_flags),
        motion: ParticleMotion {
            speed: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::InitialSpeed),
                initial_speed,
            ),
            variation: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::InitialSpeedRandom),
                initial_speed_random,
            ),
            gravity: ProgrammedTrailGravity {
                x: gravity_x,
                y: gravity_y,
                magnitude: gravity,
            },
        },
        direction: ProgrammedDirection {
            initial_yaw: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::InitialYaw),
                initial_yaw,
            ),
            initial_pitch: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::InitialPitch),
                initial_pitch,
            ),
            initial_horizontal: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::InitialHorizontal),
                initial_horizontal,
            ),
            initial_vertical: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::InitialVertical),
                initial_vertical,
            ),
            drag,
        },
        lifetime: ParticleLifetime {
            duration: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::Lifetime),
                lifetime,
            ),
            variation: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::LifetimeRandom),
                lifetime_random,
            ),
        },
        kill_radius,
        appearance: ProgrammedAge {
            size_mid_time,
            color_mid_time,
            alpha_mid_time,
            rotation_mid_time,
            size_mid_hold_time,
            color_mid_hold_time,
            alpha_mid_hold_time,
            rotation_mid_hold_time,
            size_animation: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::SizeAnimation),
                size_animation,
            ),
            rotation_animation: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::RotationAnimation),
                rotation_animation,
            ),
            color_start: reader.fixed_color(
                &FieldPath::root(Field::Ribbon).child(Field::ColorStart),
                color_start,
            ),
            color_mid: reader.fixed_color(
                &FieldPath::root(Field::Ribbon).child(Field::ColorMid),
                color_mid,
            ),
            color_end: reader.fixed_color(
                &FieldPath::root(Field::Ribbon).child(Field::ColorEnd),
                color_end,
            ),
        },
        forces: ProgrammedForces {
            mass,
            mass_random,
            mass_size_multiplier,
            local_forces: surface::values::LocalForces(local_forces.bits()),
            world_forces: surface::values::WorldForces(world_forces.bits()),
            local_forces_fallback: surface::values::LocalForces(local_forces_fallback.bits()),
            world_forces_fallback: surface::values::WorldForces(world_forces_fallback.bits()),
            world_forces_mass_multiplier,
        },
        noise: ProgrammedNoise {
            noise_amplitude,
            noise_frequency,
            noise_coherence,
            noise_edge,
        },
        index_plus_length,
        emitter_shape: emitter_shape.raw(),
        ribbon_type: surface::values::RibbonKind::from_raw((ribbon_type).raw()),
        divisions,
        edges,
        inner_radius,
        max_length: reader.property(
            &FieldPath::root(Field::Ribbon).child(Field::MaxLength),
            max_length,
        ),
        spline_points: spline::read(reader, nodes, spline_ribbons)?,
        active: reader.property(&FieldPath::root(Field::Ribbon).child(Field::Active), active),
        flags: surface::values::RibbonFlags(flags.bits()),
        size_smoothing: surface::values::Smoothing::from_raw(size_smoothing),
        color_smoothing: surface::values::Smoothing::from_raw(color_smoothing),
        friction,
        bounce,
        lod_reduce,
        lod_cut,
        yaw: surface::VariationControl {
            kind: surface::values::Variation::from_raw(yaw_type),
            amplitude: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::YawAmplitude),
                yaw_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::YawFrequency),
                yaw_frequency,
            ),
        },
        pitch: surface::VariationControl {
            kind: surface::values::Variation::from_raw(pitch_type),
            amplitude: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::PitchAmplitude),
                pitch_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::PitchFrequency),
                pitch_frequency,
            ),
        },
        speed: surface::VariationControl {
            kind: surface::values::Variation::from_raw((speed_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::SpeedAmplitude),
                speed_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::SpeedFrequency),
                speed_frequency,
            ),
        },
        size: surface::VariationControl {
            kind: surface::values::Variation::from_raw((size_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::SizeAmplitude),
                size_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::SizeFrequency),
                size_frequency,
            ),
        },
        alpha: surface::VariationControl {
            kind: surface::values::Variation::from_raw((alpha_type).raw()),
            amplitude: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::AlphaAmplitude),
                alpha_amplitude,
            ),
            frequency: reader.property(
                &FieldPath::root(Field::Ribbon).child(Field::AlphaFrequency),
                alpha_frequency,
            ),
        },
        particle_velocity: reader.property(
            &FieldPath::root(Field::Ribbon).child(Field::ParticleVelocity),
            particle_velocity,
        ),
        overlay: reader.property(
            &FieldPath::root(Field::Ribbon).child(Field::Overlay),
            overlay,
        ),
        deprecated_unknown: deprecated_unknown3fbae7d6,
    })
}
