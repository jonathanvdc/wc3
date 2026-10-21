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
