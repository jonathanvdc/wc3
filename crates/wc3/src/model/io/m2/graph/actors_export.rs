//! Feature-local fields and curves become typed native effect records.
use super::animation::{animated, animated_spline, fixed_alpha, time_ms};
use super::*;
use crate::model::error::Location;
use crate::model::materials::m2::material_flags;
use crate::model::Axis;
use crate::model::TimeSource;
use zerovec::ZeroVec;
fn anchor(
    model: &Graph<'_>,
    node: semantic::NodeId,
    bones: &[semantic::NodeId],
) -> Result<(u32, [f32; 3]), Error> {
    let value = model.node(node).ok_or_else(|| {
        Error::requirement(Location::new().field("m2").field("actor"), "missing node")
    })?;
    if let Some(index) = bones.iter().position(|n| *n == node) {
        return Ok((index as u32, [0.; 3]));
    }
    let parent = value
        .parent
        .and_then(|id| bones.iter().position(|n| *n == id))
        .ok_or_else(|| {
            Error::requirement(
                Location::new().field("m2").field("actor").field("anchor"),
                "feature requires an exported anchor bone",
            )
        })?;
    Ok((
        parent as u32,
        (value.transform.defaults(model.values()).translation).to_array(),
    ))
}
pub(super) fn export<V>(
    model: &Graph<'_>,
    nodes: &[semantic::NodeId],
    textures: &[semantic::Texture],
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
    for (_, attached) in model.features() {
        let feature = &attached.payload;
        match feature {
            profile::ActorKind::Attachment(value) => {
                let (bone, position) = anchor(model, attached.node, nodes)?;
                core.attachments.push(native::records::Attachment {
                    id: value.identity,
                    bone_id: u16::try_from(bone).map_err(|_| {
                        Error::requirement(
                            Location::new().field("m2").field("attachment"),
                            "bone index exceeds u16",
                        )
                    })?,
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
                    (
                        -1,
                        placement
                            .transform
                            .defaults(model.values())
                            .translation
                            .to_array(),
                    )
                } else {
                    let (b, p) = anchor(model, attached.node, nodes)?;
                    (
                        i16::try_from(b).map_err(|_| {
                            Error::requirement(
                                Location::new().field("m2").field("light"),
                                "bone index exceeds i16",
                            )
                        })?,
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
                        |v, _| Ok(v.to_array()),
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
                        |v, _| Ok(v.to_array()),
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
                        Location::new()
                            .field("m2")
                            .field("ribbon")
                            .field("geometry")
                            .field("axis"),
                        "M2 ribbon cross-sections require the native local positive Z axis",
                    )?;
                }
                let (bone, position) = anchor(model, attached.node, nodes)?;
                let texture_indices = value
                    .rendering
                    .textures
                    .iter()
                    .map(|id| {
                        texture_index(model, textures, *id)
                            .ok_or_else(|| {
                                Error::requirement(
                                    Location::new().field("m2").field("ribbon").field("texture"),
                                    "missing resource",
                                )
                            })
                            .and_then(|i| {
                                u16::try_from(i).map_err(|_| {
                                    Error::requirement(
                                        Location::new()
                                            .field("m2")
                                            .field("ribbon")
                                            .field("texture"),
                                        "index exceeds u16",
                                    )
                                })
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut material_indices = Vec::new();
                for id in &value.rendering.materials {
                    let material = model.material(*id).ok_or_else(|| {
                        Error::requirement(
                            Location::new()
                                .field("m2")
                                .field("ribbon")
                                .field("material"),
                            "missing surface",
                        )
                    })?;
                    for step in &material.surface.steps {
                        let surface = step.declaration();
                        material_indices.push(u16::try_from(core.materials.len()).map_err(
                            |_| {
                                Error::requirement(
                                    Location::new()
                                        .field("m2")
                                        .field("ribbon")
                                        .field("material"),
                                    "too many materials",
                                )
                            },
                        )?);
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
                let transform_slot = if let Some(transform) = &value.rendering.texture_transform {
                    let encoded = native::records::TextureTransform {
                        translation: animated(
                            model,
                            &transform.translation,
                            starts,
                            globals,
                            context,
                            |v, _| Ok(v.to_array()),
                        )?,
                        rotation: animated(
                            model,
                            &transform.rotation,
                            starts,
                            globals,
                            context,
                            |v, _| Ok(v.to_array()),
                        )?,
                        scaling: animated(
                            model,
                            &transform.scale,
                            starts,
                            globals,
                            context,
                            |v, _| Ok(v.to_array()),
                        )?,
                    };
                    let index = core
                        .texture_transforms
                        .iter()
                        .position(|value| value == &encoded)
                        .unwrap_or_else(|| {
                            core.texture_transforms.push(encoded);
                            core.texture_transforms.len() - 1
                        });
                    let index = u16::try_from(index).map_err(|_| {
                        Error::requirement(
                            Location::new()
                                .field("m2")
                                .field("ribbon")
                                .field("transform"),
                            "too many texture transforms",
                        )
                    })?;
                    let mut combos: Vec<_> = core.texture_transform_combos.iter().collect();
                    let slot = combos
                        .iter()
                        .position(|value| *value == index)
                        .unwrap_or_else(|| {
                            combos.push(index);
                            combos.len() - 1
                        });
                    let slot = i8::try_from(slot).map_err(|_| {
                        Error::requirement(
                            Location::new()
                                .field("m2")
                                .field("ribbon")
                                .field("transform"),
                            "lookup index exceeds signed ribbon field",
                        )
                    })?;
                    core.texture_transform_combos = ZeroVec::alloc_from_slice(&combos);
                    i16::from(slot)
                } else {
                    -1
                };
                core.ribbons.push(native::records::RibbonEmitter {
                    id: value.behavior.id,
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
                        |v, _| Ok(v.to_array()),
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
                    gravity: value.behavior.angle,
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
                        &value.behavior.enabled,
                        starts,
                        globals,
                        context,
                        |v, _| Ok(u8::from(v)),
                    )?,
                    modern: V::ribbon_tail_from(
                        [
                            value.rendering.priority_plane,
                            i16::from(value.rendering.color_index),
                            transform_slot,
                        ],
                        context,
                    )?,
                })
            }
            profile::ActorKind::Camera(value) => {
                let placement = model.node(attached.node).unwrap();
                let mut camera = native::records::Camera::<V>::new();
                camera.kind =
                    CameraKind::from_raw(value.role.expect("validated M2 camera role").raw());
                camera.position_base =
                    (placement.transform.defaults(model.values()).translation).to_array();
                camera.target_position_base = (value.target).to_array();
                camera.near_clip = value.projection.clipping[0];
                camera.far_clip = value.projection.clipping[1];
                camera.positions = animated_spline(
                    model,
                    &value.eye_offset,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v.to_array()),
                )?;
                camera.target_positions = animated_spline(
                    model,
                    &value.target_offset,
                    starts,
                    globals,
                    context,
                    |v, _| Ok(v.to_array()),
                )?;
                camera.roll =
                    animated_spline(model, &value.roll, starts, globals, context, |v, _| Ok(v))?;
                V::write_logical_fov(
                    model,
                    &value.projection.field_of_view,
                    &mut camera,
                    starts,
                    globals,
                    context,
                )?;
                core.cameras.push(camera)
            }
            profile::ActorKind::Particle(value) => {
                let (bone, position) = anchor(model, attached.node, nodes)?;
                // Graph planning emits container-level programmes separately.
                // The standalone record conversion rejects losing these fields.
                let mut record = value.as_ref().clone();
                record.details.rendering.texture_program = None;
                let mut particle = V::write_logical_particle(&record, textures, context, model)?;
                particle.bone_id = u16::try_from(bone).map_err(|_| {
                    Error::requirement(
                        Location::new().field("m2").field("particle"),
                        "bone exceeds u16",
                    )
                })?;
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
                    |v, c| {
                        gravity_word(
                            (v).to_array(),
                            packed,
                            c,
                            &Location::new()
                                .field("m2")
                                .field("particle")
                                .field("gravity"),
                        )
                    },
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
            | profile::ActorKind::Particle2(unavailable)
            | profile::ActorKind::PointLight(unavailable) => match *unavailable {},
        }
    }
    for (source_id, value) in model.event_sources() {
        let (bone, position) = anchor(model, value.node, nodes)?;
        let mut global = u16::MAX;
        let mut leaves = Vec::new();
        if let TimeSource::Shared(clock) = value.clock {
            let duration = time_ms(model.clock(clock).unwrap().period(), context)?;
            let i = globals
                .iter()
                .position(|d| *d == duration)
                .unwrap_or_else(|| {
                    globals.push(duration);
                    globals.len() - 1
                });
            global = u16::try_from(i).map_err(|_| {
                Error::requirement(
                    Location::new().field("m2").field("event").field("clock"),
                    "too many clocks",
                )
            })?;
            leaves.push(
                model
                    .global_events()
                    .filter(|event| event.source == source_id)
                    .map(|event| time_ms(event.time, context))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        } else {
            for (index, (id, clip)) in model.clips().iter().enumerate() {
                leaves.push(
                    model
                        .clip_events(id)
                        .filter(|e| {
                            e.source == source_id
                                && e.time >= clip.source_start
                                && e.time.as_seconds()
                                    <= clip.source_start.as_seconds() + clip.duration().as_seconds()
                        })
                        .map(|e| {
                            time_ms(
                                Time::seconds(
                                    e.time.as_seconds() - clip.source_start.as_seconds(),
                                )?,
                                context,
                            )
                            .and_then(|t| {
                                if V::VERSION < 264 {
                                    t.checked_add(starts[index]).ok_or_else(|| {
                                        Error::requirement(
                                            Location::new().field("m2").field("event"),
                                            "legacy time overflow",
                                        )
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
    Ok(())
}
