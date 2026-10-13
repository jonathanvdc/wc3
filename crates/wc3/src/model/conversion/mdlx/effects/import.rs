//! Import mappings for effects.
use super::*;
pub(in crate::model::conversion::mdlx) fn decorate_native<D: ModelDialect>(
    document: wem::Document<'static>,
    source: &native::Model<'_, D>,
) -> Result<wem::Document<'static>, Error> {
    let mut raw = document.into_raw();
    let root: wr::DocumentRecord<wr::V1> = raw
        .resolve::<wr::DocumentRecord<wr::V1>>(Ref::new(
            raw.root_reference().entries,
            raw.root_reference().index,
            raw.root_reference().flags,
        ))?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("wem.root", "missing root"))?;
    let model: wr::ModelRecord<wr::V9> = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("wem.model", "missing model"))?;
    let profile = raw
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("wem.profile", "missing profile"))?;
    let mut materials = raw
        .resolve::<wr::MaterialRecord>(profile.materials.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    for (index, source_material) in source.materials().enumerate() {
        let shader = append_string(&mut raw, &source_material.try_shader().unwrap_or_default())?;
        let native_material = wr::MdxMaterial {
            source_version: D::Version::NUMBER,
            priority_plane: source_material.priority_plane,
            flags: wr::MdxMaterialFlag(source_material.render_mode.bits()),
            shader,
            layers: wem::InlineList(native_layers(source_material)),
        };
        materials[index].native = wr::NativeMaterial::Mdx(append(&mut raw, &[native_material])?);
        materials[index].sync = wr::NativeSync::NATIVE_AUTHORITATIVE;
    }
    let materials_id = raw
        .section_id(profile.materials.index)
        .ok_or_else(|| Error::new("wem.materials", "missing section"))?;
    if !materials.is_empty() {
        raw.replace(materials_id, &materials)?;
    }
    let mut nodes = raw
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let source_nodes = source.nodes();
    let identities: BTreeMap<_, _> = source_nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.object_id, index))
        .collect();
    for (index, node) in source_nodes.iter().enumerate() {
        let entries = [wr::NativeEntry::<wr::V2> {
            name: append_string(&mut raw, "mdlx.object_id")?,
            value: i64::from(node.object_id),
            text: Ref::default(),
        }];
        nodes[index].native.entries = append(&mut raw, &entries)?;
        metadata::set_metadata(
            &mut raw,
            &mut nodes[index].native,
            "mdlx.node_flags",
            i64::from(node.flags.bits()),
            "",
        )?;
    }
    for chunk in &source.chunks {
        match chunk {
            ModelChunk::ParticleEmitters(chunk) => {
                for record in &chunk.records {
                    let index = identities[&record.node.object_id];
                    nodes[index].kind = wr::NodeKind::WC3_PARTICLE_EMITTER1;
                    nodes[index].payload = wr::NodePayload::Wc3ParticleEmitter1(Box::new(
                        wr::Wc3ParticleEmitter1Payload {
                            emission_rate: base(&record.emission_rate),
                            gravity: base(&record.gravity),
                            longitude: base(&record.longitude),
                            latitude: base(&record.latitude),
                            lifespan: base(&record.life_span),
                            speed: base(&record.initial_velocity),
                            spawn_model: wr::AssetKey {
                                group: 0,
                                id: u32::MAX,
                                path: append_string(&mut raw, &text(&record.path))?,
                            },
                            uses_mdl: wem::Bool::from(record.node.flags.bits() & (1 << 15) != 0),
                            uses_tga: wem::Bool::from(record.node.flags.bits() & (1 << 16) != 0),
                        },
                    ));
                }
            }
            ModelChunk::ParticleEmitters2(chunk) => {
                for record in &chunk.records {
                    let index = identities[&record.node.object_id];
                    let segments: [wr::Wc3ParticleSegment; 3] =
                        [0, 1, 2].map(|i| wr::Wc3ParticleSegment {
                            color: record.segment_colors[i],
                            alpha: record.alpha[i],
                            scaling: record.particle_scaling[i],
                        });
                    let intervals: [wr::Wc3ParticleInterval; 4] =
                        record.uv_animations.map(|v| wr::Wc3ParticleInterval {
                            start: v[0],
                            end: v[1],
                            repeat: v[2],
                        });
                    nodes[index].kind = wr::NodeKind::WC3_PARTICLE_EMITTER2;
                    nodes[index].payload = wr::NodePayload::Wc3ParticleEmitter2(Box::new(
                        wr::Wc3ParticleEmitter2Payload {
                            speed: base(&record.speed),
                            variation: base(&record.variation),
                            latitude: base(&record.latitude),
                            gravity: base(&record.gravity),
                            lifespan: record.life_span,
                            emission_rate: base(&record.emission_rate),
                            width: base(&record.width),
                            length: base(&record.length),
                            filter: wr::Wc3ParticleFilter(particle_filter(record.filter_mode)),
                            rows: record.rows,
                            columns: record.columns,
                            head_or_tail: wr::Wc3ParticleHeadOrTail(match record.frames {
                                native::Particle2Frames::Head => 0,
                                native::Particle2Frames::Tail => 1,
                                native::Particle2Frames::Both => 2,
                                native::Particle2Frames::Unknown(value) => value,
                            }),
                            tail_length: record.tail_length,
                            time: record.time,
                            start: segments[0].clone(),
                            middle: segments[1].clone(),
                            end: segments[2].clone(),
                            head_life: intervals[0].clone(),
                            head_decay: intervals[1].clone(),
                            tail_life: intervals[2].clone(),
                            tail_decay: intervals[3].clone(),
                            texture: record.texture_id,
                            replaceable_id: record.replaceable_id,
                            squirt: wem::Bool::from(record.squirt != 0),
                            priority_plane: record.priority_plane as i32,
                            unshaded: wem::Bool::from(record.node.flags.unshaded()),
                            sort_prims_far_z: wem::Bool::from(record.node.flags.sort_prims_far_z()),
                            line_emitter: wem::Bool::from(record.node.flags.line_emitter()),
                            unfogged: wem::Bool::from(record.node.flags.unfogged()),
                            xy_quad: wem::Bool::from(record.node.flags.xy_quad()),
                        },
                    ));
                }
            }
            ModelChunk::RibbonEmitters(chunk) => {
                for record in &chunk.records {
                    let index = identities[&record.node.object_id];
                    nodes[index].kind = wr::NodeKind::WC3_RIBBON_EMITTER;
                    nodes[index].payload =
                        wr::NodePayload::Wc3RibbonEmitter(Box::new(wr::Wc3RibbonEmitterPayload {
                            height_above: base(&record.height_above),
                            height_below: base(&record.height_below),
                            alpha: base(&record.alpha),
                            color: base(&record.color),
                            lifespan: record.life_span,
                            texture_slot: base(&record.texture_slot),
                            emission_rate: record.emission_rate,
                            rows: record.rows,
                            columns: record.columns,
                            material_slot: record.material_id,
                            gravity: record.gravity,
                        }));
                }
            }
            ModelChunk::PopcornEmitters(chunk) => {
                for record in &chunk.records {
                    let index = identities[&record.node.object_id];
                    nodes[index].kind = wr::NodeKind::WC3_CORN_EMITTER;
                    nodes[index].payload =
                        wr::NodePayload::Wc3CornEmitter(Box::new(wr::Wc3CornEmitterPayload {
                            lifespan: base(&record.life_span),
                            emission_rate: base(&record.emission_rate),
                            speed: base(&record.speed),
                            color: base(&record.color),
                            alpha: base(&record.alpha),
                            replaceable_id: record.replaceable_id,
                            effect: wr::AssetKey {
                                group: 0,
                                id: u32::MAX,
                                path: append_string(&mut raw, &text(&record.path))?,
                            },
                            anim_visibility_guide: append_string(
                                &mut raw,
                                &text(&record.visibility_guide),
                            )?,
                            unshaded: wem::Bool::from(record.node.flags.unshaded()),
                            sort_prims_far_z: wem::Bool::from(record.node.flags.sort_prims_far_z()),
                            unfogged: wem::Bool::from(record.node.flags.unfogged()),
                            popcorn_scaling: wem::Bool::from(record.node.flags.popcorn_scaling()),
                        }));
                }
            }
            ModelChunk::Attachments(chunk) => {
                for record in &chunk.records {
                    let index = identities[&record.node.object_id];
                    nodes[index].kind = wr::NodeKind::ATTACHMENT;
                    nodes[index].payload =
                        wr::NodePayload::Attachment(Box::new(wr::AttachmentPayload {
                            model: u32::MAX,
                            asset: wr::AssetKey {
                                group: 0,
                                id: u32::MAX,
                                path: append_string(&mut raw, &text(&record.path))?,
                            },
                        }));
                }
            }
            ModelChunk::Lights(chunk) => {
                for record in &chunk.records {
                    let index = identities[&record.node.object_id];
                    nodes[index].kind = wr::NodeKind::LIGHT;
                    nodes[index].payload =
                        wr::NodePayload::Light(Box::new(wr::LightPayload::<wr::V19> {
                            kind: wr::LightKind(match record.light_type {
                                native::scene::LightType::Omnidirectional => 0,
                                native::scene::LightType::Directional => 1,
                                native::scene::LightType::Ambient => 2,
                                native::scene::LightType::Unknown(value) => u8::try_from(value)
                                    .map_err(|_| {
                                        Error::new(
                                            "lights.kind",
                                            "light kind exceeds WEM byte representation",
                                        )
                                    })?,
                            }),
                            color: base(&record.color),
                            intensity: base(&record.intensity),
                            attenuation_start: base(&record.attenuation_start),
                            attenuation_end: base(&record.attenuation_end),
                            ..Default::default()
                        }));
                }
            }
            ModelChunk::CollisionShapes(chunk) => {
                for record in &chunk.records {
                    let index = identities[&record.node.object_id];
                    let mut shape = wr::CollisionShapeDesc::default();
                    match record.geometry {
                        native::CollisionGeometry::Box(points) => {
                            shape.kind = wr::CollisionShapeKind::BOX;
                            shape.bounds.minimum = points[0];
                            shape.bounds.maximum = points[1];
                        }
                        native::CollisionGeometry::Plane(points) => {
                            shape.kind = wr::CollisionShapeKind::PLANE;
                            shape.bounds.minimum = points[0];
                            shape.bounds.maximum = points[1];
                        }
                        native::CollisionGeometry::Sphere(center, radius) => {
                            shape.kind = wr::CollisionShapeKind::SPHERE;
                            shape.sphere = wr::Sphere { center, radius };
                        }
                        native::CollisionGeometry::Cylinder(points, radius) => {
                            shape.kind = wr::CollisionShapeKind::CYLINDER;
                            shape.bounds.minimum = points[0];
                            shape.bounds.maximum = points[1];
                            shape.sphere.radius = radius;
                        }
                    }
                    nodes[index].kind = wr::NodeKind::COLLISION_SHAPE;
                    nodes[index].payload =
                        wr::NodePayload::Collision(Box::new(wr::CollisionPayload { shape }));
                }
            }
            ModelChunk::EventObjects(chunk) => {
                for record in &chunk.records {
                    let index = identities[&record.node.object_id];
                    nodes[index].kind = wr::NodeKind::EVENT;
                    nodes[index].payload =
                        wr::NodePayload::Event(Box::new(wr::EventPayload::<wr::V19> {
                            id: record.node.object_id,
                            code: append_string(&mut raw, &text(&record.node.name))?,
                            ..Default::default()
                        }));
                    let mut entries = raw
                        .resolve::<wr::NativeEntry<wr::V2>>(nodes[index].native.entries.cast())?
                        .iter()
                        .collect::<Result<Vec<_>, _>>()?;
                    entries.push(wr::NativeEntry::<wr::V2> {
                        name: append_string(&mut raw, "mdlx.event.global_sequence")?,
                        value: i64::from(record.global_sequence_id),
                        text: Ref::default(),
                    });
                    entries.push(wr::NativeEntry::<wr::V2> {
                        name: append_string(&mut raw, "mdlx.event.frames")?,
                        value: 0,
                        text: append_string(
                            &mut raw,
                            &record
                                .frames()
                                .iter()
                                .map(|frame| frame.to_string())
                                .collect::<Vec<_>>()
                                .join(","),
                        )?,
                    });
                    nodes[index].native.entries = append(&mut raw, &entries)?;
                }
            }
            _ => {}
        }
    }
    for (index, camera) in source.cameras().enumerate() {
        let node = &mut nodes[source_nodes.len() + index];
        node.kind = wr::NodeKind::CAMERA;
        node.payload = wr::NodePayload::Camera(Box::new(wr::CameraPayload::<wr::V19> {
            fov: camera.field_of_view,
            near_clip: camera.near_clip,
            far_clip: camera.far_clip,
            target: camera.target_position,
            ..Default::default()
        }));
        let entries = [wr::NativeEntry::<wr::V2> {
            name: append_string(&mut raw, "mdlx.camera.index")?,
            value: index as i64,
            text: Ref::default(),
        }];
        node.native.entries = append(&mut raw, &entries)?;
    }
    if let Ok(entries) = source.try_face_fx() {
        let start = source_nodes.len() + source.cameras().count();
        for (index, entry) in entries.enumerate() {
            nodes[start + index].kind = wr::NodeKind::WC3_FACE_FX;
            nodes[start + index].payload =
                wr::NodePayload::Wc3FaceFx(Box::new(wr::Wc3FaceFxPayload {
                    actor: wr::AssetKey {
                        group: 0,
                        id: u32::MAX,
                        path: append_string(&mut raw, &text(&entry.path))?,
                    },
                }));
        }
    }
    let nodes_id = raw
        .section_id(model.nodes.nodes.index)
        .ok_or_else(|| Error::new("wem.nodes", "missing section"))?;
    if !nodes.is_empty() {
        raw.replace(nodes_id, &nodes)?;
    }
    Ok(wem::Document::from_raw(raw)?)
}

pub(in crate::model::conversion::mdlx) fn particle_filter(
    value: native::Particle2FilterMode,
) -> u32 {
    match value {
        native::Particle2FilterMode::Blend => 0,
        native::Particle2FilterMode::Additive => 1,
        native::Particle2FilterMode::Modulate => 2,
        native::Particle2FilterMode::Modulate2x => 3,
        native::Particle2FilterMode::AlphaKey => 4,
        native::Particle2FilterMode::Unknown(value) => value,
    }
}
