//! Native feature nodes imported directly into independent scene state.
use super::*;
fn placement<'a>(
    builder: &mut GraphBuilder<'a>,
    nodes: &[semantic::NodeId],
    bone: Option<usize>,
    position: [f32; 3],
    name: String,
) -> Result<semantic::NodeId, Error> {
    let mut node = Node::named(name);
    node.transform.translation = position;
    if let Some(bone) = bone {
        node.parent = Some(
            *nodes
                .get(bone)
                .ok_or_else(|| Error::new("m2.actor.bone", "missing anchor bone"))?,
        )
    }
    Ok(builder.add_node(node)?)
}
pub(super) fn import<'a, 'data: 'a, D: Dialect>(
    source: &'a Bundle<'data, D>,
    nodes: &[semantic::NodeId],
    textures: &[semantic::TextureId],
    clips: &[semantic::ClipId],
    clocks: &[semantic::ClockId],
    builder: &mut GraphBuilder<'a>,
) -> Result<(), Error>
where
    D::Layout: ParticleConversion + CameraConversion,
    <D::Layout as native::records::SequenceLayout>::Timing: SequenceClock,
{
    let core = source.model.core();
    let tables = sequence_tables(source, &source.key)?;
    macro_rules! bind_field {
        ($node:ident,$actor:ident,$field:ident,$native:expr,$variant:ident,$map:expr) => {{
            let property = Property::named(
                OwnerId::Feature($node),
                concat!("m2.", stringify!($actor), ".", stringify!($field)),
            );
            $actor.$field.property = Some(property.clone());
            for (seq, clip) in animation::sequences(clips) {
                let (start, _) = tables
                    .sequences
                    .get(seq)
                    .map_or((0, 0), |s| s.timing.window());
                if let Some((c, g)) = curve(source, $native, &source.key, seq, start, $map)? {
                    animation::insert(
                        builder,
                        clip,
                        Channel::$variant(bind(c, g, property.clone(), clocks)?),
                    )?
                }
            }
        }};
    }
    for (index, value) in core.attachments.iter().enumerate() {
        let node = placement(
            builder,
            nodes,
            Some(value.bone_id as usize),
            value.position,
            format!("attachment-{index}"),
        )?;
        let feature = builder.reserve_feature(node);
        let mut attachment = profile::Attachment {
            role: value.id,
            enabled: Animated::constant(initial(&value.animate).unwrap_or(1) != 0),
        };
        bind_field!(feature, attachment, enabled, &value.animate, Boolean, |v| v
            != 0);
        builder.set_feature(feature, profile::ActorKind::Attachment(attachment))?;
    }
    for (index, value) in core.lights.iter().enumerate() {
        let node = placement(
            builder,
            nodes,
            (value.bone_id >= 0).then_some(value.bone_id as usize),
            value.position,
            format!("light-{index}"),
        )?;
        let feature = builder.reserve_feature(node);
        let mut light = profile::Light {
            kind: profile::values::LightKind::from_raw(value.kind.raw()),
            ambient_color: Animated::constant(initial(&value.ambient_color).unwrap_or([0.; 3])),
            ambient_intensity: Animated::constant(initial(&value.ambient_intensity).unwrap_or(0.)),
            diffuse_color: Animated::constant(initial(&value.diffuse_color).unwrap_or([0.; 3])),
            diffuse_intensity: Animated::constant(initial(&value.diffuse_intensity).unwrap_or(0.)),
            attenuation_start: Animated::constant(initial(&value.attenuation_start).unwrap_or(0.)),
            attenuation_end: Animated::constant(initial(&value.attenuation_end).unwrap_or(0.)),
            enabled: Animated::constant(initial(&value.visibility).unwrap_or(1) != 0),
        };
        bind_field!(
            feature,
            light,
            ambient_color,
            &value.ambient_color,
            Vector3,
            |v| v
        );
        bind_field!(
            feature,
            light,
            ambient_intensity,
            &value.ambient_intensity,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            light,
            diffuse_color,
            &value.diffuse_color,
            Vector3,
            |v| v
        );
        bind_field!(
            feature,
            light,
            diffuse_intensity,
            &value.diffuse_intensity,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            light,
            attenuation_start,
            &value.attenuation_start,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            light,
            attenuation_end,
            &value.attenuation_end,
            Scalar,
            |v| v
        );
        bind_field!(feature, light, enabled, &value.visibility, Boolean, |v| v
            != 0);
        builder.set_feature(feature, profile::ActorKind::Light(light))?;
    }
    for (index, value) in core.ribbons.iter().enumerate() {
        let node = placement(
            builder,
            nodes,
            Some(value.bone_id as usize),
            value.position,
            format!("ribbon-{index}"),
        )?;
        let mut materials = Vec::new();
        for index in value.material_indices.iter() {
            let native = core
                .materials
                .get(index as usize)
                .ok_or_else(|| Error::new("m2.ribbon.material", "missing material"))?;
            materials.push(builder.add_material(semantic::Material {
                name: Cow::Borrowed(""),
                surface: profile::Material {
                    flags: profile::values::MaterialFlags::from_bits_retain(native.flags.bits()),
                    blend: profile::values::BlendMode::from_raw(native.blending_mode.raw()),
                    ..Default::default()
                },
            })?)
        }
        let tail = D::Layout::ribbon_tail(&value.modern);
        let feature = builder.reserve_feature(node);
        let mut ribbon = profile::Ribbon {
            id: value.id,
            textures: value
                .texture_indices
                .iter()
                .map(|i| {
                    textures
                        .get(i as usize)
                        .copied()
                        .ok_or_else(|| Error::new("m2.ribbon.texture", "missing texture"))
                })
                .collect::<Result<_, _>>()?,
            materials,
            color: Animated::constant(initial(&value.color).unwrap_or([1.; 3])),
            alpha: Animated::constant(f32::from(initial(&value.alpha).unwrap_or(32767)) / 32767.),
            height_above: Animated::constant(initial(&value.height_above).unwrap_or(0.)),
            height_below: Animated::constant(initial(&value.height_below).unwrap_or(0.)),
            edges_per_second: value.edges_per_second,
            edge_lifetime: value.edge_lifetime,
            emission_angle: value.gravity,
            atlas: [value.texture_rows, value.texture_columns],
            texture_slot: Animated::constant(initial(&value.texture_slot).unwrap_or(0)),
            enabled: Animated::constant(initial(&value.visibility).unwrap_or(1) != 0),
            priority_plane: tail[0],
            color_index: tail[1] as i8,
            texture_transform: None,
        };
        bind_field!(feature, ribbon, color, &value.color, Vector3, |v| v);
        bind_field!(feature, ribbon, alpha, &value.alpha, Scalar, |v| f32::from(
            v
        ) / 32767.);
        bind_field!(
            feature,
            ribbon,
            height_above,
            &value.height_above,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            ribbon,
            height_below,
            &value.height_below,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            ribbon,
            texture_slot,
            &value.texture_slot,
            Unsigned16,
            |v| v
        );
        bind_field!(feature, ribbon, enabled, &value.visibility, Boolean, |v| v
            != 0);
        builder.set_feature(feature, profile::ActorKind::Ribbon(ribbon))?;
    }
    for (index, value) in core.particles.iter().enumerate() {
        let node = placement(
            builder,
            nodes,
            Some(value.bone_id as usize),
            value.position,
            format!("particle-{index}"),
        )?;
        let feature = builder.reserve_feature(node);
        let mut particle = D::Layout::logical_particle(value, textures)?;
        bind_field!(
            feature,
            particle,
            speed,
            &value.emission_speed,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            speed_variation,
            &value.speed_variation,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            vertical_range,
            &value.vertical_range,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            horizontal_range,
            &value.horizontal_range,
            Scalar,
            |v| v
        );
        bind_field!(feature, particle, lifespan, &value.lifespan, Scalar, |v| v);
        bind_field!(feature, particle, rate, &value.emission_rate, Scalar, |v| v);
        bind_field!(
            feature,
            particle,
            area_width,
            &value.emission_area_width,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            area_length,
            &value.emission_area_length,
            Scalar,
            |v| v
        );
        bind_field!(feature, particle, z_source, &value.z_source, Scalar, |v| v);
        bind_field!(feature, particle, enabled, &value.enabled, Boolean, |v| v
            != 0);
        let packed = value.flags.packed_gravity();
        bind_field!(
            feature,
            particle,
            gravity,
            &value.gravity,
            Vector3,
            move |v| { gravity_vector(v, packed).unwrap_or([f32::NAN; 3]) }
        );
        builder.set_feature(feature, profile::ActorKind::Particle(Box::new(particle)))?;
    }
    for (index, value) in core.cameras.iter().enumerate() {
        let node = placement(
            builder,
            nodes,
            None,
            value.position_base,
            format!("camera-{index}"),
        )?;
        let feature = builder.reserve_feature(node);
        let mut camera = profile::Camera {
            kind: profile::values::CameraKind::from_raw(value.kind.raw()),
            field_of_view: D::Layout::logical_fov(source, value, feature, clips, clocks, builder)?,
            clip: [value.near_clip, value.far_clip],
            target_base: value.target_position_base,
            position: Animated::constant([0.; 3]),
            target: Animated::constant([0.; 3]),
            roll: Animated::constant(0.),
        };
        let position = Property::named(OwnerId::Feature(feature), "m2.camera.position");
        let target = Property::named(OwnerId::Feature(feature), "m2.camera.target");
        let roll = Property::named(OwnerId::Feature(feature), "m2.camera.roll");
        camera.position.property = Some(position.clone());
        camera.target.property = Some(target.clone());
        camera.roll.property = Some(roll.clone());
        for (seq, clip) in animation::sequences(clips) {
            let (start, _) = tables
                .sequences
                .get(seq)
                .map_or((0, 0), |s| s.timing.window());
            for (property, track) in [
                (position.clone(), &value.positions),
                (target.clone(), &value.target_positions),
            ] {
                if let Some((c, g)) = animation::spline_curve(source, track, seq, start)? {
                    animation::insert(
                        builder,
                        clip,
                        Channel::Vector3(bind(c, g, property, clocks)?),
                    )?;
                }
            }
            if let Some((c, g)) = animation::spline_curve(source, &value.roll, seq, start)? {
                animation::insert(
                    builder,
                    clip,
                    Channel::Scalar(bind(c, g, roll.clone(), clocks)?),
                )?;
            }
        }
        builder.set_feature(feature, profile::ActorKind::Camera(camera))?;
    }
    for (index, value) in core.events.iter().enumerate() {
        let node = placement(
            builder,
            nodes,
            Some(value.bone_id as usize),
            value.position,
            format!("event-{index}"),
        )?;
        let feature = builder.add_feature(
            node,
            profile::ActorKind::Event(profile::Event {
                identifier: value.identifier,
                data: value.data,
            }),
        )?;
        let global = value.enabled.global_sequence;
        let count = if global != u16::MAX { 1 } else { clips.len() };
        for (seq, clip) in (0..count).map(|seq| (seq, clips.get(seq).copied())) {
            let leaf = if global != u16::MAX || D::VERSION < 264 {
                0
            } else {
                seq
            };
            let Some(times) = value.enabled.timeline.leaf(leaf) else {
                continue;
            };
            let times = animation::resolve(source, times, seq)?;
            let range = if global == u16::MAX {
                value
                    .enabled
                    .timeline
                    .ranges()
                    .get(seq)
                    .map(|[a, b]| a as usize..b as usize + 1)
            } else {
                None
            };
            let start = if global == u16::MAX {
                tables
                    .sequences
                    .get(seq)
                    .map_or((0, 0), |s| s.timing.window())
                    .0
            } else {
                0
            };
            if global == u16::MAX
                && builder
                    .model()
                    .clip(clip.ok_or_else(|| Error::new("m2.event.clip", "sequence has no clip"))?)
                    .is_some_and(|c| c.alias().is_some())
            {
                continue;
            }
            for (index, ms) in times.iter().enumerate() {
                if range.as_ref().is_some_and(|r| !r.contains(&index)) {
                    continue;
                }
                let event = semantic::Event {
                    time: Time::seconds((f64::from(ms) - f64::from(start)) / 1000.)?,
                    clock: if global == u16::MAX {
                        semantic::Clock::Clip
                    } else {
                        semantic::Clock::Shared(
                            *clocks
                                .get(global as usize)
                                .ok_or_else(|| Error::new("m2.event.clock", "missing clock"))?,
                        )
                    },
                    name: Cow::Owned(String::from_utf8_lossy(&value.identifier).into_owned()),
                    node: Some(node),
                    feature: Some(feature),
                    payload: Values::default(),
                };
                if global == u16::MAX {
                    builder.add_event(
                        clip.ok_or_else(|| Error::new("m2.event.clip", "sequence has no clip"))?,
                        event,
                    )?
                } else {
                    builder.add_shared_event(event)?
                }
            }
        }
    }
    Ok(())
}
