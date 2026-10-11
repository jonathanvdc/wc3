use bevy_asset::io::{AssetReader, AssetReaderError, Reader};
use bevy_mpq::{MpqAssetReader, OverlayAssetReader, OverlayMount};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use futures_lite::future::{block_on, poll_fn};
use std::{
    env, future::Future, hint::black_box, num::NonZeroUsize, path::Path, sync::Arc, task::Poll,
};
use wc3::mpq::{Compression, SharedArchive};

#[path = "../../../benchmarks/support/archive.rs"]
mod fixtures;

async fn consume(mut reader: impl Reader) -> Vec<u8> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await.unwrap();
    bytes
}

async fn batch(reader: &MpqAssetReader<Arc<[u8]>>, paths: &[String]) -> Vec<Vec<u8>> {
    let mut pending: Vec<_> = paths
        .iter()
        .map(|path| {
            Box::pin(async move { consume(reader.read(Path::new(path)).await.unwrap()).await })
        })
        .collect();
    let mut output: Vec<Option<Vec<u8>>> = (0..paths.len()).map(|_| None).collect();
    poll_fn(|cx| {
        for (future, slot) in pending.iter_mut().zip(&mut output) {
            if slot.is_none() {
                if let Poll::Ready(bytes) = future.as_mut().poll(cx) {
                    *slot = Some(bytes);
                }
            }
        }
        if output.iter().all(Option::is_some) {
            Poll::Ready(output.iter_mut().map(|slot| slot.take().unwrap()).collect())
        } else {
            Poll::Pending
        }
    })
    .await
}

fn assets(c: &mut Criterion) {
    let payload = fixtures::payload(65536, false);
    let bytes: Arc<[u8]> = fixtures::archive(&payload, 16, Compression::Zlib, true, 0).into();
    let shared = SharedArchive::open(bytes).unwrap();
    let reader = MpqAssetReader::new(shared.clone(), "synthetic");
    assert_eq!(
        block_on(consume(
            block_on(reader.read(Path::new("entry-0"))).unwrap()
        )),
        payload
    );
    let mut group = c.benchmark_group("assets");
    group.throughput(Throughput::Bytes(payload.len() as u64));
    group.bench_function("direct/shared", |b| {
        b.iter(|| black_box(shared.read_file("entry-0").unwrap()))
    });
    group.bench_function("adapter/single", |b| {
        b.iter(|| {
            black_box(block_on(async {
                consume(reader.read(Path::new("entry-0")).await.unwrap()).await
            }))
        })
    });
    let quick = env::var_os("WC3_BENCH_QUICK").is_some();
    for limit in if quick { vec![1, 4] } else { vec![1, 2, 4, 8] } {
        let reader = reader
            .clone()
            .with_max_concurrent_reads(NonZeroUsize::new(limit).unwrap());
        for width in if quick { vec![1, 16] } else { vec![1, 4, 16] } {
            let paths: Vec<_> = (0..width).map(|i| format!("entry-{i}")).collect();
            assert!(block_on(batch(&reader, &paths))
                .iter()
                .all(|bytes| bytes == &payload));
            group.throughput(Throughput::Bytes((payload.len() * width) as u64));
            group.bench_function(
                BenchmarkId::new(format!("batch/limit-{limit}"), width),
                |b| b.iter(|| black_box(block_on(batch(&reader, &paths)))),
            );
        }
    }
    let empty: Arc<[u8]> = fixtures::archive(&[], 0, Compression::Stored, false, 0).into();
    let empty = SharedArchive::open(empty).unwrap();
    for depth in if quick { vec![1, 16] } else { vec![1, 4, 16] } {
        let mounts = (0..depth)
            .map(|i| {
                OverlayMount::mpq(MpqAssetReader::new(
                    if i + 1 == depth {
                        shared.clone()
                    } else {
                        empty.clone()
                    },
                    "overlay",
                ))
            })
            .collect();
        let overlay = OverlayAssetReader::new(mounts);
        assert_eq!(
            block_on(async { consume(overlay.read(Path::new("entry-0")).await.unwrap()).await }),
            payload
        );
        group.throughput(Throughput::Bytes(payload.len() as u64));
        group.bench_function(BenchmarkId::new("overlay/last-hit", depth), |b| {
            b.iter(|| {
                black_box(block_on(async {
                    consume(overlay.read(Path::new("entry-0")).await.unwrap()).await
                }))
            })
        });
        group.throughput(Throughput::Elements(1));
        assert!(matches!(
            block_on(overlay.read(Path::new("missing"))),
            Err(AssetReaderError::NotFound(_))
        ));
        group.bench_function(BenchmarkId::new("overlay/miss", depth), |b| {
            b.iter(|| {
                black_box(block_on(overlay.read(Path::new("missing"))).err().unwrap());
            })
        });
    }
    group.finish();
}

criterion_group!(benches, assets);
criterion_main!(benches);
