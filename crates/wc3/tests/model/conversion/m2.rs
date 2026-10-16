use std::borrow::Cow;
use wc3::model::common::{IntoZeroVec, ZeroVec};
use wc3::model::io;
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
#[test]
fn classic_geometry_becomes_editable_and_exports_modern_skin() {
    let source = native_triangle();
    let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();
    let semantic = &imported.value;
    let mesh = semantic.meshes().next().unwrap().1;
    assert!(mesh.positions().is_owned());
    assert_eq!(mesh.positions().get(1), Some([1., 0., 0.]));
    let sm::Topology::Triangles(triangles) = mesh.topology() else {
        panic!("triangle topology lost")
    };
    assert!(triangles.is_owned());
    assert_eq!(triangles.get(0), Some([0, 1, 2]));
    assert_eq!(mesh.influences().joint_weights(0).unwrap()[0].weight, 1.);
    let texture = imported.value.resources().textures().next().unwrap().1;
    assert!(
        matches!(&texture.resource,sm::Resource::Path(Cow::Borrowed(path))if *path==b"textures/checker.blp")
    );
    let target = io::m2::plan::<m2::Chunked<m2::V274>>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
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
    let clip = model.clips().next().unwrap().1;
    let sm::Channel::Vector3(track) = &clip.channels()[0] else {
        panic!("translation channel lost")
    };
    assert_eq!(track.curve.interpolation(), sm::Interpolation::Hermite);
    assert!(track.curve.values().is_owned());
    assert_eq!(track.curve.times().get(0).unwrap().as_seconds(), 0.);
    assert_eq!(track.curve.times().get(1).unwrap().as_seconds(), 1.);
    assert_eq!(
        track.curve.hermite_derivatives().unwrap().incoming.get(1),
        Some([3., 0., 0.])
    );
    let exported = io::m2::plan::<m2::Flat<m2::V256>>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
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
    let (id, mesh) = asset.meshes().next().unwrap();
    let mut mesh = mesh.clone();
    let mut positions: Vec<_> = mesh.positions().iter().collect();
    positions[1] = [5., 0., 0.];
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
        (positions).into_zero_vec(),
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
        .unwrap()
        .value;
    assert_eq!(
        output.model.core().vertices.get(1).unwrap().position,
        [5., 0., 0.]
    );
}
#[test]
fn quantization_requires_an_explicit_tolerance() {
    let mut builder = sm::Model::<sm::profiles::M2>::builder("quantized");
    let node = builder
        .add_node(sm::Node::named("bone").with_controls(mp::BoneControls {
            key_bone: -1,
            flags: mp::values::BoneBehaviorFlags::default(),
            submesh: 0,
            name_crc: 0,
        }))
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
        .add_channel(
            clip,
            sm::Channel::Vector3(sm::Track {
                target: node.translation(),
                curve,
                clock: sm::Clock::Clip,
            }),
        )
        .unwrap();
    let asset = builder.finish().unwrap();
    assert!(io::m2::plan::<m2::Flat<m2::V264>>(&asset, &ExportOptions::default()).is_err());
    let plan = io::m2::plan::<m2::Flat<m2::V264>>(
        &asset,
        &ExportOptions {
            approximation: ApproximationPolicy::Allow { tolerance: 0.0005 },
            ..Default::default()
        },
    )
    .unwrap();
    assert!(!plan.report().issues.is_empty());
}
#[test]
fn unresolved_companions_are_errors() {
    let mut model = m2::Document::<m2::Flat<m2::V264>>::new();
    model.core_mut().skins = 1;
    assert!(
        io::m2::import(
            &m2::Bundle::from_model(model),
            &ImportOptions {
                loss_policy: LossPolicy::Drop,
                ..Default::default()
            }
        )
        .is_err()
    );
}
#[test]
fn native_unknown_extensions_are_not_silently_reinserted() {
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
        ..Default::default()
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
        ..Default::default()
    });
    core.collision_vertices =
        ZeroVec::alloc_from_slice(&[[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]);
    core.collision_indices = ZeroVec::alloc_from_slice(&[0, 1, 2]);
    let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();

    let model = &imported.value;
    assert_eq!(
        model
            .features()
            .filter(|(_, f)| matches!(
                f.payload,
                mp::ActorKind::Attachment(_)
                    | mp::ActorKind::Light(_)
                    | mp::ActorKind::Ribbon(_)
                    | mp::ActorKind::Particle(_)
                    | mp::ActorKind::Camera(_)
                    | mp::ActorKind::Event(_)
            ))
            .count(),
        3
    );
    assert!(
        model
            .systems()
            .any(|(_, system)| matches!(system, mp::System::Collision(_)))
    );
    let exported = io::m2::plan::<m2::Flat<m2::V256>>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
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
    let (id, material) = asset.materials().next().unwrap();
    let mut surface = material.surface.clone();
    surface.shader = 42;
    asset
        .edit(|draft| draft.set_material_surface(id, surface))
        .unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(output.model.core().skins[0].batches[0].shader_id, 42);
}
#[test]
fn unknown_extensions_require_explicit_drop_policy() {
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
            ..Default::default()
        },
    )
    .unwrap();

    assert!(
        imported
            .report
            .issues
            .iter()
            .any(|issue| issue.path == "m2.chunk" && issue.kind == conversion::IssueKind::Dropped)
    );
    let output = io::m2::plan::<m2::Chunked<m2::V256>>(&imported.value, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap();
    assert!(
        !output
            .value
            .model
            .extensions()
            .any(|chunk| chunk.tag() == *b"TEST")
    );
}
#[test]
fn compressed_rotations_decode_to_unit_components() {
    use io::m2::RotationComponents as _;
    assert_eq!([-32768i16, -32768, -32768, -1].floats(), [0., 0., 0., 1.]);
    assert_eq!([0i16, 0, 0, 32767].floats(), [-1., -1., -1., 0.]);
}

