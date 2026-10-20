//! Explicit opt-in native fixtures for the shared offscreen capture example.
use std::fs::{create_dir_all, write};
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::m2::{
    Bundle as NativeM2Bundle, Flat as NativeM2Flat, V256 as NativeM2V256,
};
use wc3::model::io::m2 as semantic_m2;

use wc3::model::TimeSource;

fn gravity_fixture(
    packed: bool,
    gravity: u32,
) -> NativeM2Bundle<'static, NativeM2Flat<NativeM2V256>> {
    use wc3::model::common::ZeroVec;
    use wc3::model::formats::m2::codec::Value;
    use wc3::model::formats::m2::records::{
        Bone, LegacyParticleLifetime, LegacyTimeline, LegacyValues, ModelFlags, ParticleBlend,
        ParticleEmitter, ParticleGenerator, Texture, TextureSource, Track, TrackBase,
    };
    use wc3::model::formats::m2::{Bundle, Document, Flat, V256};

    fn constant<T: Value>(value: T) -> Track<'static, T, V256>
    where
        [T; 3]: Value,
    {
        Track {
            base: TrackBase {
                global_sequence: 0,
                timeline: LegacyTimeline {
                    timestamps: ZeroVec::alloc_from_slice(&[0]),
                    ..Default::default()
                },
                ..Default::default()
            },
            values: LegacyValues::Values(ZeroVec::alloc_from_slice(&[value])),
        }
    }

    let mut document = Document::<Flat<V256>>::new();
    let core = document.core_mut();
    core.name = ZeroVec::alloc_from_slice(b"gravity\0");
    core.global_sequences = ZeroVec::alloc_from_slice(&[1000]);
    core.bones = vec![Bone::new()];
    core.textures = vec![Texture {
        kind: TextureSource::File,
        filename: ZeroVec::alloc_from_slice(b"atlas.tga\0"),
        ..Default::default()
    }];
    let mut emitter = ParticleEmitter::<V256>::new(ModelFlags::default());
    emitter.bone_id = 0;
    emitter.twinkle_percent = 1.;
    emitter.twinkle_scale = [1.; 2];
    emitter.flags.set_packed_gravity(packed);
    emitter.kind.blending = ParticleBlend::Alpha;
    emitter.kind.emitter = ParticleGenerator::Plane;
    emitter.emission_speed = constant(0.2);
    emitter.lifespan = constant(2.);
    emitter.emission_rate = constant(12.);
    emitter.gravity = constant(gravity);
    emitter.enabled = constant(1);
    emitter.lifetime = LegacyParticleLifetime {
        midpoint: 0.5,
        colors_bgra: [[255, 255, 255, 255]; 3],
        scales: [0.12; 3],
        ..Default::default()
    };
    core.particles.push(emitter);
    Bundle::from_model(document)
}

#[test]
fn native_packed_gravity_preserves_sign_and_compiles_emitters() {
    use wc3::model::DynamicTrack;

    for (x, magnitude, expected_x, expected_z) in [
        (64, 100i16, 1., 1.),
        (192, 100, -1., 1.),
        (64, -100, 1., -1.),
    ] {
        let [low, high] = magnitude.to_le_bytes();
        let bundle = gravity_fixture(true, u32::from_le_bytes([x, 0, low, high]));
        let imported = semantic_m2::import(&bundle, &ImportOptions::default())
            .unwrap()
            .value;
        let gravity = imported
            .global_tracks()
            .filter(|v| matches!(v.clock(), TimeSource::Shared(_)))
            .find_map(|channel| {
                let DynamicTrack::Vector3(track) = channel else {
                    return None;
                };
                Some(track.curve.values().get(0).unwrap())
            })
            .expect("native packed gravity must become a vector channel");
        assert_eq!(gravity[0].signum(), expected_x);
        assert_eq!(gravity[1], 0.);
        assert_eq!(gravity[2].signum(), expected_z);
        let compiled = super::super::compile_model(&imported.widen(), |_| None).unwrap();
        assert!(!compiled.particles.is_empty());
    }
}

/// Native packed words keep direction and signed magnitude independently visible.
#[test]
#[ignore = "writes native M2 gravity variants and TGA under /tmp for the shared capture example"]
fn write_packed_gravity_capture_fixtures() {
    let directory = "/tmp/wc3-m2-gravity-fixture";
    create_dir_all(directory).unwrap();
    for (name, packed, gravity) in [
        ("positive", true, u32::from_le_bytes([64, 0, 100, 0])),
        ("negative", true, u32::from_le_bytes([192, 0, 100, 0])),
        ("inverted", true, u32::from_le_bytes([64, 0, 156, 255])),
        ("scalar", false, 4.0f32.to_bits()),
    ] {
        let bundle = gravity_fixture(packed, gravity);
        let imported = semantic_m2::import(&bundle, &ImportOptions::default())
            .unwrap()
            .value
            .widen();
        let compiled = super::super::compile_model(&imported, |_| None).unwrap();
        assert!(!compiled.particles.is_empty());
        write(
            format!("{directory}/{name}.m2"),
            bundle.encode().unwrap().model,
        )
        .unwrap();
    }
    // Asymmetric opaque texels reveal rotation without external copyrighted assets.
    let mut tga = vec![0u8; 18];
    tga[2] = 2;
    tga[12..14].copy_from_slice(&8u16.to_le_bytes());
    tga[14..16].copy_from_slice(&8u16.to_le_bytes());
    tga[16] = 24;
    tga[17] = 0x20;
    for y in 0..8 {
        for x in 0..8 {
            tga.extend_from_slice(if x < 2 || y < 2 {
                &[80, 160, 255]
            } else {
                &[255, 255, 255]
            });
        }
    }
    write(format!("{directory}/atlas.tga"), tga).unwrap();
    println!("Native M2 fixtures: {directory}/{{positive,negative,inverted,scalar}}.m2");
}

