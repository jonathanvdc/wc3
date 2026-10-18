//! Explicit opt-in native fixtures for the shared offscreen capture example.
use std::fs::{create_dir_all, write};
use wc3::model::conversion::ImportOptions;
use wc3::model::formats::m2::{
    Bundle as NativeM2Bundle, Flat as NativeM2Flat, V256 as NativeM2V256,
};
use wc3::model::formats::m3::{self, records as nr};
use wc3::model::io::m2 as semantic_m2;
use wc3::model::io::m3 as semantic_m3;
use wc3::model::TimeSource;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

fn value<T: Clone + Default>(initial: T) -> nr::AnimationValue<T> {
    nr::AnimationValue {
        initial_value: initial.clone(),
        null_value: initial,
        ..Default::default()
    }
}
fn color(rgba: [u8; 4]) -> nr::AnimationValue<nr::FixedBytes<4>> {
    value(nr::FixedBytes(rgba))
}

#[test]
#[ignore = "writes a native M3 and TGA under /tmp for the existing GPU capture example"]
fn write_programmed_effect_capture_fixture() {
    let mut slot_values = SlotValues::default();
    let directory = "/tmp/wc3-m3-effects-fixture";
    create_dir_all(directory).unwrap();
    let document = m3::Document::new::<m3::StarCraft2>().unwrap();
    let root_id = document.root_id();
    let mut model = document.root::<nr::ModelRecord<nr::V29>>().unwrap();
    let mut container = document.into_container();
    let names: Vec<_> = b"Effects\0"
        .iter()
        .map(|value| nr::Character { value: *value })
        .collect();
    let names = container.append(&names).unwrap();
    model.name = m3::Ref::from_section(8, names.index());
    let bones: Vec<_> = [[-1.5, 0., 0.], [1.5, 0., 0.], [-2., 0., -1.], [2., 0., -1.]]
        .into_iter()
        .enumerate()
        .map(|(index, position)| nr::Bone::<nr::V1> {
            id: index as i32,
            parent_index: u16::MAX,
            name: m3::Ref::from_section(8, names.index()),
            position: value(position),
            rotation: value([0., 0., 0., 1.]),
            scale: value([1.; 3]),
            ..SlotConstruct::construct(&mut slot_values)
        })
        .collect();
    let bones = container.append(&bones).unwrap();
    model.bones = m3::Ref::from_section(4, bones.index());
    let texture_path: Vec<_> = b"atlas.tga\0"
        .iter()
        .map(|value| nr::Character { value: *value })
        .collect();
    let texture_path = container.append(&texture_path).unwrap();
    let diffuse = nr::TextureLayer::<nr::V25> {
        texture_path: m3::Ref::from_section(10, texture_path.index()),
        color: value([255; 4]),
        rgb_multiply: value(1.),
        map_alpha: value(1.),
        uv_tiling: value([1., 1.]),
        flipbook_rows: 1,
        flipbook_columns: 1,
        ..SlotConstruct::construct(&mut slot_values)
    };
    let diffuse = container.append(&[diffuse]).unwrap();
    let material = nr::StandardMaterial::<nr::V20> {
        blend_mode: nr::BlendMode::AlphaBlend,
        diffuse_layer: m3::Ref::from_section(1, diffuse.index()),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let material = container.append(&[material]).unwrap();
    model.standard_materials = m3::Ref::from_section(1, material.index());
    let map = container
        .append(&[nr::MaterialMap {
            material_type: nr::MaterialKind::Standard,
            material_index: 0,
        }])
        .unwrap();
    model.material_maps = m3::Ref::from_section(1, map.index());
    let base = nr::ParticleEmitter::<nr::V24> {
        bone_index: 0,
        material_index: 0,
        additional_flags: nr::ParticleAdditionalFlags(8),
        initial_speed: value(1.),
        lifetime: value(2.),
        max_particles: 32,
        emission_rate: value(4.),
        emitter_shape: nr::EmitterShape::Point,
        velocity_type: nr::VelocityKind::ZAxis,
        instance_type: nr::ParticleInstance::Billboard,
        size_animation: value([0.15, 0.5, 0.05]),
        size_mid_time: 0.5,
        rotation_animation: value([0.; 3]),
        rotation_mid_time: 0.5,
        color_start: color([255, 80, 80, 255]),
        color_mid: color([80, 255, 80, 255]),
        color_end: color([80, 80, 255, 0]),
        color_mid_time: 0.5,
        alpha_mid_time: 0.75,
        flipbook_rows: 2,
        flipbook_columns: 2,
        flipbook_column_fraction: 0.5,
        flipbook_row_fraction: 0.5,
        flipbook_start_init_index: 0,
        flipbook_start_stop_index: 1,
        flipbook_end_init_index: 2,
        flipbook_end_stop_index: 3,
        flipbook_mid_time: 0.5,
        trail_link_index: -1,
        collision_spawn_index: -1,
        splat_projection_index: -1,
        ribbon_link_index: -1,
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut lit = base;
    lit.bone_index = 1;
    lit.flags.set_lit_parts(true);
    let particles = container.append(&[base, lit]).unwrap();
    model.particle_emitters = m3::Ref::from_section(2, particles.index());
    let endpoint = container
        .append(&[nr::SplineRibbon {
            bone_index: 3,
            ..SlotConstruct::construct(&mut slot_values)
        }])
        .unwrap();
    let ribbon = nr::RibbonEmitter::<nr::V9> {
        bone_index: 2,
        bone_index_fallback: 2,
        material_index: 0,
        ribbon_type: nr::RibbonKind::Billboard,
        divisions: 16.,
        max_length: value(4.),
        active: value(1),
        size_animation: value([0.2, 0.5, 0.1]),
        size_mid_time: 0.5,
        color_start: color([255, 100, 100, 255]),
        color_mid: color([100, 255, 100, 255]),
        color_end: color([100, 100, 255, 255]),
        color_mid_time: 0.5,
        alpha_mid_time: 0.5,
        rotation_mid_time: 0.5,
        spline_ribbons: m3::Ref::from_section(1, endpoint.index()),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let ribbons = container.append(&[ribbon]).unwrap();
    model.ribbon_emitters = m3::Ref::from_section(1, ribbons.index());
    model.bounds = nr::Extent {
        minimum: [-3., -1., -2.],
        maximum: [3., 1., 3.],
        radius: 5.,
    };
    container.replace(root_id, &[model]).unwrap();
    let document = m3::Document::from_container(container).unwrap();
    let bundle = m3::Bundle::from_model(document);
    let imported = semantic_m3::import(&bundle, &ImportOptions::default())
        .unwrap()
        .value
        .widen();
    let compiled = super::super::compile_model(&imported, |_| None).unwrap();
    assert!(!compiled.particles.is_empty());

    let encoded = bundle.write().unwrap();
    write(format!("{directory}/effects.m3"), encoded.model).unwrap();
    // Four bright asymmetric cells expose atlas progression without extra assets.
    let mut tga = vec![0u8; 18];
    tga[2] = 2;
    tga[12..14].copy_from_slice(&64u16.to_le_bytes());
    tga[14..16].copy_from_slice(&64u16.to_le_bytes());
    tga[16] = 24;
    tga[17] = 0x20;
    for y in 0..64 {
        for x in 0..64 {
            let cell = (y / 32) * 2 + x / 32;
            let rgba = match cell {
                0 => [255, 255, 255],
                1 => [200, 255, 255],
                2 => [255, 200, 255],
                _ => [255, 255, 200],
            };
            let dark = (x % 32 < 4) || (y % 32 < 4);
            let rgb = if dark { [40, 40, 40] } else { rgba };
            tga.extend_from_slice(&[rgb[2], rgb[1], rgb[0]]);
        }
    }
    write(format!("{directory}/atlas.tga"), tga).unwrap();
    println!("Native M3 fixture: {directory}/effects.m3");
}

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
