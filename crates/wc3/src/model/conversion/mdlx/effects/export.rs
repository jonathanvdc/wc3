//! Export mappings for effects.
use super::*;
use std::str::from_utf8;
pub(in crate::model::conversion::mdlx) fn overlay_effects<D: ModelDialect>(
    model: &mut native::Model<'static, D>,
    nodes: &[wr::Node<wr::V19>],
    ids: &[u32],
    updates: &BTreeMap<u32, native::Node<'static>>,
    raw: &RawDocument<'_>,
    semantic: &s::Model<'_>,
    context: &mut Context,
) -> Result<(), Error> {
    let mut particles: Vec<_> = model.particle_emitters().cloned().collect();
    let mut particles2: Vec<_> = model.particle_emitters2().cloned().collect();
    let mut ribbons: Vec<_> = model.ribbon_emitters().cloned().collect();
    let mut corn: Vec<_> = model
        .try_popcorn_emitters()
        .ok()
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    let mut attachments: Vec<_> = model.attachments().cloned().collect();
    let mut lights: Vec<_> = model.lights().cloned().collect();
    let mut typed_ids = Vec::new();
    let mut collisions: Vec<_> = model.collision_shapes().cloned().collect();
    let mut events: Vec<_> = model.event_objects().cloned().collect();
    let mut cameras: Vec<_> = model.cameras().cloned().collect();
    let mut face_fx = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        let id = *ids
            .get(index)
            .ok_or_else(|| Error::new("nodes", "native node count mismatch"))?;
        let base_node = updates[&id].clone();
        match &node.payload {
            wr::NodePayload::Wc3ParticleEmitter1(data) => {
                typed_ids.push(id);
                let path = raw.string(data.spawn_model.path)?.unwrap_or_default();
                let record = if let Some(position) = particles
                    .iter()
                    .position(|record| record.node.object_id == id)
                {
                    &mut particles[position]
                } else {
                    particles.push(
                        native::ParticleEmitter::new(base_node, path)
                            .map_err(|error| failure("particle.path", error))?,
                    );
                    particles.last_mut().unwrap()
                };
                record.emission_rate.set_value(data.emission_rate);
                record.gravity.set_value(data.gravity);
                record.longitude.set_value(data.longitude);
                record.latitude.set_value(data.latitude);
                record.life_span.set_value(data.lifespan);
                record.initial_velocity.set_value(data.speed);
                record
                    .path
                    .set_text(path)
                    .map_err(|error| failure("particle.path", error))?;
                let bits = (record.node.flags.bits() & !((1 << 15) | (1 << 16)))
                    | (u32::from(data.uses_mdl.get()) << 15)
                    | (u32::from(data.uses_tga.get()) << 16);
                record.node.flags = native::emitters::ParticleEmitterFlags(bits);
            }
            wr::NodePayload::Wc3ParticleEmitter2(data) => {
                typed_ids.push(id);
                let record = if let Some(position) = particles2
                    .iter()
                    .position(|record| record.node.object_id == id)
                {
                    &mut particles2[position]
                } else {
                    particles2.push(native::ParticleEmitter2::new(base_node));
                    particles2.last_mut().unwrap()
                };
                record.speed.set_value(data.speed);
                record.variation.set_value(data.variation);
                record.latitude.set_value(data.latitude);
                record.gravity.set_value(data.gravity);
                record.life_span = data.lifespan;
                record.emission_rate.set_value(data.emission_rate);
                record.width.set_value(data.width);
                record.length.set_value(data.length);
                record.filter_mode = match data.filter.0 {
                    0 => native::Particle2FilterMode::Blend,
                    1 => native::Particle2FilterMode::Additive,
                    2 => native::Particle2FilterMode::Modulate,
                    3 => native::Particle2FilterMode::Modulate2x,
                    4 => native::Particle2FilterMode::AlphaKey,
                    value => native::Particle2FilterMode::Unknown(value),
                };
                record.rows = data.rows;
                record.columns = data.columns;
                record.frames = match data.head_or_tail.0 {
                    0 => native::Particle2Frames::Head,
                    1 => native::Particle2Frames::Tail,
                    2 => native::Particle2Frames::Both,
                    value => native::Particle2Frames::Unknown(value),
                };
                record.tail_length = data.tail_length;
                record.time = data.time;
                let segments = [&data.start, &data.middle, &data.end];
                record.segment_colors = segments.map(|segment| segment.color);
                record.alpha = segments.map(|segment| segment.alpha);
                record.particle_scaling = segments.map(|segment| segment.scaling);
                record.uv_animations = [
                    &data.head_life,
                    &data.head_decay,
                    &data.tail_life,
                    &data.tail_decay,
                ]
                .map(|interval| [interval.start, interval.end, interval.repeat]);
                record.texture_id = data.texture;
                record.replaceable_id = data.replaceable_id;
                record.set_squirt_enabled(data.squirt.get());
                record.priority_plane = data.priority_plane as u32;
                let mut bits = record.node.flags.bits();
                for (bit, value) in [
                    (15, data.unshaded.get()),
                    (16, data.sort_prims_far_z.get()),
                    (17, data.line_emitter.get()),
                    (18, data.unfogged.get()),
                    (20, data.xy_quad.get()),
                ] {
                    bits = (bits & !(1 << bit)) | (u32::from(value) << bit);
                }
                record.node.flags = native::emitters::Particle2Flags(bits);
            }
            wr::NodePayload::Wc3RibbonEmitter(data) => {
                typed_ids.push(id);
                let record = if let Some(position) = ribbons
                    .iter()
                    .position(|record| record.node.object_id == id)
                {
                    &mut ribbons[position]
                } else {
                    ribbons.push(native::RibbonEmitter::new(base_node));
                    ribbons.last_mut().unwrap()
                };
                record.height_above.set_value(data.height_above);
                record.height_below.set_value(data.height_below);
                record.alpha.set_value(data.alpha);
                record.color.set_value(data.color);
                record.life_span = data.lifespan;
                record.texture_slot.set_value(data.texture_slot);
                record.emission_rate = data.emission_rate;
                record.rows = data.rows;
                record.columns = data.columns;
                record.material_id = data.material_slot;
                record.gravity = data.gravity;
            }
            wr::NodePayload::Wc3CornEmitter(data) => {
                typed_ids.push(id);
                let path = raw.string(data.effect.path)?.unwrap_or_default();
                let guide = raw.string(data.anim_visibility_guide)?.unwrap_or_default();
                let record = if let Some(position) =
                    corn.iter().position(|record| record.node.object_id == id)
                {
                    &mut corn[position]
                } else {
                    corn.push(
                        native::PopcornEmitter::new(base_node, path, guide)
                            .map_err(|error| failure("corn.path", error))?,
                    );
                    corn.last_mut().unwrap()
                };
                record.life_span.set_value(data.lifespan);
                record.emission_rate.set_value(data.emission_rate);
                record.speed.set_value(data.speed);
                record.color.set_value(data.color);
                record.alpha.set_value(data.alpha);
                record.replaceable_id = data.replaceable_id;
                record
                    .path
                    .set_text(path)
                    .map_err(|error| failure("corn.path", error))?;
                record
                    .visibility_guide
                    .set_text(guide)
                    .map_err(|error| failure("corn.visibility_guide", error))?;
                let mut bits = record.node.flags.bits();
                for (bit, value) in [
                    (15, data.unshaded.get()),
                    (16, data.sort_prims_far_z.get()),
                    (17, data.unfogged.get()),
                    (18, data.popcorn_scaling.get()),
                ] {
                    bits = (bits & !(1 << bit)) | (u32::from(value) << bit);
                }
                record.node.flags = native::emitters::PopcornFlags(bits);
            }
            wr::NodePayload::Attachment(data) => {
                typed_ids.push(id);
                if data.model != u32::MAX {
                    context.unsupported(
                        format!("nodes[{index}].attachment.model"),
                        "MDLX attachment cannot embed another WEM model",
                    )?;
                }
                let path = raw.string(data.asset.path)?.unwrap_or_default();
                let record = if let Some(position) = attachments
                    .iter()
                    .position(|record| record.node.object_id == id)
                {
                    &mut attachments[position]
                } else {
                    attachments.push(
                        native::Attachment::new(base_node, path, id)
                            .map_err(|error| failure("attachment.path", error))?,
                    );
                    attachments.last_mut().unwrap()
                };
                record
                    .path
                    .set_text(path)
                    .map_err(|error| failure("attachment.path", error))?;
            }
            wr::NodePayload::Light(data) => {
                typed_ids.push(id);
                let kind = match data.kind.0 {
                    0 => native::scene::LightType::Omnidirectional,
                    1 => native::scene::LightType::Directional,
                    2 => native::scene::LightType::Ambient,
                    value => native::scene::LightType::Unknown(u32::from(value)),
                };
                let record = if let Some(position) =
                    lights.iter().position(|record| record.node.object_id == id)
                {
                    &mut lights[position]
                } else {
                    lights.push(native::Light::new(base_node, kind));
                    lights.last_mut().unwrap()
                };
                record.light_type = kind;
                record.color.set_value(data.color);
                record.intensity.set_value(data.intensity);
                record.attenuation_start.set_value(data.attenuation_start);
                record.attenuation_end.set_value(data.attenuation_end);
                let mut range = record.try_shadow_casting_range().unwrap_or_default();
                range.start.set_value(data.shadow_casting_start);
                range.end.set_value(data.shadow_casting_end);
                if let Err(error) = record.try_set_shadow_casting_range(range) {
                    if data.shadow_casting_start != 0. || data.shadow_casting_end != 0. {
                        context.unsupported("light.shadow_range", error.to_string())?;
                    }
                }
                let mut falloff = record.falloff();
                falloff.quadratic.set_value(data.quadratic_falloff);
                falloff.linear.set_value(data.linear_falloff);
                falloff.damping.set_value(data.damping);
                if let Err(error) = record.try_set_falloff(falloff) {
                    if data.quadratic_falloff != 0.0005
                        || data.linear_falloff != 0.
                        || data.damping != 0.00001
                    {
                        context.unsupported("light.falloff", error.to_string())?;
                    }
                }
                if let Err(error) = record.try_set_shadow_casting(data.shadow_casting.get()) {
                    if data.shadow_casting.get() {
                        context.unsupported("light.shadow", error.to_string())?;
                    }
                }
            }
            wr::NodePayload::Collision(data) => {
                typed_ids.push(id);
                let shape = &data.shape;
                let geometry = match shape.kind {
                    wr::CollisionShapeKind::BOX => {
                        native::CollisionGeometry::Box([shape.bounds.minimum, shape.bounds.maximum])
                    }
                    wr::CollisionShapeKind::PLANE => native::CollisionGeometry::Plane([
                        shape.bounds.minimum,
                        shape.bounds.maximum,
                    ]),
                    wr::CollisionShapeKind::SPHERE => {
                        native::CollisionGeometry::Sphere(shape.sphere.center, shape.sphere.radius)
                    }
                    wr::CollisionShapeKind::CYLINDER => native::CollisionGeometry::Cylinder(
                        [shape.bounds.minimum, shape.bounds.maximum],
                        shape.sphere.radius,
                    ),
                    _ => {
                        context.unsupported(
                            format!("nodes[{index}].collision"),
                            "MDLX collision primitives cannot represent this shape",
                        )?;
                        continue;
                    }
                };
                if let Some(record) = collisions
                    .iter_mut()
                    .find(|record| record.node.object_id == id)
                {
                    record.geometry = geometry;
                } else {
                    collisions.push(native::CollisionShape {
                        node: base_node,
                        geometry,
                    });
                }
            }
            wr::NodePayload::Event(data) => {
                typed_ids.push(id);
                let mut global = u32::MAX;
                let mut frames = Vec::new();
                for entry in raw
                    .resolve::<wr::NativeEntry<wr::V2>>(node.native.entries.cast())?
                    .iter()
                {
                    let entry = entry?;
                    let name = raw.string(entry.name)?.unwrap_or_default();
                    if name == "mdlx.event.global_sequence" {
                        global = u32::try_from(entry.value).map_err(|_| {
                            Error::new("events.global_sequence", "invalid global sequence identity")
                        })?;
                    } else if name == "mdlx.event.frames" {
                        let value = raw.string(entry.text)?.unwrap_or_default();
                        if !value.is_empty() {
                            frames = value
                                .split(',')
                                .map(|frame| {
                                    frame
                                        .parse::<i32>()
                                        .map_err(|error| failure("events.frames", error))
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                        }
                    }
                }
                let mut event = native::EventObject::new(base_node, global, &frames);
                if !data.code.is_empty() {
                    event
                        .node
                        .name
                        .set_text(raw.string(data.code)?.unwrap_or_default())
                        .map_err(|error| failure("event.code", error))?;
                }
                if let Some(record) = events.iter_mut().find(|record| record.node.object_id == id) {
                    *record = event;
                } else {
                    events.push(event);
                }
            }
            wr::NodePayload::Camera(data) => {
                typed_ids.push(id);
                let mut camera_index = None;
                for entry in raw
                    .resolve::<wr::NativeEntry<wr::V2>>(node.native.entries.cast())?
                    .iter()
                {
                    let entry = entry?;
                    if raw.string(entry.name)?.unwrap_or_default() == "mdlx.camera.index" {
                        camera_index = usize::try_from(entry.value).ok();
                    }
                }
                let position = camera_index
                    .filter(|index| *index < cameras.len())
                    .unwrap_or_else(|| {
                        cameras.push(
                            native::Camera::new(&base_node.name.text())
                                .expect("existing fixed name fits"),
                        );
                        cameras.len() - 1
                    });
                let camera = &mut cameras[position];
                camera.name = base_node.name;
                camera.position = node.local.translation;
                camera.field_of_view = data.fov;
                camera.near_clip = data.near_clip;
                camera.far_clip = data.far_clip;
                camera.target_position = data.target;
                camera.translation = if semantic
                    .clips
                    .iter()
                    .flat_map(|clip| &clip.tracks)
                    .any(|track| track.target == s::Target::Translation(index as u32))
                {
                    base_node.translation
                } else {
                    None
                };
                let values = metadata(raw, node.native.clone())?;
                if let Some((code, prefix)) = values.get("mdlx.camera.variant") {
                    let variant =
                        u8::try_from(*code).map_err(|error| failure("camera.variant", error))?;
                    camera.variant = match variant {
                        0 => native::CameraVariant::Variant0,
                        3 => native::CameraVariant::Variant3,
                        1 | 2 => {
                            let bytes = if prefix.len() == 24 {
                                prefix
                                    .as_bytes()
                                    .as_chunks::<2>()
                                    .0
                                    .iter()
                                    .map(|value| {
                                        u8::from_str_radix(
                                            from_utf8(value)
                                                .map_err(|error| failure("camera.prefix", error))?,
                                            16,
                                        )
                                        .map_err(|error| failure("camera.prefix", error))
                                    })
                                    .collect::<Result<Vec<_>, Error>>()?
                            } else {
                                return Err(Error::new(
                                    "camera.prefix",
                                    "camera prefix must have twelve bytes",
                                ));
                            };
                            let prefix = bytes.as_slice().try_into().unwrap();
                            if variant == 1 {
                                native::CameraVariant::Variant1(prefix)
                            } else {
                                native::CameraVariant::Variant2(prefix)
                            }
                        }
                        code => native::CameraVariant::Unknown(code),
                    };
                }
            }
            wr::NodePayload::Wc3FaceFx(data) => {
                typed_ids.push(id);
                face_fx.push(
                    native::FaceFx::new(
                        &base_node.name.text(),
                        raw.string(data.actor.path)?.unwrap_or_default(),
                    )
                    .map_err(|error| failure("face_fx", error))?,
                );
            }
            wr::NodePayload::Helper(_) | wr::NodePayload::Bone(_) => {}
            _ => context.unsupported(
                format!("nodes[{index}].payload"),
                "Node payload has no MDLX representation",
            )?,
        }
    }
    let bones: Vec<_> = model
        .bones()
        .filter(|record| !typed_ids.contains(&record.node.object_id))
        .cloned()
        .collect();
    let helpers: Vec<_> = model
        .helpers()
        .filter(|record| !typed_ids.contains(&record.object_id))
        .cloned()
        .collect();
    model.set_bones(bones);
    model.set_helpers(helpers);
    model.set_particle_emitters(particles);
    model.set_particle_emitters2(particles2);
    model.set_ribbon_emitters(ribbons);
    if !corn.is_empty() {
        if let Err(error) = model.try_set_popcorn_emitters(corn) {
            context.unsupported("corn", error.to_string())?;
        }
    }
    model.set_attachments(attachments);
    model.set_lights(lights);
    model.set_collision_shapes(collisions);
    model.set_event_objects(events);
    model.set_cameras(cameras);
    if !face_fx.is_empty() {
        if let Err(error) = model.try_set_face_fx(face_fx) {
            context.unsupported("face_fx", error.to_string())?;
        }
    }
    Ok(())
}
