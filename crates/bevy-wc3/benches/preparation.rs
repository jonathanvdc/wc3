use bevy::{mesh::skinning::SkinnedMeshInverseBindposes, prelude::*};
use bevy_wc3::{prepare_model, Wc3Model, Wc3PoseBaker, Wc3PoseOptions, Wc3TextureBindings};
use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use mdl::Write as _;
use mdx::Write as _;
use std::hint::black_box;
use wc3::model::formats::mdlx::{mdl, mdx};

#[path = "../../../benchmarks/support/model.rs"]
mod fixtures;

fn preparation(c: &mut Criterion) {
    for profile in fixtures::profiles() {
        let model = fixtures::modern(profile);
        let bytes = model.encode_mdx().unwrap();
        let text = model.encode_mdl().unwrap();
        let source = Wc3Model::decode_mdx(&bytes).unwrap();
        let mut group = c.benchmark_group(format!("preparation/{}", profile.name));
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_function("decode/mdx", |b| {
            b.iter(|| black_box(Wc3Model::decode_mdx(black_box(&bytes)).unwrap()))
        });
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_function("decode/mdl", |b| {
            b.iter(|| black_box(Wc3Model::decode_mdl(black_box(&text)).unwrap()))
        });
        group.throughput(Throughput::Elements(
            (profile.geosets * profile.vertices) as u64,
        ));
        group.bench_function("assets", |b| {
            b.iter_batched(
                || {
                    (
                        Assets::<Mesh>::default(),
                        Assets::<SkinnedMeshInverseBindposes>::default(),
                    )
                },
                |(mut meshes, mut bindposes)| {
                    let prepared =
                        prepare_model(&mut meshes, &mut bindposes, black_box(&source), |_| None)
                            .unwrap();
                    black_box((prepared, meshes, bindposes))
                },
                BatchSize::LargeInput,
            )
        });
        let mut meshes = Assets::<Mesh>::default();
        let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
        let prepared = prepare_model(&mut meshes, &mut bindposes, &source, |_| None).unwrap();
        assert_eq!(prepared.parts().count(), profile.geosets * profile.layers);
        let mut images = Assets::<Image>::default();
        let mut baker = Wc3PoseBaker::default();
        let bindings = Wc3TextureBindings::default();
        let options = Wc3PoseOptions {
            sequence: Some(0),
            elapsed_ms: 750.0,
            global_elapsed_ms: 750.0,
            ..default()
        };
        let pose = baker
            .bake(&prepared, &meshes, &mut images, &bindings, options)
            .unwrap();
        assert_eq!(pose.parts.len(), prepared.parts().count());
        assert!(pose.bounds.is_some());
        group.bench_function("pose/bake", |b| {
            b.iter(|| {
                black_box(
                    baker
                        .bake(
                            &prepared,
                            &meshes,
                            &mut images,
                            &bindings,
                            black_box(options),
                        )
                        .unwrap(),
                )
            })
        });
        group.finish();
    }
}

criterion_group!(benches, preparation);
criterion_main!(benches);