#[test]
fn native_physics_shapes_and_body_offsets_are_editable() {
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
                    ..Default::default()
                },
                ..Default::default()
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
                ..Default::default()
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
        .find(|(_, s)| matches!(s, mp::System::Physics(_)))
        .unwrap();
    let mut system = system.clone();
    let mp::System::Physics(physics) = &mut system else {
        unreachable!()
    };
    let shape = &mut physics.bodies[0].shapes[0];
    assert!(matches!(
        shape.geometry,
        mp::ShapeGeometry::Box([1., 2., 3.])
    ));
    shape.geometry = mp::ShapeGeometry::Box([7., 8., 9.]);
    asset
        .edit(|draft| draft.set_system(system_id, system))
        .unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
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
        .find(|(_, s)| matches!(s, mp::System::BoneOverrides(_)))
        .unwrap();
    let mut system = system.clone();
    let mp::System::BoneOverrides(overrides) = &mut system else {
        unreachable!()
    };
    let matrices = &mut overrides[0].transforms;
    assert!(matrices.is_owned());
    let mut matrix = matrices.get(0).unwrap();
    assert_eq!(matrix[3][0], 3.);
    matrix[3][0] = 9.;
    *matrices = (vec![matrix]).into_zero_vec();
    asset
        .edit(|draft| draft.set_system(system_id, system))
        .unwrap();
    let output = io::m2::plan::<m2::Chunked<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
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
    let sm::Channel::Vector3(track)=semantic.shared_channels().iter().find(|c|matches!(c,sm::Channel::Vector3(t)if matches!(t.target.target(),sm::PropertyTarget::Profile{key:mp::properties::Target::Particle(mp::properties::ParticleField::Gravity),..}))).unwrap() else{panic!("gravity channel lost")};
    let gravity = track.curve.values().get(0).unwrap();
    assert!(gravity[0] > 1.);
    assert_eq!(gravity[1], 0.);
    assert!(gravity[2] > 4.);
    let options = ExportOptions {
        approximation: ApproximationPolicy::Allow { tolerance: 0.00001 },
        ..Default::default()
    };
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &options)
        .unwrap()
        .execute()
        .unwrap()
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
        .find(|(_, node)| node.controls.is_some())
        .unwrap()
        .0;
    let mut node = asset.node(id).unwrap().clone();
    node.transform.translation = [2., 3., 4.];
    asset.edit(|draft| draft.set_node(id, node)).unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
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
    let mut source = native_triangle();
    source.model.core_mut().lights.push(m2::records::Light {
        kind: LightKind::Point,
        bone_id: 0,
        position: [1., 2., 3.],
        ..Default::default()
    });
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let node = asset
        .features()
        .find(|(_, f)| matches!(f.payload, mp::ActorKind::Light(_)))
        .unwrap()
        .1
        .node;
    asset
        .edit(|draft| {
            let clock = draft.add_clock(sm::SharedClock {
                period: sm::Time::seconds(1.)?,
            })?;
            let curve = sm::Curve::step(
                (vec![sm::Time::default()]).into_zero_vec(),
                (vec![[5., 0., 0.]]).into_zero_vec(),
            )?;
            draft.add_shared_channel(sm::Channel::Vector3(sm::Track {
                target: node.translation(),
                curve,
                clock: sm::Clock::Shared(clock),
            }))
        })
        .unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
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
    let surface = &asset.materials().next().unwrap().1.surface;
    assert!(surface.state.blend == mp::values::BlendMode::Unknown(8));
    let id = asset.nodes().next().unwrap().0;
    let mut node = asset.node(id).unwrap().clone();
    node.name = Cow::Borrowed("edited");
    asset.edit(|draft| draft.set_node(id, node)).unwrap();
    let output = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
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
        .unwrap()
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
fn texture_options_keep_unknown_bits_and_reject_unencoded_interpretations() {
    let mut source = native_triangle();
    source.model.core_mut().textures[0].flags = TextureFlags::from_bits_retain(0x8000_0001);
    let mut asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let id = asset.resources().textures().next().unwrap().0;
    let texture = asset.resources().texture(id).unwrap();
    assert_eq!(texture.controls.flags.bits(), 0x8000_0000);
    assert_eq!(texture.color_space, sm::ColorSpace::Unspecified);
    let native = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    assert_eq!(native.model.core().textures[0].flags.bits(), 0x8000_0001);
    let original = texture.sampler;
    assert!(
        asset
            .edit_resources(|draft| {
                let mut value = draft.texture(id).unwrap().clone();
                value.sampler.u = sm::AddressMode::Mirror;
                draft.set_texture(id, value)
            })
            .is_err()
    );
    assert_eq!(asset.resources().texture(id).unwrap().sampler, original);
    assert!(
        asset
            .edit_resources(|draft| {
                let mut value = draft.texture(id).unwrap().clone();
                value.color_space = sm::ColorSpace::Srgb;
                draft.set_texture(id, value)
            })
            .is_err()
    );
}

