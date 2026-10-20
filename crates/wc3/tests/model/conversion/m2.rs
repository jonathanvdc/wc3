use glam::Vec3;
use std::borrow::Cow;
use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::io;
use wc3::model::IntoPackedArray;
use wc3::model::ModelMetadata;
use wc3::model::TimeSource;
use wc3::model::{
    self as sm,
    conversion::{
        self, ApproximationPolicy, ExportOptions, ImportOptions, LossPolicy, UnknownPolicy,
    },
    formats::m2::{
        self,
        companions::SkinProfile,
        records::{
            Batch, BlendMode, Bone, Interpolation, LegacyTimeline, LegacyValues, LightKind,
            Material, MaterialFlags, ModelFlags, PhysicsBodyKind, PhysicsGameFlags, PhysicsKind,
            PhysicsShapeKind, SequenceFlags, SkinSection, Texture, TextureFlags, TextureSource,
            Track, TrackBase, Vertex,
        },
    },
    profiles::m2 as mp,
};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

fn native_triangle() -> m2::Bundle<'static, m2::Flat<m2::V256>> {
    let mut model = m2::Document::<m2::Flat<m2::V256>>::new();
    let core = model.core_mut();
    core.name = ZeroVec::alloc_from_slice(b"triangle\0");
    core.bones = vec![Bone::new()];
    core.vertices = ZeroVec::alloc_from_slice(&[
        Vertex {
            position: [0., 0., 0.],
            normal: [0., 0., 1.],
            texture_coordinates: [[0., 0.], [0.25, 0.25]],
            bone_indices: [0; 4],
            bone_weights: [255, 0, 0, 0],
        },
        Vertex {
            position: [1., 0., 0.],
            normal: [0., 0., 1.],
            texture_coordinates: [[1., 0.], [0.75, 0.25]],
            bone_indices: [0; 4],
            bone_weights: [255, 0, 0, 0],
        },
        Vertex {
            position: [0., 1., 0.],
            normal: [0., 0., 1.],
            texture_coordinates: [[0., 1.], [0.25, 0.75]],
            bone_indices: [0; 4],
            bone_weights: [255, 0, 0, 0],
        },
    ]);
    core.textures = vec![Texture {
        filename: ZeroVec::alloc_from_slice(b"textures/checker.blp\0"),
        kind: TextureSource::File,
        flags: TextureFlags::from_bits_retain(3),
    }];
    core.materials = ZeroVec::alloc_from_slice(&[Material {
        flags: MaterialFlags::from_bits_retain(4),
        blending_mode: BlendMode::Alpha,
    }]);
    core.texture_combos = ZeroVec::alloc_from_slice(&[0]);
    core.texture_weight_combos = ZeroVec::alloc_from_slice(&[u16::MAX]);
    core.texture_transform_combos = ZeroVec::alloc_from_slice(&[u16::MAX]);
    core.texture_transform_bone_map = ZeroVec::alloc_from_slice(&[0]);
    core.skins = vec![SkinProfile {
        vertices: ZeroVec::alloc_from_slice(&[0, 1, 2]),
        indices: ZeroVec::alloc_from_slice(&[0, 1, 2]),
        bones: ZeroVec::alloc_from_slice(&[[0; 4]; 3]),
        sections: vec![SkinSection {
            vertex_count: 3,
            index_count: 3,
            id: 17,
            ..Default::default()
        }],
        batches: vec![Batch {
            color_index: -1,
            texture_count: 1,
            shader_id: 3,
            ..Default::default()
        }],
        ..Default::default()
    }];
    m2::Bundle::from_model(model)
}

fn particle_program_bundle(programs: &[[u8; 2]]) -> m2::Bundle<'static, m2::Chunked<m2::V274>> {
    let source = native_triangle();
    let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();
    let mut modern =
        io::m2::plan::<m2::Chunked<m2::V274>>(&imported.value, &ExportOptions::default())
            .unwrap()
            .execute()
            .value;
    for _ in 0..2 {
        let mut particle = m2::records::ParticleEmitter::<m2::V274>::new(ModelFlags::default());
        particle.flags.set_head(true);
        modern.model.core_mut().particles.push(particle);
    }
    modern
        .model
        .insert_extension(
            0,
            m2::chunks::Chunk::TextureCombinerHints(ZeroVec::alloc_from_slice(programs)),
        )
        .unwrap();
    modern
}

