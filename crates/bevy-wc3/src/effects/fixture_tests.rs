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