#[test]
fn replacement_texture_retains_its_separately_declared_filename() {
    let mut source = native_triangle();
    source.model.core_mut().textures[0].kind = TextureSource::Unknown(0x1234);
    let asset = io::m2::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let texture = asset.resources().textures().next().unwrap().1;
    assert_eq!(texture.resource, sm::Resource::Replaceable(0x1234));
    assert!(matches!(texture.controls.path, Some(Cow::Borrowed(_))));
    assert_eq!(
        texture.controls.path.as_deref(),
        Some(b"textures/checker.blp".as_slice())
    );
    let native = io::m2::plan::<m2::Flat<m2::V256>>(&asset, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let texture = &native.model.core().textures[0];
    assert_eq!(texture.kind, TextureSource::Unknown(0x1234));
    assert_eq!(texture.filename.as_bytes(), b"textures/checker.blp\0");
}

#[test]
fn native_alias_keeps_light_programme_on_its_referenced_clip() {
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
            ..Default::default()
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
            ..Default::default()
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
        ..Default::default()
    });
    let imported = io::m2::import(&source, &ImportOptions::default()).unwrap();
    let model = &imported.value;
    let clips = model.clips().collect::<Vec<_>>();
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[1].1.alias(), Some(clips[0].0));
    assert!(clips[1].1.channels().is_empty());
    assert!(!clips[0].1.channels().is_empty());
    let program = sm::AnimationProgram::compile(model);
    let sample = program
        .sample(sm::AnimationSample {
            clip: Some(clips[1].0),
            clip_time: sm::Time::seconds(0.5).unwrap(),
            ..Default::default()
        })
        .unwrap();
    let sm::Channel::Scalar(track) = &clips[0].1.channels()[0] else {
        panic!("light intensity must remain a typed scalar channel")
    };
    assert_eq!(sample.property(&track.target), Some(4.));
}
