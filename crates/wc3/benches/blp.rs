use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::{env, hint::black_box};
use wc3::blp::{Blp, BlpRef, BlpVersion, EncodeFormat, EncodeOptions, IndexedAlpha};

#[path = "../../../benchmarks/support/texture.rs"]
mod fixtures;

fn textures(c: &mut Criterion) {
    let sizes = if env::var_os("WC3_BENCH_QUICK").is_some() {
        vec![256]
    } else if env::var_os("WC3_BENCH_LARGE").is_some() {
        vec![64, 256, 1024]
    } else {
        vec![64, 256]
    };
    let mut group = c.benchmark_group("blp");
    for size in sizes {
        let image = fixtures::texture(size);
        for (name, format) in [
            (
                "indexed1-opaque",
                EncodeFormat::Indexed {
                    version: BlpVersion::Blp1,
                    alpha: IndexedAlpha::Opaque,
                },
            ),
            (
                "indexed1-a1",
                EncodeFormat::Indexed {
                    version: BlpVersion::Blp1,
                    alpha: IndexedAlpha::Bit1,
                },
            ),
            (
                "indexed1-a4",
                EncodeFormat::Indexed {
                    version: BlpVersion::Blp1,
                    alpha: IndexedAlpha::Bit4,
                },
            ),
            (
                "indexed1-a8",
                EncodeFormat::Indexed {
                    version: BlpVersion::Blp1,
                    alpha: IndexedAlpha::Bit8,
                },
            ),
            (
                "indexed2-a8",
                EncodeFormat::Indexed {
                    version: BlpVersion::Blp2,
                    alpha: IndexedAlpha::Bit8,
                },
            ),
            (
                "jpeg1",
                EncodeFormat::Jpeg {
                    version: BlpVersion::Blp1,
                    alpha: true,
                    quality: 85,
                },
            ),
            (
                "jpeg2",
                EncodeFormat::Jpeg {
                    version: BlpVersion::Blp2,
                    alpha: false,
                    quality: 85,
                },
            ),
            ("dxt1", EncodeFormat::Dxt1 { alpha: true }),
            ("dxt3", EncodeFormat::Dxt3),
            ("dxt5", EncodeFormat::Dxt5),
            ("bgra", EncodeFormat::Bgra),
        ] {
            let options = EncodeOptions {
                format,
                mipmaps: true,
                ..Default::default()
            };
            let encoded = Blp::encode_image(&image, options).unwrap();
            let bytes = encoded.write().unwrap();
            let parsed = BlpRef::read(&bytes).unwrap();
            let levels = size.ilog2() as usize + 1;
            let decoded: Vec<_> = (0..levels)
                .map(|mip| parsed.decode_mip(mip).unwrap())
                .collect();
            for (level, mip) in decoded.iter().enumerate() {
                assert_eq!(
                    mip.dimensions(),
                    ((size >> level).max(1), (size >> level).max(1))
                );
            }
            if name == "bgra" {
                assert_eq!(decoded[0], image);
            }
            group.throughput(Throughput::Bytes(bytes.len() as u64));
            group.bench_function(
                BenchmarkId::new(format!("container/read/{name}"), size),
                |b| b.iter(|| black_box(BlpRef::read(black_box(&bytes)).unwrap())),
            );
            group.bench_function(
                BenchmarkId::new(format!("container/write/{name}"), size),
                |b| b.iter(|| black_box(parsed.write().unwrap())),
            );
            group.throughput(Throughput::Elements((size * size) as u64));
            group.bench_function(BenchmarkId::new(format!("decode/base/{name}"), size), |b| {
                b.iter(|| black_box(parsed.decode_mip(0).unwrap()))
            });
            let mut pixels = vec![0; (size * size * 4) as usize];
            parsed.decode_mip_into(0, &mut pixels).unwrap();
            assert_eq!(pixels, *decoded[0].as_raw());
            group.bench_function(BenchmarkId::new(format!("decode/into/{name}"), size), |b| {
                b.iter(|| {
                    parsed.decode_mip_into(0, black_box(&mut pixels)).unwrap();
                    black_box(&pixels);
                })
            });
            group.throughput(Throughput::Elements(
                decoded
                    .iter()
                    .map(|mip| (mip.width() * mip.height()) as u64)
                    .sum(),
            ));
            group.bench_function(
                BenchmarkId::new(format!("decode/chain/{name}"), size),
                |b| {
                    b.iter(|| {
                        for mip in 0..levels {
                            black_box(parsed.decode_mip(mip).unwrap());
                        }
                    })
                },
            );
            group.throughput(Throughput::Elements((size * size) as u64));
            for mipmaps in [false, true] {
                group.bench_function(
                    BenchmarkId::new(
                        format!("encode/{}/{name}", if mipmaps { "chain" } else { "base" }),
                        size,
                    ),
                    |b| {
                        b.iter(|| {
                            black_box(
                                Blp::encode_image(
                                    black_box(&image),
                                    EncodeOptions { mipmaps, ..options },
                                )
                                .unwrap(),
                            )
                        })
                    },
                );
            }
            // Supplied-level encoding separates compression from mip generation.
            group.bench_function(
                BenchmarkId::new(format!("encode/supplied/{name}"), size),
                |b| {
                    b.iter(|| black_box(Blp::encode_mipmaps(black_box(&decoded), options).unwrap()))
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, textures);
criterion_main!(benches);
