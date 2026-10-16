use bevy::{mesh::skinning::SkinnedMeshInverseBindposes, prelude::*};
use bevy_wc3::compile_model;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use mdl::Write as _;
use mdx::Write as _;
use std::hint::black_box;
use wc3::model::formats::mdlx::{mdl, mdx};

#[path = "../../../benchmarks/support/model.rs"]
mod fixtures;
#[path = "../examples/support/model.rs"]
mod semantic;

fn preparation(c: &mut Criterion) {
    for profile in fixtures::profiles() {
        let model = fixtures::modern(profile);
        let bytes = model.encode_mdx().unwrap();
        let text = model.encode_mdl().unwrap();
        let source = semantic::decode(&bytes).unwrap();
        let mut group = c.benchmark_group(format!("preparation/{}", profile.name));
        group.throughput(Throughput::Bytes(bytes.len() as u64));
        group.bench_function("decode/mdx", |b| {
            b.iter(|| black_box(semantic::decode(black_box(&bytes)).unwrap()))
        });
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_function("decode/mdl", |b| {
            b.iter(|| black_box(semantic::decode(black_box(text.as_bytes())).unwrap()))
        });
        group.throughput(Throughput::Elements(
            (profile.geosets * profile.vertices) as u64,
        ));
        group.bench_function("compile", |b| {
            b.iter(|| black_box(compile_model(black_box(&source), |_| None).unwrap()))
        });
        let compiled = compile_model(&source, |_| None).unwrap();
        group.bench_function("assets", |b| {
            b.iter_batched(
                || {
                    (
                        Assets::<Mesh>::default(),
                        Assets::<SkinnedMeshInverseBindposes>::default(),
                    )
                },
                |(mut meshes, mut bindposes)| {
                    let prepared = compiled.prepare(&mut meshes, &mut bindposes);
                    black_box((prepared, meshes, bindposes))
                },
                BatchSize::LargeInput,
            )
        });
        group.finish();
    }
}
criterion_group!(benches, preparation);
criterion_main!(benches);
