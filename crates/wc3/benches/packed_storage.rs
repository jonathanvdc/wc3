use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{Quat, Vec3};
use std::hint::black_box;
use wc3::model::common::ZeroVec;
use wc3::model::PackedArray;

fn packed_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("packed_storage");
    for count in [256, 16_384] {
        let components: ZeroVec<[f32; 3]> =
            (0..count).map(|i| [i as f32 * 0.01, 2., -3.]).collect();
        let values = PackedArray::<Vec3>::from_storage(components.as_slice().as_zerovec());
        let rotation = Quat::from_rotation_y(0.7);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("zerovec_transform", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let rotation = black_box(rotation);
                    let sum = black_box(&components)
                        .iter()
                        .map(|value| rotation * Vec3::from_array(value))
                        .sum::<Vec3>();
                    black_box(sum)
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("packed_transform", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let rotation = black_box(rotation);
                    let sum = black_box(&values)
                        .iter()
                        .map(|value| rotation * value)
                        .sum::<Vec3>();
                    black_box(sum)
                });
            },
        );
        group.bench_with_input(BenchmarkId::new("zerovec_index", count), &count, |b, _| {
            b.iter(|| {
                let values = black_box(&components);
                let mut sum = Vec3::ZERO;
                for index in 0..values.len() {
                    sum += Vec3::from_array(values.get(index).unwrap());
                }
                black_box(sum)
            });
        });
        group.bench_with_input(BenchmarkId::new("packed_index", count), &count, |b, _| {
            b.iter(|| {
                let values = black_box(&values);
                let mut sum = Vec3::ZERO;
                for index in 0..values.len() {
                    sum += values.get(index).unwrap();
                }
                black_box(sum)
            });
        });
    }
    group.finish();
}
criterion_group!(benches, packed_access);
criterion_main!(benches);
