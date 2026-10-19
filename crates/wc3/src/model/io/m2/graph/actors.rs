//! Native feature nodes imported directly into independent scene state.
use super::*;
use crate::model::materials::m2::rasterization;
use crate::model::materials::state::PassState;
use crate::model::LightEmission;
use crate::model::{Axis, M2RibbonControls, M2RibbonRendering, TrailAppearance, TrailGeometry};
use glam::{Vec2, Vec3};
fn placement<'a>(
    builder: &mut GraphBuilder<'a>,
    nodes: &[semantic::NodeId],
    bone: Option<usize>,
    position: [f32; 3],
    name: String,
) -> Result<semantic::NodeId, Error> {
    let mut node = builder.new_node(name);
    builder
        .values_mut()
        .set(node.transform.translation, Vec3::from_array(position))?;
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
    textures: &[semantic::Texture],
    animation_source: &AnimationResolver<'a, 'data, D>,
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
            let property = $field;
            for (seq, clip) in animation::sequences(clips) {
                let (start, _) = tables
                    .sequences
                    .get(seq)
                    .map_or((0, 0), |s| s.timing.window());
                if let Some((c, g)) = curve(
                    animation_source,
                    $native,
                    &source.key,
                    seq,
                    start,
                    move |v| ($map)(v).into(),
                )? {
                    animation::insert(
                        builder,
                        clip,
                        DynamicTrack::$variant(bind(c, g, property.clone(), clocks)?),
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
        let attachment = profile::Attachment {
            identity: value.id,
            control: (),
            visibility: builder.add_value(initial(&value.animate).unwrap_or(1) != 0),
        };
        let property = attachment.visibility;
        for (sequence, clip) in animation::sequences(clips) {
            let (start, _) = attachment_tables
                .sequences
                .get(sequence)
                .map_or((0, 0), |sequence| sequence.timing.window());
            if let Some((curve, global)) = animation::curve_in_stream(
                animation_source,
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
                    DynamicTrack::Boolean(bind(curve, global, property, clocks)?),
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
        let light = profile::Light {
            kind: profile::values::LightKind::from_raw(value.kind.raw()),
            ambient: LightEmission {
                color: builder.add_value(Vec3::from_array(
                    initial(&value.ambient_color).unwrap_or([0.; 3]),
                )),
                intensity: builder.add_value(initial(&value.ambient_intensity).unwrap_or(0.)),
            },
            diffuse: LightEmission {
                color: builder.add_value(Vec3::from_array(
                    initial(&value.diffuse_color).unwrap_or([0.; 3]),
                )),
                intensity: builder.add_value(initial(&value.diffuse_intensity).unwrap_or(0.)),
            },
            attenuation: [
                builder.add_value(initial(&value.attenuation_start).unwrap_or(0.)),
                builder.add_value(initial(&value.attenuation_end).unwrap_or(0.)),
            ],
            activation: builder.add_value(initial(&value.visibility).unwrap_or(1) != 0),
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
            materials.push({
                let definition = semantic::Material {
                    name: Cow::Borrowed(""),
                    surface: profile::MaterialStep::from_native(
                        0,
                        profile::MaterialDeclaration {
                            state: PassState {
                                blend: profile::values::BlendMode::from_raw(
                                    native.blending_mode.raw(),
                                ),
                                rasterization: rasterization(
                                    profile::values::MaterialFlags::from_bits_retain(
                                        native.flags.bits(),
                                    ),
                                ),
                                ordering: 0,
                            },
                            ..builder.construct()
                        },
                        Vec::new(),
                        Default::default(),
                    )
                    .into(),
                };
                builder.add_material(definition)
            }?)
        }
        let tail = D::Layout::ribbon_tail(&value.modern);
        let feature = builder.reserve_feature(node);
        let ribbon = profile::RibbonEmitter {
            controls: M2RibbonControls {
                id: value.id,
                angle: value.gravity,
                enabled: builder.add_value(initial(&value.visibility).unwrap_or(1) != 0),
                priority_plane: tail[0],
                color_index: tail[1] as i8,
                texture_transform: None,
            },
            rendering: M2RibbonRendering {
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
                color: builder
                    .add_value(Vec3::from_array(initial(&value.color).unwrap_or([1.; 3]))),
                opacity: builder
                    .add_value(f32::from(initial(&value.alpha).unwrap_or(32767)) / 32767.),
                atlas: [value.texture_rows, value.texture_columns],
                texture_slot: builder.add_value(initial(&value.texture_slot).unwrap_or(0)),
            },
            geometry: TrailGeometry {
                axis: Axis::PositiveZ,
                height_above: builder.add_value(initial(&value.height_above).unwrap_or(0.)),
                height_below: builder.add_value(initial(&value.height_below).unwrap_or(0.)),
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
        let camera = profile::Camera {
            convention: profile::values::CameraKind::from_raw(value.kind.raw()),
            field_of_view: D::Layout::logical_fov(
                source,
                value,
                feature,
                animation_source,
                clips,
                clocks,
                builder,
            )?,
            clipping: Vec2::from_array([value.near_clip, value.far_clip]),
            target: Vec3::from_array(value.target_position_base),
            eye_offset: builder.add_value(Vec3::from_array([0.; 3])),
            target_offset: builder.add_value(Vec3::from_array([0.; 3])),
            roll: builder.add_value(0.),
        };
        let position = camera.eye_offset;
        let target = camera.target_offset;
        let roll = camera.roll;
        for (seq, clip) in animation::sequences(clips) {
            let (start, _) = tables
                .sequences
                .get(seq)
                .map_or((0, 0), |s| s.timing.window());
            for (property, track) in [
                (position, &value.positions),
                (target, &value.target_positions),
            ] {
                if let Some((c, g)) =
                    animation::spline_curve(animation_source, track, seq, start, Vec3::from_array)?
                {
                    animation::insert(
                        builder,
                        clip,
                        DynamicTrack::Vector3(bind(c, g, property, clocks)?),
                    )?;
                }
            }
            if let Some((c, g)) =
                animation::spline_curve(animation_source, &value.roll, seq, start, |v| v)?
            {
                animation::insert(
                    builder,
                    clip,
                    DynamicTrack::Scalar(bind(c, g, roll, clocks)?),
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
        let global = value.enabled.global_sequence;
        let clock = if global == u16::MAX {
            semantic::TimeSource::Animation
        } else {
            semantic::TimeSource::Shared(
                *clocks
                    .get(global as usize)
                    .ok_or_else(|| Error::new("m2.event.clock", "missing clock"))?,
            )
        };
        let event_source = builder.add_event_source(profile::EventSource {
            node,
            clock,
            identifier: value.identifier,
            data: value.data,
        });
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
            let times = animation_source
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
                    .is_some_and(|c| c.controls.alias.is_some())
            {
                continue;
            }
            for (index, ms) in times.iter().enumerate() {
                if range.as_ref().is_some_and(|r| !r.contains(&index)) {
                    continue;
                }
                let event = semantic::EventOccurrence {
                    time: Time::seconds((f64::from(ms) - f64::from(start)) / 1000.)?,
                    source: event_source,
                };
                if global == u16::MAX {
                    builder.add_event(
                        clip.ok_or_else(|| Error::new("m2.event.clip", "sequence has no clip"))?,
                        event,
                    )?
                } else {
                    builder.add_global_event(event)?;
                }
            }
        }
    }
    Ok(())
}
