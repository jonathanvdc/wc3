//! Native feature nodes imported directly into independent scene state.
use super::*;
use crate::model::materials::model_shader::rasterization;
use crate::model::materials::state::PassState;
use crate::model::LightEmission;
use crate::model::{
    AngleTrailControls, Axis, SurfaceTrailRendering, TrailAppearance, TrailGeometry,
};
use zerovec::ZeroVec;
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
        ($node:ident,$actor:ident,$field:expr,$key:ident,$native:expr,$variant:ident,$map:expr) => {{
            let property = profile::properties::$actor::$key.bind($node);
            $field.binding = Some(property.clone().widen().into());
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
    let (attachment_owner, attachments) = source.attachments().map_err(failure)?;
    let attachment_tables = sequence_tables(source, attachment_owner)?;
    for (index, value) in attachments.iter().enumerate() {
        let node = placement(
            builder,
            nodes,
            Some(value.bone_id as usize),
            value.position,
            format!("attachment-{index}"),
        )?;
        let feature = builder.reserve_feature(node);
        let mut attachment = profile::Attachment {
            identity: value.id,
            control: (),
            visibility: Animated::constant(initial(&value.animate).unwrap_or(1) != 0),
        };
        let property = profile::properties::attachment::ENABLED.bind(feature);
        attachment.visibility.binding = Some(property.clone().widen());
        for (sequence, clip) in animation::sequences(clips) {
            let (start, _) = attachment_tables
                .sequences
                .get(sequence)
                .map_or((0, 0), |sequence| sequence.timing.window());
            if let Some((curve, global)) = animation::curve_in_stream(
                source,
                &value.animate,
                attachment_owner,
                if attachment_owner == &source.key {
                    *b"AFM2"
                } else {
                    *b"AFSA"
                },
                sequence,
                start,
                |value| value != 0,
            )? {
                animation::insert(
                    builder,
                    clip,
                    Channel::Boolean(bind(curve, global, property.clone(), clocks)?),
                )?;
            }
        }
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
            ambient: LightEmission {
                color: Animated::constant(initial(&value.ambient_color).unwrap_or([0.; 3])),
                intensity: Animated::constant(initial(&value.ambient_intensity).unwrap_or(0.)),
            },
            diffuse: LightEmission {
                color: Animated::constant(initial(&value.diffuse_color).unwrap_or([0.; 3])),
                intensity: Animated::constant(initial(&value.diffuse_intensity).unwrap_or(0.)),
            },
            attenuation: [
                Animated::constant(initial(&value.attenuation_start).unwrap_or(0.)),
                Animated::constant(initial(&value.attenuation_end).unwrap_or(0.)),
            ],
            activation: Animated::constant(initial(&value.visibility).unwrap_or(1) != 0),
        };
        bind_field!(
            feature,
            light,
            light.ambient.color,
            AMBIENT_COLOR,
            &value.ambient_color,
            Vector3,
            |v| v
        );
        bind_field!(
            feature,
            light,
            light.ambient.intensity,
            AMBIENT_INTENSITY,
            &value.ambient_intensity,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            light,
            light.diffuse.color,
            DIFFUSE_COLOR,
            &value.diffuse_color,
            Vector3,
            |v| v
        );
        bind_field!(
            feature,
            light,
            light.diffuse.intensity,
            DIFFUSE_INTENSITY,
            &value.diffuse_intensity,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            light,
            light.attenuation[0],
            ATTENUATION_START,
            &value.attenuation_start,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            light,
            light.attenuation[1],
            ATTENUATION_END,
            &value.attenuation_end,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            light,
            light.activation,
            ENABLED,
            &value.visibility,
            Boolean,
            |v| v != 0
        );
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
                    state: PassState {
                        blend: profile::values::BlendMode::from_raw(native.blending_mode.raw()),
                        rasterization: rasterization(
                            profile::values::MaterialFlags::from_bits_retain(native.flags.bits()),
                        ),
                        ordering: 0,
                    },
                    ..Default::default()
                },
            })?)
        }
        let tail = D::Layout::ribbon_tail(&value.modern);
        let feature = builder.reserve_feature(node);
        let mut ribbon = profile::Ribbon {
            controls: AngleTrailControls {
                id: value.id,
                angle: value.gravity,
                enabled: Animated::constant(initial(&value.visibility).unwrap_or(1) != 0),
                priority_plane: tail[0],
                color_index: tail[1] as i8,
                texture_transform: None,
            },
            rendering: SurfaceTrailRendering {
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
            },
            appearance: TrailAppearance {
                color: Animated::constant(initial(&value.color).unwrap_or([1.; 3])),
                opacity: Animated::constant(
                    f32::from(initial(&value.alpha).unwrap_or(32767)) / 32767.,
                ),
                atlas: [value.texture_rows, value.texture_columns],
                texture_slot: Animated::constant(initial(&value.texture_slot).unwrap_or(0)),
            },
            geometry: TrailGeometry {
                axis: Axis::PositiveZ,
                height_above: Animated::constant(initial(&value.height_above).unwrap_or(0.)),
                height_below: Animated::constant(initial(&value.height_below).unwrap_or(0.)),
            },
            emission_rate: value.edges_per_second,
            lifetime: value.edge_lifetime,
        };
        bind_field!(
            feature,
            ribbon,
            ribbon.appearance.color,
            COLOR,
            &value.color,
            Vector3,
            |v| v
        );
        bind_field!(
            feature,
            ribbon,
            ribbon.appearance.opacity,
            ALPHA,
            &value.alpha,
            Scalar,
            |v| f32::from(v) / 32767.
        );
        bind_field!(
            feature,
            ribbon,
            ribbon.geometry.height_above,
            HEIGHT_ABOVE,
            &value.height_above,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            ribbon,
            ribbon.geometry.height_below,
            HEIGHT_BELOW,
            &value.height_below,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            ribbon,
            ribbon.appearance.texture_slot,
            TEXTURE_SLOT,
            &value.texture_slot,
            Unsigned16,
            |v| v
        );
        bind_field!(
            feature,
            ribbon,
            ribbon.controls.enabled,
            ENABLED,
            &value.visibility,
            Boolean,
            |v| v != 0
        );
        builder.set_feature(feature, profile::ActorKind::Ribbon(Box::new(ribbon)))?;
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
        let mut particle = D::Layout::logical_particle(value, textures, builder)?;
        particle.details.rendering.texture_program = source.model.extensions().find_map(|chunk| {
            if let Chunk::TextureCombinerHints(programs) = chunk {
                programs.get(core.materials.len() + index)
            } else {
                None
            }
        });
        bind_field!(
            feature,
            particle,
            particle.motion.speed,
            SPEED,
            &value.emission_speed,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            particle.motion.variation,
            SPEED_VARIATION,
            &value.speed_variation,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            particle.details.shape.vertical_range,
            VERTICAL_RANGE,
            &value.vertical_range,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            particle.details.shape.horizontal_range,
            HORIZONTAL_RANGE,
            &value.horizontal_range,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            particle.lifetime.duration,
            LIFESPAN,
            &value.lifespan,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            particle.emission.rate,
            RATE,
            &value.emission_rate,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            particle.details.shape.area_width,
            AREA_WIDTH,
            &value.emission_area_width,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            particle.details.shape.area_length,
            AREA_LENGTH,
            &value.emission_area_length,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            particle.details.shape.z_source,
            Z_SOURCE,
            &value.z_source,
            Scalar,
            |v| v
        );
        bind_field!(
            feature,
            particle,
            particle.emission.activation,
            ENABLED,
            &value.enabled,
            Boolean,
            |v| v != 0
        );
        let packed = value.flags.packed_gravity();
        bind_field!(
            feature,
            particle,
            particle.motion.gravity,
            GRAVITY,
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
            convention: profile::values::CameraKind::from_raw(value.kind.raw()),
            field_of_view: D::Layout::logical_fov(source, value, feature, clips, clocks, builder)?,
            clipping: [value.near_clip, value.far_clip],
            target: value.target_position_base,
            eye_offset: Animated::constant([0.; 3]),
            target_offset: Animated::constant([0.; 3]),
            roll: Animated::constant(0.),
        };
        let position = profile::properties::camera::POSITION.bind(feature);
        let target = profile::properties::camera::TARGET.bind(feature);
        let roll = profile::properties::camera::ROLL.bind(feature);
        camera.eye_offset.binding = Some(position.clone().widen());
        camera.target_offset.binding = Some(target.clone().widen());
        camera.roll.binding = Some(roll.clone().widen().into());
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
            let times = source
                .resolve_animation_array(times, &source.key, *b"AFM2", seq)
                .map_err(failure)?;
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
                    payload: ZeroVec::new(),
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
