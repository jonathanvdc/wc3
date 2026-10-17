//! Feature-local fields and curves become typed native effect records.
use super::animation::{animated, animated_spline, fixed_alpha, time_ms};
use super::*;
use crate::model::materials::m2::material_flags;
use crate::model::Axis;
use zerovec::ZeroVec;
fn anchor(
    model: &Graph<'_>,
    node: semantic::NodeId,
    bones: &[semantic::NodeId],
) -> Result<(u32, [f32; 3]), Error> {
    let value = model
        .node(node)
        .ok_or_else(|| Error::new("m2.actor", "missing node"))?;
    if let Some(index) = bones.iter().position(|n| *n == node) {
        return Ok((index as u32, [0.; 3]));
    }
    let parent = value
        .parent
        .and_then(|id| bones.iter().position(|n| *n == id))
        .ok_or_else(|| {
            Error::new(
                "m2.actor.anchor",
                "feature requires an exported anchor bone",
            )
        })?;
    Ok((parent as u32, value.transform.translation))
}
pub(super) fn export<V>(
    model: &Graph<'_>,
    nodes: &[semantic::NodeId],
    textures: &[semantic::TextureId],
    core: &mut Core<'static, V>,
    starts: &[u32],
    globals: &mut Vec<u32>,
    context: &mut Context,
) -> Result<(), Error>
where
    V: ModelLayout + ParticleConversion + CameraConversion,
    V::Timeline<'static>: TimelineFactory,
    V::TrackValues<'static, [f32; 3]>: ValuesFactory<[f32; 3]>,
    V::TrackValues<'static, [f32; 4]>: ValuesFactory<[f32; 4]>,
    V::TrackValues<'static, f32>: ValuesFactory<f32>,
    V::TrackValues<'static, u8>: ValuesFactory<u8>,
    V::TrackValues<'static, u32>: ValuesFactory<u32>,
    V::TrackValues<'static, i16>: ValuesFactory<i16>,
    V::TrackValues<'static, u16>: ValuesFactory<u16>,
    V::SplineValues<'static, f32>: SplineFactory<f32>,
    V::SplineValues<'static, [f32; 3]>: SplineFactory<[f32; 3]>,
{
    for (feature_id, attached) in model.features() {
        let feature = &attached.payload;
        match feature {
            profile::ActorKind::Attachment(value) => {
                let (bone, position) = anchor(model, attached.node, nodes)?;
                core.attachments.push(native::records::Attachment {
                    id: value.identity,
                    bone_id: u16::try_from(bone)
                        .map_err(|_| Error::new("m2.attachment", "bone index exceeds u16"))?,
                    position,
                    animate: animated(
                        model,
                        &value.visibility,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(u8::from(v)),
                    )?,
                    ..Default::default()
                })
            }
            profile::ActorKind::Light(value) => {
                let placement = model.node(attached.node).unwrap();
                let (bone, position) = if placement.parent.is_none() {
                    (-1, placement.transform.translation)
                } else {
                    let (b, p) = anchor(model, attached.node, nodes)?;
                    (
                        i16::try_from(b)
                            .map_err(|_| Error::new("m2.light", "bone index exceeds i16"))?,
                        p,
                    )
                };
                core.lights.push(native::records::Light {
                    kind: LightKind::from_raw(value.kind.raw()),
                    bone_id: bone,
                    position,
                    ambient_color: animated(
                        model,
                        &value.ambient.color,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    ambient_intensity: animated(
                        model,
                        &value.ambient.intensity,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    diffuse_color: animated(
                        model,
                        &value.diffuse.color,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    diffuse_intensity: animated(
                        model,
                        &value.diffuse.intensity,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    attenuation_start: animated(
                        model,
                        &value.attenuation[0],
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    attenuation_end: animated(
                        model,
                        &value.attenuation[1],
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    visibility: animated(
                        model,
                        &value.activation,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(u8::from(v)),
                    )?,
                })
            }
            profile::ActorKind::Ribbon(value) => {
                if value.geometry.axis != Axis::PositiveZ {
                    context.unsupported(
                        "m2.ribbon.geometry.axis",
                        "M2 ribbon cross-sections require the native local positive Z axis",
                    )?;
                }
                let (bone, position) = anchor(model, attached.node, nodes)?;
                let texture_indices = value
                    .rendering
                    .textures
                    .iter()
                    .map(|id| {
                        textures
                            .iter()
                            .position(|t| t == id)
                            .ok_or_else(|| Error::new("m2.ribbon.texture", "missing resource"))
                            .and_then(|i| {
                                u16::try_from(i).map_err(|_| {
                                    Error::new("m2.ribbon.texture", "index exceeds u16")
                                })
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut material_indices = Vec::new();
                for id in &value.rendering.materials {
                    let material = model
                        .material(*id)
                        .ok_or_else(|| Error::new("m2.ribbon.material", "missing surface"))?;
                    for step in &material.surface.steps {
                        let surface = step.declaration();
                        material_indices.push(
                            u16::try_from(core.materials.len()).map_err(|_| {
                                Error::new("m2.ribbon.material", "too many materials")
                            })?,
                        );
                        let mut materials: Vec<_> = core.materials.iter().collect();
                        materials.push(native::records::Material {
                            flags: MaterialFlags::from_bits_retain(
                                material_flags(&surface.state.rasterization).bits(),
                            ),
                            blending_mode: BlendMode::from_raw(surface.state.blend.raw()),
                        });
                        core.materials = ZeroVec::alloc_from_slice(&materials)
                    }
                }
                core.ribbons.push(native::records::RibbonEmitter {
                    id: value.controls.id,
                    bone_id: bone,
                    position,
                    texture_indices: ZeroVec::alloc_from_slice(&texture_indices),
                    material_indices: ZeroVec::alloc_from_slice(&material_indices),
                    color: animated(
                        model,
                        &value.appearance.color,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    alpha: animated(
                        model,
                        &value.appearance.opacity,
                        starts,
                        globals,
                        context,
                        fixed_alpha,
                    )?,
                    height_above: animated(
                        model,
                        &value.geometry.height_above,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    height_below: animated(
                        model,
                        &value.geometry.height_below,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    edges_per_second: value.emission_rate,
                    edge_lifetime: value.lifetime,
                    gravity: value.controls.angle,
                    texture_rows: value.appearance.atlas[0],
                    texture_columns: value.appearance.atlas[1],
                    texture_slot: animated(
                        model,
                        &value.appearance.texture_slot,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(v),
                    )?,
                    visibility: animated(
                        model,
                        &value.controls.enabled,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(u8::from(v)),
                    )?,
                    modern: V::ribbon_tail_from(
                        [
                            value.controls.priority_plane,
                            i16::from(value.controls.color_index),
                            0,
                        ],
                        context,
                    )?,
                })
            }
            profile::ActorKind::Camera(value) => {
                let placement = model.node(attached.node).unwrap();
                let mut camera = native::records::Camera::<V>::new();
                camera.kind = CameraKind::from_raw(value.convention.raw());
                camera.position_base = placement.transform.translation;
                camera.target_position_base = value.target;
                camera.near_clip = value.clipping[0];
                camera.far_clip = value.clipping[1];
                camera.positions = animated_spline(
                    model,
                    &value.eye_offset,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                camera.target_positions = animated_spline(
                    model,
                    &value.target_offset,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                camera.roll =
                    animated_spline(model, &value.roll, starts, globals, context, |v, _| Ok(v))?;
                V::write_logical_fov(
                    model,
                    &value.field_of_view,
                    &mut camera,
                    starts,
                    globals,
                    context,
                )?;
                core.cameras.push(camera)
            }
            profile::ActorKind::Event(value) => {
                let (bone, position) = anchor(model, attached.node, nodes)?;
                let shared: Vec<_> = model
                    .shared_events()
                    .iter()
                    .filter(|e| e.feature == Some(feature_id))
                    .collect();
                let mut global = u16::MAX;
                let mut leaves = Vec::new();
                if let Some(first) = shared.first() {
                    let semantic::Clock::Shared(clock) = first.clock else {
                        return Err(Error::new(
                            "m2.event.clock",
                            "shared event requires shared clock",
                        ));
                    };
                    let duration = time_ms(model.clock(clock).unwrap().period(), context)?;
                    let i = globals
                        .iter()
                        .position(|d| *d == duration)
                        .unwrap_or_else(|| {
                            globals.push(duration);
                            globals.len() - 1
                        });
                    global = u16::try_from(i)
                        .map_err(|_| Error::new("m2.event.clock", "too many clocks"))?;
                    leaves.push(
                        shared
                            .iter()
                            .map(|e| time_ms(e.time, context))
                            .collect::<Result<Vec<_>, _>>()?,
                    )
                } else {
                    for (index, (_, clip)) in model.clips().iter().enumerate() {
                        leaves.push(
                            clip.events()
                                .iter()
                                .filter(|e| e.feature == Some(feature_id))
                                .map(|e| {
                                    time_ms(e.time, context).and_then(|t| {
                                        if V::VERSION < 264 {
                                            t.checked_add(starts[index]).ok_or_else(|| {
                                                Error::new("m2.event", "legacy time overflow")
                                            })
                                        } else {
                                            Ok(t)
                                        }
                                    })
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        )
                    }
                }
                core.events.push(native::records::Event {
                    identifier: value.identifier,
                    data: value.data,
                    bone_id: bone,
                    position,
                    enabled: native::records::TrackBase {
                        interpolation: NativeInterpolation::Step,
                        global_sequence: global,
                        timeline: TimelineFactory::build(leaves, global != u16::MAX),
                    },
                })
            }
            profile::ActorKind::Particle(value) => {
                let (bone, position) = anchor(model, attached.node, nodes)?;
                // Graph planning emits container-level programmes separately.
                // The standalone record conversion rejects losing these fields.
                let mut record = value.as_ref().clone();
                record.details.rendering.texture_program = None;
                let mut particle = V::write_logical_particle(&record, textures, context, model)?;
                particle.bone_id = u16::try_from(bone)
                    .map_err(|_| Error::new("m2.particle", "bone exceeds u16"))?;
                particle.position = position;
                particle.emission_speed = animated(
                    model,
                    &value.motion.speed,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                particle.speed_variation = animated(
                    model,
                    &value.motion.variation,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                particle.vertical_range = animated(
                    model,
                    &value.details.shape.vertical_range,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                particle.horizontal_range = animated(
                    model,
                    &value.details.shape.horizontal_range,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                particle.lifespan = animated(
                    model,
                    &value.lifetime.duration,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                particle.emission_rate = animated(
                    model,
                    &value.emission.rate,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                particle.emission_area_width = animated(
                    model,
                    &value.details.shape.area_width,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                particle.emission_area_length = animated(
                    model,
                    &value.details.shape.area_length,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                particle.z_source = animated(
                    model,
                    &value.details.shape.z_source,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v),
                )?;
                particle.enabled = animated(
                    model,
                    &value.emission.activation,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(u8::from(v)),
                )?;
                let packed = particle.flags.packed_gravity();
                particle.gravity = animated(
                    model,
                    &value.motion.gravity,
                    starts,
                    globals,
                    context,
                    |v, c| gravity_word(v, packed, c, "m2.particle.gravity"),
                )?;
                let required = V::required_flags(&particle);
                core.flags = ModelFlags::from_bits_retain(core.flags.bits() | required.bits());
                core.particles.push(particle)
            }

            profile::ActorKind::GeometryAnimation(unavailable)
            | profile::ActorKind::CollisionShape(unavailable)
            | profile::ActorKind::ParticleProgram(unavailable)
            | profile::ActorKind::FaceActor(unavailable)
            | profile::ActorKind::LightProgram(unavailable)
            | profile::ActorKind::Extent(unavailable)
            | profile::ActorKind::ParticleCopy(unavailable)
            | profile::ActorKind::Projector(unavailable)
            | profile::ActorKind::Force(unavailable)
            | profile::ActorKind::Warp(unavailable)
            | profile::ActorKind::HitTest(unavailable)
            | profile::ActorKind::AttachmentVolume(unavailable)
            | profile::ActorKind::Billboard(unavailable)
            | profile::ActorKind::Turret(unavailable)
            | profile::ActorKind::IkJoint(unavailable)
            | profile::ActorKind::IkTwoJoint(unavailable)
            | profile::ActorKind::IkCcd(unavailable)
            | profile::ActorKind::OneBoneSolver(unavailable)
            | profile::ActorKind::ShadowBox(unavailable)
            | profile::ActorKind::ViewVolume(unavailable)
            | profile::ActorKind::AtlasParticles(unavailable)
            | profile::ActorKind::PointLight(unavailable) => match *unavailable {},
        }
    }
    Ok(())
}
