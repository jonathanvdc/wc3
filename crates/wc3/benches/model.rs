use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use mdl::{Read as _, Write as _};
use mdx::{Read as _, Write as _};
use std::hint::black_box;
use wc3::model::formats::mdlx::{mdl, mdx, ConversionOptions, Document, V1800, V800};

#[path = "../../../benchmarks/support/model.rs"]
mod fixtures;

fn models(c: &mut Criterion) {
    for p in fixtures::profiles() {
        let mut group = c.benchmark_group(format!("model/{}", p.name));
        let classic = fixtures::classic(p);
        let modern = fixtures::modern(p);
        macro_rules! codec {
            ($model:expr, $version:ty, $name:literal) => {{
                let model = &$model;
                let bytes = model.encode_mdx().unwrap();
                let text = model.encode_mdl().unwrap();
                assert_eq!(
                    Document::<$version>::decode_mdx(&bytes)
                        .unwrap()
                        .encode_mdx()
                        .unwrap(),
                    bytes
                );
                assert_eq!(
                    Document::<$version>::decode_mdl(&text)
                        .unwrap()
                        .encode_mdx()
                        .unwrap(),
                    bytes
                );
                group.throughput(Throughput::Bytes(bytes.len() as u64));
                group.bench_function(concat!("mdx/read/", $name), |b| {
                    b.iter(|| {
                        black_box(Document::<$version>::decode_mdx(black_box(&bytes)).unwrap())
                    })
                });
                group.bench_function(concat!("mdx/read-owned/", $name), |b| {
                    b.iter(|| {
                        black_box(
                            Document::<$version>::decode_mdx(black_box(&bytes))
                                .unwrap()
                                .into_owned(),
                        )
                    })
                });
                group.bench_function(concat!("mdx/read-traverse/", $name), |b| {
                    b.iter(|| {
                        let decoded = Document::<$version>::decode_mdx(black_box(&bytes)).unwrap();
                        for geoset in decoded.geosets() {
                            black_box((geoset.vertices(), geoset.face_indices(), geoset.uv_sets()));
                        }
                        black_box(decoded.nodes());
                    })
                });
                group.bench_function(concat!("mdx/write/", $name), |b| {
                    b.iter(|| black_box(model.encode_mdx().unwrap()))
                });
                group.throughput(Throughput::Bytes(text.len() as u64));
                group.bench_function(concat!("mdl/read/", $name), |b| {
                    b.iter(|| {
                        black_box(Document::<$version>::decode_mdl(black_box(&text)).unwrap())
                    })
                });
                group.bench_function(concat!("mdl/write/", $name), |b| {
                    b.iter(|| black_box(model.encode_mdl().unwrap()))
                });
            }};
        }
        codec!(classic, V800, "800");
        codec!(modern, V1800, "1800");
        group.throughput(Throughput::Elements((p.geosets * p.vertices) as u64));
        group.bench_function("geometry/access", |b| {
            b.iter(|| {
                for g in modern.geosets() {
                    black_box((
                        g.vertices(),
                        g.normals(),
                        g.face_indices(),
                        g.try_skin_weights().unwrap(),
                    ));
                }
            })
        });
        group.throughput(Throughput::Elements(1));
        group.bench_function("view/borrow", |b| {
            b.iter(|| black_box(modern.as_borrowed()))
        });
        group.bench_function("view/clone", |b| b.iter(|| black_box(modern.clone())));
        group.bench_function("view/to-owned", |b| b.iter(|| black_box(modern.to_owned())));
        group.bench_function("resources", |b| {
            b.iter(|| black_box(modern.resources().collect::<Vec<_>>()))
        });
        group.bench_with_input(
            BenchmarkId::new("convert", "800-1800"),
            &classic,
            |b, model| {
                b.iter(|| {
                    black_box(
                        model
                            .convert::<V1800>(&ConversionOptions::strict())
                            .unwrap(),
                    )
                })
            },
        );
        group.bench_function("convert-owned/800-1800", |b| {
            b.iter_batched(
                || classic.clone(),
                |model| {
                    black_box(
                        model
                            .into_convert::<V1800>(&ConversionOptions::strict())
                            .unwrap(),
                    )
                },
                BatchSize::SmallInput,
            )
        });
        group.finish();
    }
}

criterion_group!(benches, models);
criterion_main!(benches);