#[test]
fn particle_texture_programs_follow_the_material_prefix_and_round_trip() {
    let source = particle_program_bundle(&[[0, 0], [3, 11], [0, 0]]);
    let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();
    let programs: Vec<_> = imported
        .value
        .features()
        .iter()
        .filter_map(|(_, feature)| {
            if let mp::ActorKind::Particle(particle) = &feature.payload {
                Some(particle.details.rendering.texture_program)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(programs, vec![Some([3, 11]), Some([0, 0])]);
    for (_, feature) in imported.value.features() {
        if let mp::ActorKind::Particle(particle) = &feature.payload {
            let textures = imported.value.sampled_images();
            let options = ExportOptions::default();
            let error = <m2::V274 as io::m2::ParticleConversion>::write_logical_particle(
                particle,
                &textures,
                &mut conversion::Context::export(&options),
                &imported.value,
            )
            .unwrap_err();
            assert!(error.to_string().contains("standalone particle record"));
        }
    }
    let output = io::m2::plan::<m2::Chunked<m2::V274>>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let restored = output
        .model
        .extensions()
        .find_map(|chunk| {
            if let m2::chunks::Chunk::TextureCombinerHints(programs) = chunk {
                Some(programs.iter().collect::<Vec<_>>())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(restored, vec![[0, 0], [3, 11], [0, 0]]);
    assert!(
        io::m2::plan::<m2::Flat<m2::V274>>(&imported.value, &ExportOptions::default()).is_err()
    );
}

#[test]
fn particle_texture_program_counts_and_unmapped_material_controls_are_explicit() {
    let mut slot_values = SlotValues::default();
    let mut duplicated = particle_program_bundle(&[[0, 0], [7, 9], [0, 0]]);
    duplicated
        .model
        .insert_extension(
            0,
            m2::chunks::Chunk::TextureCombinerHints(ZeroVec::alloc_from_slice(&[
                [0, 0],
                [2, 1],
                [0, 0],
            ])),
        )
        .unwrap();
    let error = io::m2::import(&duplicated, &ImportOptions::default()).unwrap_err();
    assert!(error.to_string().contains("multiple TXAC"));
    let malformed = particle_program_bundle(&[[0, 0], [7, 9]]);
    assert!(io::m2::import(&malformed, &ImportOptions::default()).is_err());
    let source = particle_program_bundle(&[[5, 1], [3, 11], [0, 0]]);
    assert!(io::m2::import(&source, &ImportOptions::default()).is_err());
    let imported = io::m2::import(
        &source,
        &ImportOptions {
            loss_policy: LossPolicy::Drop,
            ..SlotConstruct::construct(&mut slot_values)
        },
    )
    .unwrap();
    assert!(!imported.report.issues.is_empty());
    assert!(imported.value.features().iter().any(|(_, feature)| {
        matches!(&feature.payload, mp::ActorKind::Particle(particle) if particle.details.rendering.texture_program == Some([3,11]))
    }));
}
#[test]
fn classic_geometry_becomes_editable_and_exports_modern_skin() {
    let source = native_triangle();
    let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();
    let semantic = &imported.value;
    let mesh = semantic.meshes().iter().next().unwrap().1;
    assert!(mesh.positions().is_owned());
    assert_eq!(
        mesh.positions().get(1),
        Some(Vec3::from_array([1., 0., 0.]))
    );
    let sm::Topology::Triangles(triangles) = mesh.topology() else {
        panic!("triangle topology lost")
    };
    assert!(triangles.is_owned());
    assert_eq!(triangles.get(0), Some([0, 1, 2]));
    assert_eq!(mesh.influences().joint_weights(0).unwrap()[0].weight, 1.);
    let texture = imported.value.sampled_images()[0];
    assert!(
        matches!(imported.value.image_resource(texture.resource).unwrap(),sm::M2ImageResource::Path(Cow::Borrowed(path))if *path==b"textures/checker.blp")
    );
    let target = io::m2::plan::<m2::Chunked<m2::V274>>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(
        target
            .model
            .core()
            .vertices
            .get(1)
            .unwrap()
            .texture_coordinates[1],
        [0.75, 0.25]
    );
    assert_eq!(
        target.model.core().materials.get(0).unwrap().blending_mode,
        BlendMode::Alpha
    );
    assert_eq!(target.skin(0, false).unwrap().profile.sections[0].id, 17);
    let encoded = target.encode().unwrap();
    assert_eq!(encoded.files.len(), 1);
    let decoded = m2::Bundle::<m2::Chunked<m2::V274>>::decode(
        &encoded.model,
        m2::FileKey::Root,
        |dependency| Ok(encoded.files.get(&dependency.key).map(Vec::as_slice)),
    )
    .unwrap();
    assert_eq!(
        decoded.skin(0, false).unwrap().profile.indices.get(2),
        Some(2)
    );
}
#[test]
fn hermite_tangents_and_legacy_clock_survive_editable_roundtrip() {
    let mut slot_values = SlotValues::default();
    let mut source = native_triangle();
    let core = source.model.core_mut();
    core.sequences.push(m2::records::Sequence {
        timing: m2::records::LegacyTiming {
            start: 2000,
            end: 3000,
        },
        id: 4,
        flags: SequenceFlags::from_bits_retain(0x20),
        variation_next: -1,
        ..SlotConstruct::construct(&mut slot_values)
    });
    core.bones[0].translation = Track {
        base: TrackBase {
            interpolation: Interpolation::Hermite,
            global_sequence: u16::MAX,
            timeline: LegacyTimeline {
                ranges: ZeroVec::alloc_from_slice(&[[0, 1]]),
                timestamps: ZeroVec::alloc_from_slice(&[2000, 3000]),
            },
        },
        values: LegacyValues::Splines(ZeroVec::alloc_from_slice(&[
            [[0., 0., 0.], [1., 0., 0.], [2., 0., 0.]],
            [[2., 0., 0.], [3., 0., 0.], [4., 0., 0.]],
        ])),
    };
    let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();
    let model = &imported.value;
    let (clip_id, _clip) = model.clips().iter().next().unwrap();
    let sm::DynamicTrack::Vector3(track) = model.clip_tracks(clip_id).next().unwrap() else {
        panic!("translation channel lost")
    };
    assert_eq!(track.curve.interpolation(), sm::Interpolation::Hermite);
    assert!(track.curve.values().is_owned());
    assert_eq!(track.curve.times().get(0).unwrap().as_seconds(), 0.);
    assert_eq!(track.curve.times().get(1).unwrap().as_seconds(), 1.);
    assert_eq!(
        track.curve.hermite_derivatives().unwrap().incoming.get(1),
        Some(Vec3::from_array([3., 0., 0.]))
    );
    let exported = io::m2::plan::<m2::Flat<m2::V256>>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let track = exported.model.core().bones[0]
        .translation
        .sequence(0)
        .unwrap();
    assert_eq!(track.timestamps.get(1), Some(1000));
    match track.values {
        m2::records::SequenceValues::Splines(values) => {
            assert_eq!(values.get(1).unwrap()[2], [4., 0., 0.])
        }
        _ => panic!("tangent representation lost"),
    };
}
#[test]
fn semantic_geometry_edits_rebuild_native_values() {
    let source = native_triangle();
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, mesh) = asset.meshes().iter().next().unwrap();
    let mut mesh = mesh.clone();
    let mut positions: Vec<_> = mesh.positions().iter().collect();
    positions[1] = Vec3::from_array([5., 0., 0.]);
    let sm::Topology::Triangles(triangles) = mesh.topology() else {
        unreachable!()
    };
    let normals = mesh.normals().unwrap().clone();
    let uv = std::array::from_fn(|slot| mesh.texture_coordinates()[slot].clone());
    let influences = (mesh
        .influences()
        .iter()
        .map(|row| -> [sm::Influence; 4] { row.into_joint_weights().unwrap().try_into().unwrap() })
        .collect::<Vec<_>>())
    .into_zero_vec();
    mesh.geometry = sm::M2Geometry::new(
        (positions).into_packed_array(),
        normals,
        uv,
        triangles.clone(),
        influences,
    )
    .unwrap();
    asset.edit(|draft| draft.set_mesh(id, mesh)).unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V263>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(
        output.model.core().vertices.get(1).unwrap().position,
        [5., 0., 0.]
    );
}
#[test]
fn quantization_requires_an_explicit_tolerance() {
    let mut slot_values = SlotValues::default();
    let mut builder = sm::Model::<sm::profiles::M2>::builder("quantized");
    let node = {
        let definition = builder
            .new_node("bone")
            .with_properties(mp::BoneProperties {
                key_bone: -1,
                flags: mp::values::BoneBehaviorFlags::default(),
                billboard_flags: Default::default(),
                submesh: 0,
                name_crc: 0,
            });
        builder.add_node(definition)
    }
    .unwrap();
    let clip = builder
        .add_clip(sm::Clip::named("clip").with_duration(sm::Time::seconds(1.).unwrap()))
        .unwrap();
    let curve = sm::Curve::linear(
        (vec![sm::Time::seconds(0.0004).unwrap()]).into_zero_vec(),
        (vec![[0.; 3]]).into_zero_vec(),
    )
    .unwrap();
    builder
        .add_track(
            clip,
            sm::DynamicTrack::Vector3(sm::Track {
                target: builder.model().node(node).unwrap().transform.translation,
                curve,
                clock: sm::TimeSource::Animation,
            }),
        )
        .unwrap();
    let asset = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    assert!(io::m2::plan::<m2::Flat<m2::V264>>(&asset, &ExportOptions::default()).is_err());
    let plan = io::m2::plan::<m2::Flat<m2::V264>>(
        &asset,
        &ExportOptions {
            approximation: ApproximationPolicy::Allow { tolerance: 0.0005 },
            ..SlotConstruct::construct(&mut slot_values)
        },
    )
    .unwrap();
    assert!(!plan.report().issues.is_empty());
}
#[test]
fn unresolved_companions_are_errors() {
    let mut slot_values = SlotValues::default();
    let mut model = m2::Document::<m2::Flat<m2::V264>>::new();
    model.core_mut().skins = 1;
    assert!(io::m2::import(
        &m2::Bundle::from_model(model),
        &ImportOptions {
            loss_policy: LossPolicy::Drop,
            ..SlotConstruct::construct(&mut slot_values)
        }
    )
    .is_err());
}
fn constant<T: m2::codec::Value>(value: T) -> Track<'static, T, m2::V256>
where
    [T; 3]: m2::codec::Value,
{
    Track {
        base: TrackBase {
            global_sequence: 0,
            timeline: LegacyTimeline {
                timestamps: ZeroVec::alloc_from_slice(&[0]),
                ..Default::default()
            },
            ..Default::default()
        },
        values: LegacyValues::Values(ZeroVec::alloc_from_slice(&[value])),
    }
}
#[test]
fn lights_particles_ribbons_and_collision_use_logical_fields() {
    let mut slot_values = SlotValues::default();
    let mut source = native_triangle();
    let core = source.model.core_mut();
    core.global_sequences = ZeroVec::alloc_from_slice(&[1000]);
    core.lights.push(m2::records::Light {
        kind: LightKind::Point,
        bone_id: 0,
        position: [1., 2., 3.],
        diffuse_color: constant([0.1, 0.2, 0.3]),
        diffuse_intensity: constant(2.),
        visibility: constant(1),
        ..SlotConstruct::construct(&mut slot_values)
    });
    let mut particle = m2::records::ParticleEmitter::<m2::V256>::new(ModelFlags::default());
    particle.id = 7;
    particle.bone_id = 0;
    particle.emission_speed = constant(12.);
    particle.lifespan = constant(3.);
    particle.emission_rate = constant(5.);
    particle.wind = [1., 0., 2.];
    particle.drag = 0.25;
    core.particles.push(particle);
    core.ribbons.push(m2::records::RibbonEmitter {
        bone_id: 0,
        position: [3., 2., 1.],
        edges_per_second: 12.5,
        edge_lifetime: 1.5,
        texture_indices: ZeroVec::alloc_from_slice(&[0]),
        material_indices: ZeroVec::alloc_from_slice(&[0]),
        height_above: constant(2.),
        height_below: constant(1.),
        color: constant([0.25, 0.5, 1.]),
        alpha: constant(32767),
        texture_rows: 1,
        texture_columns: 1,
        ..SlotConstruct::construct(&mut slot_values)
    });
    core.collision_vertices =
        ZeroVec::alloc_from_slice(&[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
    core.collision_indices = ZeroVec::alloc_from_slice(&[0, 1, 2]);
    let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();

    let model = &imported.value;
    assert_eq!(
        model
            .features()
            .iter()
            .filter(|(_, f)| matches!(
                f.payload,
                mp::ActorKind::Attachment(_)
                    | mp::ActorKind::Light(_)
                    | mp::ActorKind::Ribbon(_)
                    | mp::ActorKind::Particle(_)
                    | mp::ActorKind::Camera(_)
            ))
            .count(),
        3
    );
    assert!(model
        .systems()
        .iter()
        .any(|(_, system)| matches!(system, mp::System::Collision(_))));
    let exported = io::m2::plan::<m2::Flat<m2::V256>>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let core = exported.model.core();
    assert_eq!(core.lights.len(), 1);
    match core.lights[0].diffuse_intensity.sequence(0).unwrap().values {
        m2::records::SequenceValues::Values(values) => assert_eq!(values.get(0), Some(2.)),
        _ => panic!("unexpected light spline"),
    };
    assert_eq!(core.particles[0].wind, [1., 0., 2.]);
    assert_eq!(core.particles[0].drag, 0.25);
    assert_eq!(core.ribbons[0].edges_per_second, 12.5);
    assert_eq!(core.ribbons[0].edge_lifetime, 1.5);
    let (ribbon_id, ribbon) = imported
        .value
        .features()
        .iter()
        .find(|(_, feature)| matches!(feature.payload, mp::ActorKind::Ribbon(_)))
        .unwrap();
    let mp::ActorKind::Ribbon(definition) = &ribbon.payload else {
        unreachable!()
    };
    assert_eq!(definition.geometry.axis, sm::Axis::PositiveZ);
    let roundtrip = io::m2::import(&exported, &ImportOptions::default())
        .unwrap()
        .value;
    let (_, ribbon) = roundtrip
        .features()
        .iter()
        .find(|(_, feature)| matches!(feature.payload, mp::ActorKind::Ribbon(_)))
        .unwrap();
    let mp::ActorKind::Ribbon(definition) = &ribbon.payload else {
        unreachable!()
    };
    assert_eq!(definition.geometry.axis, sm::Axis::PositiveZ);
    let mut changed = imported.value.feature(ribbon_id).unwrap().payload.clone();
    let mp::ActorKind::Ribbon(definition) = &mut changed else {
        unreachable!()
    };
    definition.geometry.axis = sm::Axis::PositiveY;
    let mut edited = imported.value.clone();
    edited
        .edit(|builder| builder.set_feature(ribbon_id, changed))
        .unwrap();
    assert!(io::m2::plan::<m2::Flat<m2::V256>>(&edited, &ExportOptions::default()).is_err());
    assert_eq!(core.collision_indices.get(2), Some(2));
    assert_eq!(core.collision_normals.get(0), Some([0., 0., 1.]));
}
#[test]
fn native_material_edits_affect_rebuilt_output() {
    let source = native_triangle();
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, material) = asset.materials().iter().next().unwrap();
    let mut surface = material.surface.clone();
    let mp::MaterialStep::Diffuse(shader) = &mut surface.steps[0] else {
        panic!("expected diffuse")
    };
    shader.use_texture_alpha = true;
    asset
        .edit(|draft| draft.set_material_surface(id, surface))
        .unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(output.model.core().skins[0].batches[0].shader_id, 0x10);
}
#[test]
fn unknown_extensions_require_explicit_drop_policy() {
    let mut slot_values = SlotValues::default();
    let mut model = native_triangle().model.into_chunked();
    model
        .insert_extension(
            0,
            m2::chunks::Chunk::Unknown {
                tag: *b"TEST",
                data: Cow::Owned(vec![1, 2, 3]),
            },
        )
        .unwrap();
    let source = m2::Bundle::from_model(model);
    assert!(io::m2::import(&source, &ImportOptions::default()).is_err());
    let imported = io::m2::import(
        &source,
        &ImportOptions {
            unknown: UnknownPolicy::Drop,
            ..SlotConstruct::construct(&mut slot_values)
        },
    )
    .unwrap();

    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "m2.chunk" && issue.kind == conversion::IssueKind::Dropped));
    let output = io::m2::plan::<m2::Chunked<m2::V256>>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute();
    assert!(!output
        .value
        .model
        .extensions()
        .any(|chunk| chunk.tag() == *b"TEST"));
}
#[test]
fn compressed_rotations_decode_to_unit_components() {
    use io::m2::RotationComponents as _;
    assert_eq!([-32768i16, -32768, -32768, -1].floats(), [0., 0., 0., 1.]);
    assert_eq!([0i16, 0, 0, 32767].floats(), [-1., -1., -1., 0.]);
}

#[test]
fn native_physics_shapes_and_body_offsets_are_editable() {
    let mut slot_values = SlotValues::default();
    use wc3::model::formats::m2::companions::{
        PhysicsBody4, PhysicsBox, PhysicsChunk, PhysicsFile, PhysicsFrame, PhysicsShape,
        PhysicsShape2,
    };
    let mut source = native_triangle();
    source.model.core_mut().flags.set_external_physics(true);
    let file = PhysicsFile {
        chunks: vec![
            PhysicsChunk::Header(6),
            PhysicsChunk::Kind(PhysicsKind::WornItem),
            PhysicsChunk::Boxes(vec![PhysicsBox {
                frame: PhysicsFrame {
                    axis_x: [1., 0., 0.],
                    axis_y: [0., 1., 0.],
                    axis_z: [0., 0., 1.],
                    origin: [0., 0., 0.],
                },
                half_extents: [1., 2., 3.],
            }]),
            PhysicsChunk::Shapes2(vec![PhysicsShape2 {
                shape: PhysicsShape {
                    shape_type: PhysicsShapeKind::Box,
                    shape_index: 0,
                    friction: 0.25,
                    restitution: 0.5,
                    density: 2.,
                    game_flags: PhysicsGameFlags::from_bits_retain(7),
                    ..SlotConstruct::construct(&mut slot_values)
                },
                ..SlotConstruct::construct(&mut slot_values)
            }]),
            PhysicsChunk::Bodies4(vec![PhysicsBody4 {
                body_type: PhysicsBodyKind::Kinematic,
                bone_index: 0,
                position: [4., 5., 6.],
                shape_index: 0,
                shape_count: 1,
                gravity_scale: 0.5,
                inertia_scale: 1.5,
                follow_factor: 0.75,
                ..SlotConstruct::construct(&mut slot_values)
            }]),
        ],
        padding: Cow::Owned(Vec::new()),
    };
    source.insert(source.key.sibling(".phys"), m2::File::Physics(file));
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (system_id, system) = asset
        .systems()
        .iter()
        .find(|(_, s)| matches!(s, mp::System::Physics(_)))
        .unwrap();
    let mut system = system.clone();
    let mp::System::Physics(physics) = &mut system else {
        unreachable!()
    };
    let shape = &mut physics.bodies[0].shapes[0];
    assert!(matches!(
        shape.geometry,
        mp::ShapeGeometry::Box(v) if v == Vec3::new(1., 2., 3.)
    ));
    shape.geometry = mp::ShapeGeometry::Box(Vec3::from_array([7., 8., 9.]));
    asset
        .edit(|draft| draft.set_system(system_id, system))
        .unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let file = output
        .files()
        .find_map(|(_, file)| {
            if let m2::File::Physics(file) = file {
                Some(file)
            } else {
                None
            }
        })
        .unwrap();
    let boxes = file
        .chunks
        .iter()
        .find_map(|chunk| {
            if let PhysicsChunk::Boxes(boxes) = chunk {
                Some(boxes)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(boxes[0].half_extents, [7., 8., 9.]);
    let bodies = file
        .chunks
        .iter()
        .find_map(|chunk| {
            if let PhysicsChunk::Bodies4(bodies) = chunk {
                Some(bodies)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(bodies[0].position, [4., 5., 6.]);
    assert_eq!(bodies[0].follow_factor, 0.75);
    assert_eq!(output.model.core().bones.len(), 1);
    output.encode().unwrap();
}

#[test]
fn sparse_bone_overrides_use_editable_pose_matrices() {
    use wc3::model::formats::m2::companions::{BoneFile, BoneMatrix};
    let mut model = native_triangle().model.into_chunked();
    model
        .insert_extension(
            0,
            m2::chunks::Chunk::BoneIds(ZeroVec::alloc_from_slice(&[55])),
        )
        .unwrap();
    let mut source = m2::Bundle::from_model(model);
    let mut values = [0.; 16];
    for i in 0..4 {
        values[i * 5] = 1.;
    }
    values[12] = 3.;
    source.insert(
        m2::FileKey::FileId(55),
        m2::File::Bone(BoneFile {
            version: 1,
            indices: ZeroVec::alloc_from_slice(&[0]),
            matrices: vec![BoneMatrix { values }],
        }),
    );
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (system_id, system) = asset
        .systems()
        .iter()
        .find(|(_, s)| matches!(s, mp::System::BoneOverrides(_)))
        .unwrap();
    let mut system = system.clone();
    let mp::System::BoneOverrides(overrides) = &mut system else {
        unreachable!()
    };
    let matrices = &mut overrides[0].transforms;
    assert!(matrices.is_owned());
    let mut matrix = matrices.get(0).unwrap();
    assert_eq!(matrix.w_axis.x, 3.);
    matrix.w_axis.x = 9.;
    *matrices = (vec![matrix]).into_packed_array();
    asset
        .edit(|draft| draft.set_system(system_id, system))
        .unwrap();
    let output = io::m2::plan::<m2::Chunked<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let bone = output
        .files()
        .find_map(|(key, file)| {
            if *key == m2::FileKey::FileId(55) {
                if let m2::File::Bone(bone) = file {
                    Some(bone)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(bone.indices.get(0), Some(0));
    assert_eq!(bone.matrices[0].values[12], 9.);
    output.encode().unwrap();
}

#[test]
fn packed_particle_gravity_is_a_direction_and_rejects_malformed_words() {
    let mut slot_values = SlotValues::default();
    let mut source = native_triangle();
    source.model.core_mut().global_sequences = ZeroVec::alloc_from_slice(&[1000]);
    let word = u32::from_le_bytes([32, 0, 100, 0]);
    let mut emitter = m2::records::ParticleEmitter::<m2::V256>::new(ModelFlags::default());
    emitter.flags.set_packed_gravity(true);
    emitter.bone_id = 0;
    emitter.gravity = constant(word);
    source.model.core_mut().particles.push(emitter);
    let asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let semantic = &asset;
    let gravity_slot = semantic
        .features()
        .iter()
        .find_map(|(_, feature)| match &feature.payload {
            mp::ActorKind::Particle(particle) => Some(particle.motion.gravity),
            _ => None,
        })
        .unwrap();
    let sm::DynamicTrack::Vector3(track) = semantic
        .global_tracks()
        .filter(|v| matches!(v.clock(), TimeSource::Shared(_)))
        .find(|channel| channel.target() == gravity_slot.slot())
        .unwrap()
    else {
        panic!("gravity channel lost")
    };
    let gravity = track.curve.values().get(0).unwrap();
    assert!(gravity[0] > 1.);
    assert_eq!(gravity[1], 0.);
    assert!(gravity[2] > 4.);
    let options = ExportOptions {
        approximation: ApproximationPolicy::Allow { tolerance: 0.00001 },
        ..SlotConstruct::construct(&mut slot_values)
    };
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &options)
        .unwrap()
        .execute()
        .value;
    match &output.model.core().particles[0].gravity.values {
        LegacyValues::Values(values) => assert_eq!(values.get(0), Some(word)),
        _ => panic!("unexpected spline"),
    }
    drop(asset);
    source.model.core_mut().particles[0].gravity = constant(u32::from_le_bytes([127, 127, 100, 0]));
    assert!(io::m2::import(&source, &ImportOptions::default()).is_err());
}

#[test]
fn static_node_transforms_become_constant_native_curves() {
    let source = native_triangle();
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let id = asset
        .nodes()
        .iter()
        .find(|(_, node)| node.properties.is_some())
        .unwrap()
        .0;
    let node = asset.node(id).unwrap().clone();
    let translation = node.transform.translation;
    asset
        .edit(|draft| {
            draft
                .values_mut()
                .set(translation, Vec3::from_array([2., 3., 4.]))
        })
        .unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let LegacyValues::Values(values) = &output.model.core().bones[0].translation.values else {
        panic!("unexpected spline")
    };
    assert_eq!(values.get(0), Some([2., 3., 4.]));
    assert_ne!(
        output.model.core().bones[0]
            .translation
            .base
            .global_sequence,
        u16::MAX
    );
    output.encode().unwrap();
}

#[test]
fn local_effect_transforms_drive_the_effects_emitted_bone() {
    let mut slot_values = SlotValues::default();
    let mut source = native_triangle();
    source.model.core_mut().lights.push(m2::records::Light {
        kind: LightKind::Point,
        bone_id: 0,
        position: [1., 2., 3.],
        ..SlotConstruct::construct(&mut slot_values)
    });
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let node = asset
        .features()
        .iter()
        .find(|(_, f)| matches!(f.payload, mp::ActorKind::Light(_)))
        .unwrap()
        .1
        .node;
    asset
        .edit(|draft| {
            let clock = draft.add_clock(sm::SharedClock::Repeating {
                period: sm::Time::seconds(1.)?,
            })?;
            let curve = sm::Curve::step(
                (vec![sm::Time::default()]).into_zero_vec(),
                (vec![[5., 0., 0.]]).into_zero_vec(),
            )?;
            draft.add_global_track(sm::DynamicTrack::Vector3(sm::Track {
                target: draft.model().node(node).unwrap().transform.translation,
                curve,
                clock: sm::TimeSource::Shared(clock),
            }))
        })
        .unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(output.model.core().lights[0].bone_id, 1);
    let LegacyValues::Values(values) = &output.model.core().bones[1].translation.values else {
        panic!("unexpected spline")
    };
    assert_eq!(values.get(0), Some([5., 0., 0.]));
    output.encode().unwrap();
}

#[test]
fn unknown_native_blend_does_not_collide_with_common_transparency() {
    let mut source = native_triangle();
    source.model.core_mut().materials = ZeroVec::alloc_from_slice(&[Material {
        flags: MaterialFlags::default(),
        blending_mode: BlendMode::Unknown(8),
    }]);
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let surface = &asset.materials().iter().next().unwrap().1.surface;
    assert!(surface.steps[0].declaration().state.blend == mp::values::BlendMode::Unknown(8));
    let id = asset.nodes().iter().next().unwrap().0;
    let mut node = asset.node(id).unwrap().clone();
    node.name = Cow::Borrowed("edited");
    asset.edit(|draft| draft.set_node(id, node)).unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(
        output.model.core().materials.get(0).unwrap().blending_mode,
        BlendMode::Unknown(8)
    );
}

#[test]
fn legacy_particle_shape_and_head_tail_controls_survive_conversion() {
    use m2::records::{LegacyTextureScale, ParticleEmitter, ParticleHeadTail};
    let mut source = native_triangle();
    let mut particle = ParticleEmitter::<m2::V256>::new(ModelFlags::default());
    particle.id = 51;
    particle.texture_scale = LegacyTextureScale {
        particle_type: 77,
        head_or_tail: ParticleHeadTail::Both,
    };
    source.model.core_mut().particles.push(particle);
    let imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(
        output.model.core().particles[0].texture_scale.particle_type,
        77
    );
    assert_eq!(
        output.model.core().particles[0].texture_scale.head_or_tail,
        ParticleHeadTail::Both
    );
}

#[test]
fn texture_sampling_drops_uninterpreted_flags_and_rejects_unencoded_interpretations() {
    let mut slot_values = SlotValues::default();
    let mut source = native_triangle();
    source.model.core_mut().textures[0].flags = TextureFlags::from_bits_retain(0x8000_0001);
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let id = asset.sampled_images()[0];
    let texture = id;
    assert_eq!(texture.color_space, sm::ColorSpace::Unspecified);
    let native = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(native.model.core().textures[0].flags.bits(), 1);
    let (material, value) = asset.materials().iter().next().unwrap();
    let original = value.surface.clone();
    for invalid_color in [false, true] {
        assert!(asset
            .edit(|draft| {
                let edit_result = {
                    let mut value = original.clone();
                    let input = value.steps[0].inputs()[0].clone();
                    let mut changed = input;
                    if invalid_color {
                        changed.image.texture.as_mut().unwrap().color_space = sm::ColorSpace::Srgb;
                    } else {
                        changed.image.texture.as_mut().unwrap().sampler.u = sm::AddressMode::Mirror;
                    }
                    let mut inputs = value.steps[0]
                        .inputs()
                        .into_iter()
                        .cloned()
                        .collect::<Vec<_>>();
                    inputs[0] = changed;
                    let declaration = value.steps[0].declaration().clone();
                    value.steps[0] = mp::MaterialStep::from_native(
                        value.steps[0].native_shader(),
                        declaration,
                        inputs,
                        SlotConstruct::construct(&mut slot_values),
                    );
                    draft.set_material_surface(material, value)
                };
                draft.import_values(&slot_values)?;
                edit_result
            })
            .is_err());
    }
}

#[test]
fn replacement_texture_retains_its_separately_declared_filename() {
    let mut source = native_triangle();
    source.model.core_mut().textures[0].kind = TextureSource::Unknown(0x1234);
    let asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let texture = asset.sampled_images()[0];
    assert!(matches!(asset.image_resource(texture.resource).unwrap(),
        sm::M2ImageResource::Replaceable { id: 0x1234, filename: Cow::Borrowed(path) } if *path == b"textures/checker.blp"));
    let native = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let texture = &native.model.core().textures[0];
    assert_eq!(texture.kind, TextureSource::Unknown(0x1234));
    assert_eq!(texture.filename.as_bytes(), b"textures/checker.blp\0");
}

#[test]
fn native_alias_keeps_light_programme_on_its_referenced_clip() {
    let mut slot_values = SlotValues::default();
    let mut source = native_triangle();
    let core = source.model.core_mut();
    core.sequences = vec![
        m2::records::Sequence {
            id: 4,
            timing: m2::records::LegacyTiming {
                start: 0,
                end: 1000,
            },
            flags: SequenceFlags::from_bits_retain(0x20),
            variation_next: -1,
            ..SlotConstruct::construct(&mut slot_values)
        },
        m2::records::Sequence {
            id: 5,
            timing: m2::records::LegacyTiming {
                start: 0,
                end: 1000,
            },
            flags: SequenceFlags::from_bits_retain(0x40),
            alias_next: 0,
            variation_next: -1,
            ..SlotConstruct::construct(&mut slot_values)
        },
    ];
    core.lights.push(m2::records::Light {
        kind: LightKind::Point,
        bone_id: 0,
        diffuse_intensity: Track {
            base: TrackBase {
                interpolation: Interpolation::Linear,
                global_sequence: u16::MAX,
                timeline: LegacyTimeline {
                    ranges: ZeroVec::alloc_from_slice(&[[0, 1], [0, 1]]),
                    timestamps: ZeroVec::alloc_from_slice(&[0, 1000]),
                },
            },
            values: LegacyValues::Values(ZeroVec::alloc_from_slice(&[2., 6.])),
        },
        ..SlotConstruct::construct(&mut slot_values)
    });
    let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();
    let model = &imported.value;
    let clips = model.clips().iter().collect::<Vec<_>>();
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[1].1.properties.alias, Some(clips[0].0));
    assert_eq!(clips[1].1.animations, clips[0].1.animations);
    assert!(model.clip_tracks(clips[0].0).next().is_some());
    let program = sm::AnimationProgram::compile(model);
    let sample = program
        .sample(sm::AnimationSample {
            clip: Some(clips[1].0),
            clip_time: sm::Time::seconds(0.5).unwrap(),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .unwrap();
    let sm::DynamicTrack::Scalar(track) = model.clip_tracks(clips[0].0).next().unwrap() else {
        panic!("light intensity must remain a typed scalar channel")
    };
    assert_eq!(sample.override_value(&track.target), Some(4.));
}

#[test]
fn zero_duration_global_tracks_are_typed_frozen_clocks_and_roundtrip_without_retiming() {
    let mut slot_values = SlotValues::default();
    let mut invalid = sm::ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    invalid
        .add_clock(sm::SharedClock::Repeating {
            period: sm::Time::default(),
        })
        .unwrap();
    assert!(invalid.finish().is_err());
    let mut source = native_triangle();
    source.model.core_mut().global_sequences = ZeroVec::alloc_from_slice(&[0, 1000]);
    source.model.core_mut().bones[0].translation = constant([2., 3., 4.]);
    let mut scale = constant([1., 2., 3.]);
    scale.base.global_sequence = 1;
    source.model.core_mut().bones[0].scale = scale;
    let imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let clocks: Vec<_> = imported.clocks().iter().map(|(_, clock)| *clock).collect();
    assert_eq!(
        clocks,
        vec![
            sm::SharedClock::Frozen,
            sm::SharedClock::Repeating {
                period: sm::Time::seconds(1.).unwrap()
            }
        ]
    );
    let node = imported.nodes().iter().next().unwrap().0;
    let program = sm::AnimationProgram::compile(&imported);
    for seconds in [-1., 0., 0.5, 100.] {
        let sample = program
            .sample(sm::AnimationSample {
                shared_time: sm::Time::seconds(seconds).unwrap(),
                ..SlotConstruct::construct(&mut slot_values)
            })
            .unwrap();
        assert_eq!(
            sample.override_value(&imported.node(node).unwrap().transform.translation),
            Some(Vec3::from_array([2., 3., 4.]))
        );
        assert_eq!(
            sample.override_value(&imported.node(node).unwrap().transform.scale),
            Some(Vec3::from_array([1., 2., 3.]))
        );
    }
    let exported = io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(
        exported.model.core().global_sequences.to_vec(),
        vec![0, 1000]
    );
    assert_eq!(
        exported.model.core().bones[0]
            .translation
            .base
            .global_sequence,
        0
    );
    assert_eq!(exported.model.core().bones[0].scale.base.global_sequence, 1);
    assert_eq!(
        exported.model.core().bones[0]
            .translation
            .sequence(0)
            .unwrap()
            .timestamps
            .iter()
            .collect::<Vec<_>>(),
        vec![0]
    );
}

fn material_routes(shader: u16, count: usize) -> m2::Bundle<'static, m2::Flat<m2::V256>> {
    let mut source = native_triangle();
    let core = source.model.core_mut();
    core.global_sequences = ZeroVec::alloc_from_slice(&[1000]);
    core.texture_combos = ZeroVec::alloc_from_slice(&vec![0; count]);
    core.texture_weight_combos = ZeroVec::alloc_from_slice(&vec![u16::MAX; count]);
    core.texture_transform_bone_map = ZeroVec::alloc_from_slice(&vec![7; count]);
    core.texture_transform_combos =
        ZeroVec::alloc_from_slice(&(0..count as u16).collect::<Vec<_>>());
    core.texture_transforms = (0..count)
        .map(|index| m2::records::TextureTransform {
            translation: constant([10. * (index + 1) as f32, 0., 0.]),
            rotation: constant([0., 0., 0., 1.]),
            scaling: constant([1.; 3]),
        })
        .collect();
    core.skins[0].batches[0].shader_id = shader;
    core.skins[0].batches[0].texture_count = count as u16;
    source
}

#[test]
fn native_shader_routes_share_repeated_coordinate_animation() {
    let source = material_routes(0x8003, 3);
    let imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let surface = &imported.materials().iter().next().unwrap().1.surface;
    assert_eq!(
        surface.steps[0]
            .inputs()
            .iter()
            .map(|unit| unit.coordinates.source)
            .collect::<Vec<_>>(),
        [0, -1, 0]
    );
    assert_eq!(
        surface.steps[0].inputs()[0].coordinates.transform,
        surface.steps[0].inputs()[2].coordinates.transform
    );
    assert!(imported.is_animated(
        surface.steps[0].inputs()[0]
            .coordinates
            .transform
            .translation
    ));
    assert!(!imported.is_animated(
        surface.steps[0].inputs()[1]
            .coordinates
            .transform
            .translation
    ));
    let native = io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let reread = io::m2::import(&native, &ImportOptions::default())
        .unwrap()
        .value;
    let surface = &reread.materials().iter().next().unwrap().1.surface;
    assert_eq!(
        surface.steps[0].inputs()[0].coordinates.transform,
        surface.steps[0].inputs()[2].coordinates.transform
    );
    assert!(reread.is_animated(
        surface.steps[0].inputs()[0]
            .coordinates
            .transform
            .translation
    ));
}

#[test]
fn native_environment_then_t1_exports_the_active_transform_in_slot_zero() {
    let source = material_routes(0x80, 2);
    let imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let surface = &imported.materials().iter().next().unwrap().1.surface;
    assert_eq!(surface.steps[0].inputs()[0].coordinates.source, -1);
    assert_eq!(surface.steps[0].inputs()[1].coordinates.source, 0);
    let native = io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let core = native.model.core();
    let index = core.texture_transform_combos.get(0).unwrap() as usize;
    let LegacyValues::Values(values) = &core.texture_transforms[index].translation.values else {
        panic!("plain transform values expected")
    };
    assert_eq!(values.get(0), Some([10., 0., 0.]));
}

#[test]
fn native_three_route_programme_uses_third_transform_for_uv1() {
    let source = material_routes(0x8014, 3);
    let imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let surface = &imported.materials().iter().next().unwrap().1.surface;
    assert_eq!(
        surface.steps[0]
            .inputs()
            .iter()
            .map(|unit| unit.coordinates.source)
            .collect::<Vec<_>>(),
        [0, -1, 1]
    );
    let native = io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let core = native.model.core();
    let index = core.texture_transform_combos.get(2).unwrap() as usize;
    let LegacyValues::Values(values) = &core.texture_transforms[index].translation.values else {
        panic!("plain transform values expected")
    };
    assert_eq!(values.get(0), Some([30., 0., 0.]));
}

#[test]
fn native_export_rejects_independent_transforms_on_repeated_shader_routes() {
    let mut slot_values = SlotValues::default();
    let source = material_routes(0x8003, 3);
    let mut imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, material) = imported.materials().iter().next().unwrap();
    let mut surface = material.surface.clone();
    surface.steps[0].inputs_mut()[2]
        .coordinates
        .transform
        .translation = slot_values.add(Vec3::from_array([99., 0., 0.]));
    imported
        .edit(|draft| {
            draft.import_values(&slot_values)?;
            draft.set_material_surface(id, surface)
        })
        .unwrap();
    assert!(io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default()).is_err());
}

#[test]
fn native_export_rejects_coordinate_sources_incompatible_with_shader_routes() {
    let source = material_routes(0x8014, 3);
    let mut imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, material) = imported.materials().iter().next().unwrap();
    let mut surface = material.surface.clone();
    surface.steps[0].inputs_mut()[2].coordinates.source = 0;
    imported
        .edit(|draft| draft.set_material_surface(id, surface))
        .unwrap();
    assert!(io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default()).is_err());
}

#[test]
fn application_guild_palettes_survive_profile_changes_but_require_export_loss_policy() {
    let mut slot_values = SlotValues::default();
    let source = material_routes(0x801a, 3);
    let mut imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, material) = imported.materials().iter().next().unwrap();
    let mut surface = material.surface.clone();
    assert_eq!(
        surface.steps[0].guild_palette().unwrap(),
        mp::GuildPalette::default()
    );
    let mp::MaterialStep::Guild(guild) = &mut surface.steps[0] else {
        panic!("expected guild")
    };
    guild.palette.background = Vec3::from_array([0.25, 0.5, 0.75]);
    imported
        .edit(|draft| draft.set_material_surface(id, surface))
        .unwrap();
    let broad = imported.clone().widen::<sm::profiles::Universal>();
    let narrowed = broad.try_narrow::<sm::profiles::M2>().unwrap();
    assert_eq!(
        narrowed.materials().iter().next().unwrap().1.surface.steps[0]
            .guild_palette()
            .unwrap()
            .background,
        Vec3::from_array([0.25, 0.5, 0.75])
    );
    let error =
        io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default()).unwrap_err();
    assert!(error.to_string().contains("guild"));
    let planned = io::m2::plan::<m2::Flat<m2::V256>>(
        &imported,
        &ExportOptions {
            loss_policy: LossPolicy::Drop,
            ..SlotConstruct::construct(&mut slot_values)
        },
    )
    .unwrap();
    let exported = planned.execute();
    assert!(exported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "m2.material.guild_palette"));
    let roundtrip = io::m2::import(&exported.value, &ImportOptions::default()).unwrap();
    assert_eq!(
        roundtrip
            .value
            .materials()
            .iter()
            .next()
            .unwrap()
            .1
            .surface
            .steps[0]
            .guild_palette()
            .unwrap(),
        mp::GuildPalette::default()
    );
}

#[test]
fn batch_layers_become_ordered_steps_with_independent_animation() {
    let mut source = native_triangle();
    let core = source.model.core_mut();
    core.global_sequences = ZeroVec::alloc_from_slice(&[1000]);
    core.colors = vec![
        m2::records::ColorAnimation {
            color: constant([1., 0., 0.]),
            alpha: constant(8192),
        },
        m2::records::ColorAnimation {
            color: constant([0., 1., 0.]),
            alpha: constant(24576),
        },
    ];
    core.texture_weights = vec![
        m2::records::TextureWeight {
            weight: constant(4096),
        },
        m2::records::TextureWeight {
            weight: constant(16384),
        },
    ];
    core.texture_weight_combos = ZeroVec::alloc_from_slice(&[0, 1]);
    let mut first = core.skins[0].batches[0];
    first.shader_id = 0;
    first.material_layer = 9;
    first.geoset_index = 37;
    first.color_index = 0;
    let mut second = first;
    second.material_layer = 2;
    second.shader_id = 0x10;
    second.color_index = 1;
    second.texture_weight_combo_index = 1;
    let mut third = first;
    third.color_index = -1;
    core.skins[0].batches = vec![first, second, third];
    let imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let instance = imported.instances().iter().next().unwrap().1;
    assert_eq!(instance.draws.len(), 1);
    assert_eq!(instance.selection_group, 17);
    assert_eq!(
        instance.draws[0]
            .properties
            .as_ref()
            .unwrap()
            .auxiliary_section,
        37
    );
    let (id, material) = imported.materials().iter().next().unwrap();
    assert_eq!(material.surface.steps.len(), 3);
    for (_index, step) in material.surface.steps.iter().enumerate().take(2) {
        assert!(imported.is_animated(step.declaration().color));
    }
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let core = output.model.core();
    let batches = &core.skins[0].batches;
    assert_eq!(
        batches.iter().map(|b| b.material_layer).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert!(batches.iter().all(|b| b.geoset_index == 37));
    for (batch, expected) in batches.iter().zip([16384, 4096]) {
        let weight = core
            .texture_weight_combos
            .get(batch.texture_weight_combo_index as usize)
            .unwrap();
        let LegacyValues::Values(values) = &core.texture_weights[weight as usize].weight.values
        else {
            panic!("weight values")
        };
        assert_eq!(values.get(0), Some(expected));
    }
    assert_eq!(
        batches.iter().map(|b| b.shader_id).collect::<Vec<_>>(),
        [0x10, 0, 0]
    );
    for (batch, expected) in batches.iter().zip([[0., 1., 0.], [1., 0., 0.]]) {
        let LegacyValues::Values(values) = &core.colors[batch.color_index as usize].color.values
        else {
            panic!("color values")
        };
        assert_eq!(values.get(0), Some(expected));
    }
    // Reordering semantic steps is the only operation needed to change native layer order.
    let mut edited = imported.clone();
    let mut program = material.surface.clone();
    program.steps.swap(0, 1);
    edited
        .edit(|draft| draft.set_material_surface(id, program))
        .unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&edited, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(
        output.model.core().skins[0]
            .batches
            .iter()
            .map(|b| b.shader_id)
            .collect::<Vec<_>>(),
        [0, 0x10, 0]
    );
}

#[test]
fn auxiliary_section_words_are_draw_local_and_survive_profile_changes() {
    let mut source = native_triangle();
    let mut first = source.model.core().skins[0].batches[0];
    first.geoset_index = 41;
    let mut second = first;
    second.geoset_index = 7;
    second.material_layer = 1;
    let mut third = first;
    third.material_layer = 2;
    source.model.core_mut().skins[0].batches = vec![first, second, third];
    let imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let broad = imported.widen::<sm::profiles::Universal>();
    let imported = broad.try_narrow::<sm::profiles::M2>().unwrap();
    let instance = imported.instances().iter().next().unwrap().1;
    assert_eq!(instance.draws.len(), 3);
    assert_eq!(instance.draws[0].subset, instance.draws[1].subset);
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(
        output.model.core().skins[0]
            .batches
            .iter()
            .map(|b| b.geoset_index)
            .collect::<Vec<_>>(),
        [41, 7, 41]
    );
}

#[test]
fn ribbon_export_flattens_material_programs_into_ordered_native_materials() {
    let mut slot_values = SlotValues::default();
    let mut source = native_triangle();
    source
        .model
        .core_mut()
        .ribbons
        .push(m2::records::RibbonEmitter {
            bone_id: 0,
            edges_per_second: 1.,
            edge_lifetime: 1.,
            texture_rows: 1,
            texture_columns: 1,
            texture_indices: ZeroVec::alloc_from_slice(&[0]),
            material_indices: ZeroVec::alloc_from_slice(&[0]),
            ..SlotConstruct::construct(&mut slot_values)
        });
    let mut imported = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let ribbon = imported
        .features()
        .iter()
        .find_map(|(_, feature)| match &feature.payload {
            mp::ActorKind::Ribbon(ribbon) => Some(ribbon),
            _ => None,
        })
        .unwrap();
    let material_id = ribbon.rendering.materials[0];
    let mut surface = imported.material(material_id).unwrap().surface.clone();
    let mut second = surface.steps[0].clone();
    second.declaration_mut().state.blend = sm::values::model::BlendMode::Additive;
    surface.steps.push(second);
    imported
        .edit(|draft| draft.set_material_surface(material_id, surface))
        .unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let core = output.model.core();
    let materials = &core.ribbons[0].material_indices;
    assert_eq!(materials.len(), 2);
    assert_eq!(
        core.materials
            .get(materials.get(0).unwrap() as usize)
            .unwrap()
            .blending_mode,
        BlendMode::Alpha
    );
    assert_eq!(
        core.materials
            .get(materials.get(1).unwrap() as usize)
            .unwrap()
            .blending_mode,
        BlendMode::from_raw(3)
    );
}

#[test]
fn export_interns_equivalent_images_with_independent_resource_declarations() {
    let source = native_triangle();
    let mut model = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (material, definition) = model.materials().iter().next().unwrap();
    let mut surface = definition.surface.clone();
    let image = surface.steps[0].inputs()[0].image.texture.unwrap();
    let resource = model.image_resource(image.resource).unwrap().clone();
    model
        .edit(|draft| {
            let mut step = surface.steps[0].clone();
            step.inputs_mut()[0]
                .image
                .texture
                .as_mut()
                .unwrap()
                .resource = draft.add_image_resource(resource);
            surface.steps.push(step);
            draft.set_material_surface(material, surface)
        })
        .unwrap();
    assert_eq!(model.sampled_images().len(), 2);
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(output.model.core().textures.len(), 1);
}

#[test]
fn offset_animation_interval_exports_as_native_local_keys() {
    let mut builder = sm::Model::<sm::profiles::M2>::builder("offset source");
    let definition = builder
        .new_node("bone")
        .with_properties(mp::BoneProperties {
            key_bone: -1,
            flags: mp::values::BoneBehaviorFlags::default(),
            billboard_flags: Default::default(),
            submesh: 0,
            name_crc: 0,
        });
    let property = definition.transform.translation;
    builder.add_node(definition).unwrap();
    let time = |seconds| sm::Time::seconds(seconds).unwrap();
    let clip = builder
        .add_clip(
            sm::Clip::named("offset")
                .with_source_start(time(10.))
                .with_duration(time(1.)),
        )
        .unwrap();
    builder
        .add_track(
            clip,
            sm::DynamicTrack::Vector3(sm::Track {
                target: property,
                clock: TimeSource::Animation,
                curve: sm::Curve::linear(
                    [time(9.), time(10.), time(11.), time(12.)],
                    ([[900.; 3], [2.; 3], [4.; 3], [1200.; 3]])
                        .into_iter()
                        .map(Vec3::from_array)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
        )
        .unwrap();
    let model = builder.finish().unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V264>>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let imported = io::m2::import(&output, &ImportOptions::default())
        .unwrap()
        .value;
    let (clip, selection) = imported.clips().iter().next().unwrap();
    assert_eq!(selection.source_start, time(0.));
    let node = imported.nodes().iter().next().unwrap().1;
    let sample = imported
        .animation_sample(sm::AnimationSample {
            clip: Some(clip),
            clip_time: time(0.5),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        sample.animated(&node.transform.translation),
        Vec3::from_array([3.; 3])
    );
    let sm::DynamicTrack::Vector3(track) = imported
        .clip_tracks(clip)
        .find(|channel| channel.target() == node.transform.translation.slot())
        .unwrap()
    else {
        panic!("missing translation")
    };
    assert_eq!(track.curve.times().to_vec(), [time(0.), time(1.)]);
}

#[test]
fn event_sources_round_trip_native_codes_data_clocks_and_duplicate_occurrences() {
    let mut source = native_triangle();
    let core = source.model.core_mut();
    core.global_sequences = ZeroVec::alloc_from_slice(&[1000]);
    for (identifier, timestamps) in [(*b"$SND", vec![750, 250, 250]), (*b"$EMP", vec![])] {
        core.events.push(m2::records::Event {
            identifier,
            data: 0x8000_0001,
            bone_id: 0,
            position: [1., 2., 3.],
            enabled: TrackBase {
                interpolation: Interpolation::Step,
                global_sequence: 0,
                timeline: LegacyTimeline {
                    ranges: ZeroVec::new(),
                    timestamps: ZeroVec::alloc_from_slice(&timestamps),
                },
            },
        });
    }
    let model = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(model.event_sources().len(), 2);
    assert_eq!(model.global_events().count(), 3);
    let widened = model.widen::<sm::profiles::Universal>();
    let restored = widened.try_narrow::<sm::profiles::M2>().unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&restored, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let events = &output.model.core().events;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].identifier, *b"$SND");
    assert_eq!(events[0].data, 0x8000_0001);
    assert_eq!(events[0].position, [1., 2., 3.]);
    assert_eq!(*events[0].enabled.timeline.timestamps, [750, 250, 250]);
    assert_eq!(events[1].identifier, *b"$EMP");
    assert!(events[1].enabled.timeline.timestamps.is_empty());
    assert_eq!(
        events[1].enabled.global_sequence,
        events[0].enabled.global_sequence
    );
}

#[test]
fn event_sources_reject_selections_that_native_m2_cannot_encode() {
    let source = native_triangle();
    let model = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let node = model.nodes().iter().next().unwrap().0;
    let mut model = model;
    let result = model.edit(|draft| {
        let event_source = draft.add_event_source(mp::EventSource {
            node,
            clock: TimeSource::Animation,
            identifier: *b"$SND",
            data: 1,
        });
        draft.add_global_event(sm::EventOccurrence {
            source: event_source,
            time: sm::Time::default(),
        })?;
        Ok(())
    });
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("m2.events.selection"));
    let result = model.edit(|draft| {
        let clock = draft.add_clock(sm::SharedClock::Frozen)?;
        let event_source = draft.add_event_source(mp::EventSource {
            node,
            clock: TimeSource::Shared(clock),
            identifier: *b"$SND",
            data: 1,
        });
        let clip = draft.add_clip(sm::Clip::named("clip"))?;
        draft.add_event(
            clip,
            sm::EventOccurrence {
                source: event_source,
                time: sm::Time::default(),
            },
        )
    });
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("m2.events.selection"));
    assert!(model.event_sources().is_empty());
}

#[test]
fn ordinary_event_source_retains_sequence_timing_and_unordered_duplicate_keys() {
    let mut source = native_triangle();
    let core = source.model.core_mut();
    core.sequences.push(m2::records::Sequence {
        timing: m2::records::LegacyTiming {
            start: 2000,
            end: 3000,
        },
        id: 4,
        flags: SequenceFlags::from_bits_retain(0x20),
        variation_next: -1,
        ..Default::default()
    });
    core.events.push(m2::records::Event {
        identifier: *b"$SND",
        data: 123,
        bone_id: 0,
        position: [0.; 3],
        enabled: TrackBase {
            interpolation: Interpolation::Step,
            global_sequence: u16::MAX,
            timeline: LegacyTimeline {
                ranges: ZeroVec::alloc_from_slice(&[[0, 2]]),
                timestamps: ZeroVec::alloc_from_slice(&[2750, 2250, 2250]),
            },
        },
    });
    let model = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let clip = model.clips().iter().next().unwrap().0;
    assert_eq!(
        model
            .clip_events(clip)
            .map(|event| event.time.as_seconds())
            .collect::<Vec<_>>(),
        [0.75, 0.25, 0.25]
    );
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let core = output.model.core();
    let origin = core.sequences[0].timing.start;
    assert_eq!(
        core.events[0]
            .enabled
            .timeline
            .timestamps
            .iter()
            .map(|ms| ms - origin)
            .collect::<Vec<_>>(),
        [750, 250, 250]
    );
}

#[test]
fn billboard_flags_round_trip_while_effective_states_are_normalized() {
    use sm::{Billboard, BillboardFrame, BillboardScale, EulerLocks, TransformSemantics};
    for bits in 0..16 {
        let mut source = native_triangle();
        let flags = &mut source.model.core_mut().bones[0].flags;
        flags.set_billboard(bits & 1 != 0);
        flags.set_billboard_lock_x(bits & 2 != 0);
        flags.set_billboard_lock_y(bits & 4 != 0);
        flags.set_billboard_lock_z(bits & 8 != 0);
        let original = *flags;
        let imported = io::m2::import(&source, &ImportOptions::default())
            .unwrap()
            .value;
        let bone = imported.nodes().iter().next().unwrap().1;
        assert_eq!(bone.transform_semantics, TransformSemantics::Affine);
        let expected = if bits & 1 == 0 {
            Billboard::None
        } else {
            match bits >> 1 {
                0 => Billboard::Spherical {
                    frame: BillboardFrame::Yz,
                    scale: BillboardScale::BasisLengths,
                },
                1 => Billboard::Euler(EulerLocks::X),
                2 => Billboard::Euler(EulerLocks::Y),
                3 => Billboard::Euler(EulerLocks::Xy),
                4 => Billboard::Euler(EulerLocks::Z),
                5 => Billboard::Euler(EulerLocks::Xz),
                6 => Billboard::Euler(EulerLocks::Yz),
                _ => Billboard::None,
            }
        };
        assert_eq!(bone.billboard, expected);
        // Widening and exact narrowing preserve explicit behavior and native declarations.
        let model = imported
            .widen::<sm::profiles::Universal>()
            .try_narrow::<sm::profiles::M2>()
            .unwrap();
        let output = io::m2::plan::<m2::Flat<m2::V256>>(&model, &ExportOptions::default())
            .unwrap()
            .execute()
            .value;
        assert_eq!(output.model.core().bones[0].flags, original);
    }
}

#[test]
fn billboard_edits_do_not_reactivate_inactive_m2_declarations() {
    use sm::{Billboard, BillboardFrame, BillboardScale};
    let mut source = native_triangle();
    let flags = &mut source.model.core_mut().bones[0].flags;
    flags.set_billboard_lock_x(true);
    flags.set_billboard_lock_z(true);
    let mut model = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, original) = model.nodes().iter().next().unwrap();
    let mut node = original.clone();
    node.billboard = Billboard::Spherical {
        frame: BillboardFrame::Yz,
        scale: BillboardScale::BasisLengths,
    };
    model.edit(|draft| draft.set_node(id, node)).unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let flags = output.model.core().bones[0].flags;
    assert!(flags.billboard());
    assert!(!flags.billboard_lock_x() && !flags.billboard_lock_y() && !flags.billboard_lock_z());
    let mut node = model.node(id).unwrap().clone();
    node.billboard = Billboard::None;
    model.edit(|draft| draft.set_node(id, node)).unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    let flags = output.model.core().bones[0].flags;
    assert!(!flags.billboard());
}

#[test]
fn m2_export_rejects_incompatible_facing_and_transform_operations() {
    use sm::{Billboard, BillboardAxis, BillboardFrame, BillboardScale, TransformSemantics};
    let source = native_triangle();
    for mode in [
        Billboard::Axial(BillboardAxis::X),
        Billboard::Spherical {
            frame: BillboardFrame::Camera,
            scale: BillboardScale::SignedFactors,
        },
        Billboard::Spherical {
            frame: BillboardFrame::Yz,
            scale: BillboardScale::SignedFactors,
        },
    ] {
        let mut model = io::m2::import(&source, &ImportOptions::default())
            .unwrap()
            .value;
        let (id, node) = model.nodes().iter().next().unwrap();
        let mut node = node.clone();
        node.billboard = mode;
        model.edit(|draft| draft.set_node(id, node)).unwrap();
        assert!(io::m2::plan::<m2::Flat<m2::V256>>(&model, &ExportOptions::default()).is_err());
    }
    let mut model = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let (id, node) = model.nodes().iter().next().unwrap();
    let mut node = node.clone();
    node.transform_semantics = TransformSemantics::PivotLocal;
    node.inheritance.rotation = false;
    model.edit(|draft| draft.set_node(id, node)).unwrap();
    assert!(io::m2::plan::<m2::Flat<m2::V256>>(&model, &ExportOptions::default()).is_err());
}

#[test]
fn file_data_image_declarations_retain_accompanying_filenames() {
    let mut model = native_triangle().model.into_chunked();
    model
        .insert_extension(
            0,
            m2::chunks::Chunk::TextureIds(ZeroVec::alloc_from_slice(&[42])),
        )
        .unwrap();
    let source = m2::Bundle::from_model(model);
    let asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let texture = asset.sampled_images()[0];
    assert!(matches!(asset.image_resource(texture.resource),
        Some(sm::M2ImageResource::FileDataId { id: 42, filename: Cow::Borrowed(path) }) if *path == b"textures/checker.blp"));
    assert_eq!(asset.dependencies().images().len(), 1);
    let asset = asset
        .widen::<sm::profiles::Universal>()
        .try_narrow::<sm::profiles::M2>()
        .unwrap();
    let output = io::m2::plan::<m2::Chunked<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .value;
    assert_eq!(
        output.model.core().textures[0].filename.as_bytes(),
        b"textures/checker.blp\0"
    );
    assert!(output.model.extensions().any(
        |chunk| matches!(chunk, m2::chunks::Chunk::TextureIds(ids) if ids.get(0) == Some(42))
    ));
}

#[test]
fn animation_categories_preserve_named_and_unknown_ids_through_conversion() {
    for category in [mp::AnimationId::WALK, mp::AnimationId::new(u16::MAX)] {
        let mut slot_values = SlotValues::default();
        let mut source = native_triangle();
        source
            .model
            .core_mut()
            .sequences
            .push(m2::records::Sequence {
                id: category.get(),
                timing: m2::records::LegacyTiming {
                    start: 0,
                    end: 1000,
                },
                flags: SequenceFlags::from_bits_retain(0x20),
                variation_next: -1,
                ..SlotConstruct::construct(&mut slot_values)
            });
        let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();
        let (_, clip) = imported.value.clips().iter().next().unwrap();
        assert_eq!(clip.properties.animation_id, category);
        let exported =
            io::m2::plan::<m2::Flat<m2::V256>>(&imported.value, &ExportOptions::default())
                .unwrap()
                .execute()
                .value;
        assert_eq!(exported.model.core().sequences[0].id, category.get());
    }
}

#[test]
fn shared_bone_vectors_are_imported_once_at_model_scope_with_all_clips_retained() {
    let mut source = native_triangle();
    let core = source.model.core_mut();
    core.global_sequences = ZeroVec::alloc_from_slice(&[1000]);
    for id in [0, 4] {
        core.sequences.push(m2::records::Sequence {
            id,
            timing: m2::records::LegacyTiming {
                start: 0,
                end: 1000,
            },
            flags: SequenceFlags::from_bits_retain(0x20),
            variation_next: -1,
            ..Default::default()
        });
    }
    core.bones[0].translation = Track {
        base: TrackBase {
            global_sequence: 0,
            timeline: LegacyTimeline {
                timestamps: ZeroVec::alloc_from_slice(&[0, 1000]),
                ..Default::default()
            },
            ..Default::default()
        },
        values: LegacyValues::Values(ZeroVec::alloc_from_slice(&[[1., 0., 0.], [2., 0., 0.]])),
    };
    let model = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert_eq!(model.clips().len(), 2);
    assert_eq!(model.global_tracks().count(), 1);
    let sm::DynamicTrack::Vector3(track) = model.global_tracks().next().unwrap() else {
        panic!("shared translation must remain a vector curve");
    };
    assert_eq!(
        track.curve.values().get(0),
        Some(Vec3::from_array([1., 0., 0.]))
    );
    assert_eq!(
        track.curve.values().get(1),
        Some(Vec3::from_array([2., 0., 0.]))
    );
    assert_eq!(track.curve.times().get(1).unwrap().as_seconds(), 1.);
    for (id, _) in model.clips().iter() {
        assert_eq!(model.clip_tracks(id).count(), 0);
    }
}

#[test]
fn native_empty_model_and_sequence_extents_are_explicitly_absent() {
    for limit in [10_000_000.0, f32::MAX] {
        let mut source = native_triangle();
        let empty = m2::records::Extent {
            minimum: [limit; 3],
            maximum: [-limit; 3],
            radius: 0.0,
        };
        let mut slots = SlotValues::default();
        source.model.core_mut().bounding = empty;
        source
            .model
            .core_mut()
            .sequences
            .push(m2::records::Sequence {
                bounding: empty,
                variation_next: -1,
                ..SlotConstruct::construct(&mut slots)
            });
        let imported = io::m2::import(&source, &ImportOptions::default())
            .unwrap()
            .value;
        assert_eq!(imported.bounds(), None);
        let mesh_bounds = imported.meshes().values().next().unwrap().bounds;
        assert_eq!(mesh_bounds.minimum, Vec3::ZERO);
        assert_eq!(mesh_bounds.maximum, Vec3::new(1.0, 1.0, 0.0));
        assert!((mesh_bounds.sphere_radius - 0.5f32.sqrt()).abs() < 1e-6);
        assert_eq!(
            imported.clone().widen::<sm::profiles::Universal>().bounds(),
            None
        );
        assert!(imported.clips().iter().all(|(id, _)| imported
            .clip_properties(id)
            .unwrap()
            .bounds
            .is_none()));
        let exported = io::m2::plan::<m2::Flat<m2::V256>>(&imported, &ExportOptions::default())
            .unwrap()
            .execute()
            .value;
        assert_eq!(exported.model.core().bounding.minimum, [f32::MAX; 3]);
        assert_eq!(
            exported.model.core().sequences[0].bounding.maximum,
            [-f32::MAX; 3]
        );
        assert_eq!(source.model.core().bounding.minimum, [limit; 3]);
    }
}

#[test]
fn malformed_real_extents_remain_strict_import_errors() {
    for extent in [
        m2::records::Extent {
            minimum: [2.0; 3],
            maximum: [1.0; 3],
            radius: 0.0,
        },
        m2::records::Extent {
            minimum: [f32::MAX; 3],
            maximum: [-f32::MAX; 3],
            radius: 1.0,
        },
        m2::records::Extent {
            minimum: [0.0; 3],
            maximum: [1.0; 3],
            radius: -1.0,
        },
    ] {
        let mut source = native_triangle();
        source.model.core_mut().bounding = extent;
        assert!(io::m2::import(&source, &ImportOptions::default()).is_err());
        source.model.core_mut().bounding = Default::default();
        let mut slots = SlotValues::default();
        source
            .model
            .core_mut()
            .sequences
            .push(m2::records::Sequence {
                bounding: extent,
                variation_next: -1,
                ..SlotConstruct::construct(&mut slots)
            });
        assert!(io::m2::import(&source, &ImportOptions::default()).is_err());
    }
}
