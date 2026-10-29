use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;
use wc3::model::formats::m2::{
    records::{Bone, Sequence},
    Core, V274,
};

fn validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("m2/validate");
    for count in [32, 256, 2048] {
        let mut bones = Core::<V274>::new();
        bones.bones = (0..count)
            .map(|index| {
                let mut bone = Bone::new();
                bone.parent_bone_id = index as i16 - 1;
                bone
            })
            .collect();
        group.bench_with_input(BenchmarkId::new("bone-chain", count), &bones, |b, core| {
            b.iter(|| black_box(core).validate().unwrap())
        });
        let mut aliases = Core::<V274>::new();
        aliases.sequences = (0..count)
            .map(|index| {
                let mut sequence = Sequence::default();
                if index > 0 {
                    sequence.flags.set_alias(true);
                    sequence.alias_next = index as u16 - 1;
                }
                sequence
            })
            .collect();
        group.bench_with_input(
            BenchmarkId::new("alias-chain", count),
            &aliases,
            |b, core| b.iter(|| black_box(core).validate().unwrap()),
        );
        for bone in &mut bones.bones {
            bone.parent_bone_id = -1;
        }
        group.bench_with_input(BenchmarkId::new("bone-roots", count), &bones, |b, core| {
            b.iter(|| black_box(core).validate().unwrap())
        });
        for sequence in &mut aliases.sequences {
            sequence.flags.set_alias(false);
        }
        group.bench_with_input(
            BenchmarkId::new("no-aliases", count),
            &aliases,
            |b, core| b.iter(|| black_box(core).validate().unwrap()),
        );
    }
    group.finish();
}
criterion_group!(benches, validation);
criterion_main!(benches);
