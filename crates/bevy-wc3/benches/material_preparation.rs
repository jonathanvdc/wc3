//! CPU compilation with many material scopes and repeated tiny draws.
//!
//! Source decoding/import and image decoding/upload are excluded. Each logical
//! image request receives a distinct handle, so input-scoped ownership remains
//! visible even when all draws use the same six resource declarations.
use bevy::asset::uuid::Uuid;
use bevy::prelude::{Handle, Image};
use bevy_wc3::{compile_model, texture_requests};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::time::Duration;
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::mdlx::materials::ShaderType;
use wc3::model::formats::mdlx::{mdl, Document, V1100};
use wc3::model::io;
use wc3::model::profiles::Universal;
use wc3::model::Model;

use mdl::Read as _;

fn workload(materials: usize, draws: usize, hd: bool) -> Model<'static, Universal> {
    assert!(materials > 0 && draws >= materials);
    let mut source =
        Document::<V1100>::decode_mdl(include_str!("../tests/fixtures/hd_capture.mdl")).unwrap();
    let mut material = source.materials().next().unwrap().clone();
    if !hd {
        for layer in &mut material.layers {
            layer.set_shader_type(ShaderType::SD_LEGACY);
            layer.set_texture_slots(&[]);
        }
    }
    source.set_materials((0..materials).map(|_| material.clone()));
    let geoset = source.geosets().next().unwrap().clone();
    source.set_geosets((0..draws).map(|draw| {
        let mut geoset = geoset.clone();
        geoset.material_id = (draw % materials) as u32;
        geoset
    }));
    // Retain one static geoset appearance. Material UV and emissive channels
    // still exercise owned animation bindings on every draw.
    let animation = source.geoset_animations().next().unwrap().clone();
    source.set_geoset_animations([animation]);
    io::mdlx::import_owned(&source, &ImportOptions::default())
        .unwrap()
        .value
        .widen::<Universal>()
}

fn material_preparation(c: &mut Criterion) {
    let mut group = c.benchmark_group("material_preparation/compile");
    group.sample_size(20);
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    for (family, hd) in [("hd", true), ("classic", false)] {
        for (materials, draws) in [(1, 1), (8, 32), (32, 256)] {
            let model = workload(materials, draws, hd);
            eprintln!(
                "{family}: {materials} materials, {draws} draws, {} image requests",
                texture_requests(&model).len()
            );
            group.throughput(Throughput::Elements(draws as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("{family}_{materials}_materials"), draws),
                &model,
                |b, model| {
                    b.iter(|| {
                        let mut image = 0_u128;
                        black_box(
                            compile_model(black_box(model), |_| {
                                image += 1;
                                Some(Handle::<Image>::from(Uuid::from_u128(image)))
                            })
                            .unwrap(),
                        )
                    });
                },
            );
        }
    }
    group.finish();
}

criterion_group!(benches, material_preparation);
criterion_main!(benches);