#[test]
#[ignore = "writes synthetic M2 geometry/material variants under /tmp for GPU comparisons"]
fn write_gpu_sprite_capture_fixtures() {
    use wc3::model::common::ZeroVec;
    use wc3::model::formats::m2::records::{
        LegacyValues, ParticleBlend, ParticleGenerator, ParticleHeadTail,
    };

    write_packed_gravity_capture_fixtures();
    let directory = "/tmp/wc3-m2-gravity-fixture";
    for name in [
        "head-tail",
        "velocity",
        "xy-attached",
        "xy-detached",
        "multi-two",
        "multi-three",
        "spline",
        "premultiplied",
        "sphere",
        "implosion-inward",
        "implosion-outward",
    ] {
        let mut bundle = gravity_fixture(false, 0.0f32.to_bits());
        let core = bundle.model.core_mut();
        let mut secondary = core.textures[0].clone();
        secondary.filename = ZeroVec::alloc_from_slice(b"secondary.tga\0");
        let mut tertiary = secondary.clone();
        tertiary.filename = ZeroVec::alloc_from_slice(b"tertiary.tga\0");
        core.textures.extend([secondary, tertiary]);
        let emitter = &mut core.particles[0];
        emitter.flags.set_disable_lighting(true);
        emitter.texture_scale.head_or_tail = ParticleHeadTail::Both;
        emitter.tail_length = 0.4;
        emitter.emission_speed.values = LegacyValues::Values(ZeroVec::alloc_from_slice(&[0.8]));
        emitter.emission_area_width = emitter.emission_speed.clone();
        emitter.emission_area_length = emitter.emission_speed.clone();
        emitter.emission_area_width.values =
            LegacyValues::Values(ZeroVec::alloc_from_slice(&[0.5]));
        emitter.emission_area_length.values =
            LegacyValues::Values(ZeroVec::alloc_from_slice(&[0.5]));
        match name {
            "velocity" => emitter.flags.set_velocity_oriented(true),
            "xy-attached" => {
                emitter.flags.set_align_xy(true);
                emitter.flags.set_attached(true);
            }
            "xy-detached" => emitter.flags.set_align_xy(true),
            "multi-two" | "multi-three" => {
                emitter.flags.set_multi_texture(true);
                emitter
                    .flags
                    .set_multi_texture_three_colors(name == "multi-three");
                emitter.texture_id = (1 << 5) | (2 << 10);
            }
            "spline" => {
                emitter.kind.emitter = ParticleGenerator::Spline;
                emitter.spline_points =
                    ZeroVec::alloc_from_slice(&[[0., 0., 0.], [0.5, 0., 0.], [0.5, 0.8, 0.]]);
                emitter.emission_area_length.values =
                    LegacyValues::Values(ZeroVec::alloc_from_slice(&[0.]));
                emitter.emission_area_width.values =
                    LegacyValues::Values(ZeroVec::alloc_from_slice(&[1.]));
            }
            "premultiplied" => emitter.kind.blending = ParticleBlend::BlendAdd,
            "sphere" | "implosion-inward" | "implosion-outward" => {
                emitter.kind.emitter = ParticleGenerator::Sphere;
                emitter.flags.set_implosion_filter(name != "sphere");
                if name == "implosion-inward" {
                    emitter.emission_speed.values =
                        LegacyValues::Values(ZeroVec::alloc_from_slice(&[-0.8]));
                }
            }
            _ => {}
        }
        bundle
            .validate_complete()
            .unwrap_or_else(|error| panic!("native sprite fixture {name}: {error:?}"));
        write(
            format!("{directory}/{name}.m2"),
            bundle.encode().unwrap().model,
        )
        .unwrap();
    }
    for (name, texel) in [
        ("secondary", [255, 180, 100, 160]),
        ("tertiary", [160, 255, 180, 220]),
    ] {
        let mut tga = vec![0u8; 18];
        tga[2] = 2;
        tga[12..14].copy_from_slice(&8u16.to_le_bytes());
        tga[14..16].copy_from_slice(&8u16.to_le_bytes());
        tga[16] = 32;
        tga[17] = 0x28;
        for _ in 0..64 {
            tga.extend_from_slice(&texel);
        }
        write(format!("{directory}/{name}.tga"), tga).unwrap();
    }
    println!("GPU M2 variants written to {directory}");
}
