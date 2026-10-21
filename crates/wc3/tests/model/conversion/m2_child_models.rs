use wc3::model::common::ZeroVec;
use wc3::model::conversion::{ImportOptions, LossPolicy};
use wc3::model::formats::m2::chunks::Chunk;
use wc3::model::formats::m2::records::{Bone, ModelFlags, ParticleEmitter, Texture, TextureSource};
use wc3::model::formats::m2::{
    self, Chunked, Document, DynamicDocument, File, FileKey, Flat, V256,
};
use wc3::model::io::m2 as adapter;
use wc3::model::profiles::{self, m2 as profile};
use wc3::model::{M2ModelResource, ResourceLocator};

fn source() -> m2::Bundle<'static, Chunked<V256>> {
    let mut document = Document::<Chunked<V256>>::new();
    let core = document.core_mut();
    core.name = ZeroVec::alloc_from_slice(b"modern particle models\0");
    core.bones = vec![Bone::new()];
    core.textures = vec![Texture {
        kind: TextureSource::File,
        filename: ZeroVec::alloc_from_slice(b"white.blp\0"),
        ..Default::default()
    }];
    let mut first = ParticleEmitter::<V256>::new(ModelFlags::default());
    first.particle_model_filename = ZeroVec::alloc_from_slice(b"ignored.m2\0");
    let mut second = ParticleEmitter::<V256>::new(ModelFlags::default());
    second.particle_model_filename = ZeroVec::alloc_from_slice(b"legacy.m2\0");
    core.particles = vec![first, second];
    document
        .insert_extension(
            0,
            Chunk::GeometryParticleIds(ZeroVec::alloc_from_slice(&[11, 0])),
        )
        .unwrap();
    document
        .insert_extension(
            0,
            Chunk::RecursiveParticleIds(ZeroVec::alloc_from_slice(&[22, 22])),
        )
        .unwrap();
    let mut source = m2::Bundle::from_model(document);
    let mut child = Document::<Flat<V256>>::new();
    child.core_mut().name = ZeroVec::alloc_from_slice(b"child\0");
    for key in [
        FileKey::FileId(11),
        FileKey::FileId(22),
        FileKey::Path("legacy.m2".into()),
    ] {
        source.insert(
            key,
            File::Document(Box::new(DynamicDocument::Flat256(child.clone()))),
        );
    }
    source
}
#[test]
fn chunked_model_ids_map_both_roles_without_disabling_legacy_fallback() {
    let source = source();
    let imported = adapter::import(&source, &ImportOptions::default()).unwrap();
    assert!(imported.report.issues.is_empty());
    let declarations: Vec<_> = imported
        .value
        .features()
        .iter()
        .filter_map(|(_, feature)| match &feature.payload {
            profile::ActorKind::Particle(value) => Some((
                value.details.rendering.model.unwrap(),
                value.details.rendering.child_model.unwrap(),
            )),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 2);
    assert_eq!(
        imported.value.model_resource(declarations[0].0),
        Some(&M2ModelResource::FileDataId(11))
    );
    assert_eq!(
        imported.value.model_resource(declarations[0].1),
        Some(&M2ModelResource::FileDataId(22))
    );
    assert!(
        matches!(imported.value.model_resource(declarations[1].0), Some(M2ModelResource::Path(path)) if path.as_ref()==b"legacy.m2")
    );
    assert!(!imported.value.dependencies().models().iter().any(|(_, resource)| matches!(resource, M2ModelResource::Path(path) if path.as_ref()==b"ignored.m2")));
    let universal = imported.value.widen::<profiles::Universal>();
    assert_eq!(
        universal.model_resource(declarations[0].0),
        Some(&ResourceLocator::FileDataId(11))
    );
    assert_eq!(
        universal.model_resource(declarations[1].1),
        Some(&ResourceLocator::FileDataId(22))
    );
    universal.try_narrow::<profiles::M2>().unwrap();
}
#[test]
fn unrelated_extensions_keep_their_existing_checked_loss_boundary() {
    let mut source = source();
    source.model.insert_extension(0, Chunk::Waterfall1).unwrap();
    assert!(adapter::import(&source, &ImportOptions::default()).is_err());
    let imported = adapter::import(
        &source,
        &ImportOptions {
            loss_policy: LossPolicy::Drop,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(imported
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "m2.extensions"));
    assert!(imported
        .value
        .dependencies()
        .models()
        .iter()
        .any(|(_, resource)| *resource == M2ModelResource::FileDataId(11)));
}
#[test]
fn duplicate_or_misaligned_model_id_tables_are_rejected() {
    let mut duplicate = source();
    duplicate
        .model
        .insert_extension(
            0,
            Chunk::GeometryParticleIds(ZeroVec::alloc_from_slice(&[11, 0])),
        )
        .unwrap();
    assert!(adapter::import(&duplicate, &ImportOptions::default())
        .unwrap_err()
        .to_string()
        .contains("ambiguous"));
    let mut misaligned = source();
    misaligned.model.remove_extensions(*b"GPID");
    misaligned
        .model
        .insert_extension(
            0,
            Chunk::GeometryParticleIds(ZeroVec::alloc_from_slice(&[11])),
        )
        .unwrap();
    assert!(adapter::import(&misaligned, &ImportOptions::default())
        .unwrap_err()
        .to_string()
        .contains("align"));
}

fn bone_companion(bone: u16, x: f32) -> File<'static> {
    use wc3::model::formats::m2::companions::{BoneFile, BoneMatrix};
    let mut values = [0.; 16];
    for i in 0..4 {
        values[i * 5] = 1.;
    }
    values[12] = x;
    File::Bone(BoneFile {
        version: 1,
        indices: ZeroVec::alloc_from_slice(&[bone]),
        matrices: vec![BoneMatrix { values }],
    })
}
#[test]
fn child_bone_companions_are_not_applied_to_the_parent_graph() {
    let mut source = source();
    let mut child = Document::<Chunked<V256>>::new();
    child.core_mut().name = ZeroVec::alloc_from_slice(b"child with distinct bones\0");
    child.core_mut().bones = vec![Bone::new(), Bone::new()];
    child
        .insert_extension(0, Chunk::BoneIds(ZeroVec::alloc_from_slice(&[201])))
        .unwrap();
    source.insert(
        FileKey::FileId(11),
        File::Document(Box::new(DynamicDocument::Chunked256(child))),
    );
    source.insert(FileKey::FileId(201), bone_companion(1, 9.));
    // Native references are valid in the child, but bone 1 does not exist in the parent.
    source.validate_complete().unwrap();
    let imported = adapter::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert!(!imported
        .systems()
        .iter()
        .any(|(_, system)| matches!(system, profile::System::BoneOverrides(_))));
}
#[test]
fn root_declared_and_optional_bone_companions_remain_in_the_root_graph() {
    let mut source = source();
    let mut child = Document::<Chunked<V256>>::new();
    child.core_mut().name = ZeroVec::alloc_from_slice(b"child sharing bone companion\0");
    child.core_mut().bones = vec![Bone::new()];
    child
        .insert_extension(0, Chunk::BoneIds(ZeroVec::alloc_from_slice(&[301])))
        .unwrap();
    source.insert(
        FileKey::FileId(11),
        File::Document(Box::new(DynamicDocument::Chunked256(child))),
    );
    source
        .model
        .insert_extension(0, Chunk::BoneIds(ZeroVec::alloc_from_slice(&[301])))
        .unwrap();
    source.insert(FileKey::FileId(301), bone_companion(0, 3.));
    source.insert(FileKey::Path("optional.bone".into()), bone_companion(0, 4.));
    let imported = adapter::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    let overrides: Vec<_> = imported
        .systems()
        .iter()
        .filter_map(|(_, system)| match system {
            profile::System::BoneOverrides(values) => Some(values),
            _ => None,
        })
        .flat_map(|values| values.iter())
        .collect();
    assert_eq!(overrides.len(), 2);
    let mut x: Vec<_> = overrides
        .iter()
        .map(|value| value.transforms.get(0).unwrap().w_axis.x)
        .collect();
    x.sort_by(f32::total_cmp);
    assert_eq!(x, [3., 4.]);
}

fn physics_companion(bone: u16) -> File<'static> {
    use std::borrow::Cow;
    use wc3::model::formats::m2::companions::{
        PhysicsBody4, PhysicsBox, PhysicsChunk, PhysicsFile, PhysicsFrame, PhysicsShape,
        PhysicsShape2,
    };
    use wc3::model::formats::m2::records::{PhysicsBodyKind, PhysicsKind, PhysicsShapeKind};
    use wc3::model::{Construct, PropertyValues};
    let mut values = PropertyValues::default();
    File::Physics(PhysicsFile {
        chunks: vec![
            PhysicsChunk::Header(6),
            PhysicsChunk::Kind(PhysicsKind::WornItem),
            PhysicsChunk::Boxes(vec![PhysicsBox {
                frame: PhysicsFrame {
                    axis_x: [1., 0., 0.],
                    axis_y: [0., 1., 0.],
                    axis_z: [0., 0., 1.],
                    origin: [0.; 3],
                },
                half_extents: [1.; 3],
            }]),
            PhysicsChunk::Shapes2(vec![PhysicsShape2 {
                shape: PhysicsShape {
                    shape_type: PhysicsShapeKind::Box,
                    shape_index: 0,
                    ..Construct::construct(&mut values)
                },
                ..Construct::construct(&mut values)
            }]),
            PhysicsChunk::Bodies4(vec![PhysicsBody4 {
                body_type: PhysicsBodyKind::Kinematic,
                bone_index: bone,
                shape_index: 0,
                shape_count: 1,
                ..Construct::construct(&mut values)
            }]),
        ],
        padding: Cow::Owned(Vec::new()),
    })
}
#[test]
fn child_physics_companions_are_not_applied_to_the_parent_graph() {
    let mut source = source();
    let mut child = Document::<Chunked<V256>>::new();
    child.core_mut().name = ZeroVec::alloc_from_slice(b"child with physics\0");
    child.core_mut().bones = vec![Bone::new(), Bone::new()];
    child.core_mut().flags.set_external_physics(true);
    child.insert_extension(0, Chunk::PhysicsId(202)).unwrap();
    source.insert(
        FileKey::FileId(11),
        File::Document(Box::new(DynamicDocument::Chunked256(child))),
    );
    source.insert(FileKey::FileId(202), physics_companion(1));
    source.validate_complete().unwrap();
    let imported = adapter::import(&source, &ImportOptions::default())
        .unwrap()
        .value;
    assert!(!imported
        .systems()
        .iter()
        .any(|(_, system)| matches!(system, profile::System::Physics(_))));
}
