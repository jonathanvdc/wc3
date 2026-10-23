use super::*;
use crate::effects::age_sprites_gpu::AgeSpriteStyle;
use bytemuck::Zeroable;
use naga::front::wgsl::parse_str;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::TypeInner;
use wc3::model::values::model::ParticleBlend;

#[test]
fn two_views_sort_shared_heads_and_tails_independently() {
    let records = [-3., 2., -1.].map(|z| AgeSpriteRecord {
        position: [0., 0., z, 0.],
        ..Zeroable::zeroed()
    });
    let mut data = AgeSpriteData {
        source: AgeSpriteSource::Sampled(Arc::new(records.into())),
        live_indices: Arc::new(vec![0, 0x8000_0000, 1, 0x8000_0001, 2, 0x8000_0002]),
        uniform: Default::default(),
        style: AgeSpriteStyle {
            textures: [None, None, None],
            blend: ParticleBlend::Alpha,
            priority: 0.,
            sort: SortPolicy::Far,
            composition: Default::default(),
            uv_flip_v: false,
            unlit: false,
            head: true,
            tail: true,
        },
    };
    assert_eq!(
        sorted_indices(&data, &Affine3A::IDENTITY),
        [0, 0x8000_0000, 2, 0x8000_0002, 1, 0x8000_0001]
    );
    let other = Affine3A::from_quat(Quat::from_rotation_y(std::f32::consts::PI));
    assert_eq!(
        sorted_indices(&data, &other),
        [1, 0x8000_0001, 2, 0x8000_0002, 0, 0x8000_0000]
    );
    data.style.sort = SortPolicy::Near;
    assert_eq!(
        sorted_indices(&data, &Affine3A::IDENTITY),
        [1, 0x8000_0001, 2, 0x8000_0002, 0, 0x8000_0000]
    );
    assert_eq!(data.source.sampled().unwrap()[0].position[2], -3.);
}

#[test]
fn equal_depth_ties_keep_alive_order_and_head_tail_pairs_without_repeating_particles() {
    let data = AgeSpriteData {
        source: AgeSpriteSource::Sampled(Arc::new(
            [-3., -3., 2., -1.]
                .map(|z| AgeSpriteRecord {
                    position: [0., 0., z, 0.],
                    ..Zeroable::zeroed()
                })
                .into(),
        )),
        live_indices: Arc::new(vec![
            0,
            0x8000_0000,
            1,
            0x8000_0001,
            2,
            0x8000_0002,
            3,
            0x8000_0003,
        ]),
        uniform: Default::default(),
        style: AgeSpriteStyle {
            textures: [None, None, None],
            blend: ParticleBlend::Alpha,
            priority: 0.,
            sort: SortPolicy::Far,
            composition: Default::default(),
            uv_flip_v: false,
            unlit: false,
            head: true,
            tail: true,
        },
    };
    assert_eq!(
        sorted_indices(&data, &Affine3A::IDENTITY),
        [
            0,
            0x8000_0000,
            1,
            0x8000_0001,
            3,
            0x8000_0003,
            2,
            0x8000_0002
        ]
    );
}

#[test]
fn age_shader_validates_and_matches_host_record_layout() {
    let shader = include_str!("../../shaders/wc3_age_particle.wgsl");
    let body = &shader[shader.find("struct Particle").unwrap()..];
    let imports = r#"
struct View { world_from_view: mat4x4<f32>, view_from_world: mat4x4<f32>, world_position: vec3<f32> };
@group(0) @binding(0) var<uniform> view: View;
fn position_world_to_clip(p: vec3<f32>) -> vec4<f32> { return vec4(p, 1.0); }
fn shade_effect(color: vec4<f32>, position: vec3<f32>, normal: vec3<f32>,
    frag_coord: vec4<f32>, unlit: bool, fog_enabled: bool, fog_blend: u32) -> vec4<f32> { return color; }
"#;
    for (alpha_key, refraction, sampled, ribbon) in [
        (false, false, false, false),
        (true, false, false, false),
        (false, true, false, false),
        (true, true, false, false),
        (false, false, true, false),
        (true, false, true, false),
        (false, false, true, true),
        (true, false, true, true),
    ] {
        let mut enabled = vec![true];
        let mut source = imports.to_owned();
        for line in body.lines() {
            match line.trim() {
                "#ifndef IMMUTABLE_APPEARANCE" => enabled.push(*enabled.last().unwrap()),
                "#ifdef IMMUTABLE_APPEARANCE" => enabled.push(false),
                "#ifdef ALPHA_KEY" => enabled.push(*enabled.last().unwrap() && alpha_key),
                "#ifdef REFRACTION" => enabled.push(*enabled.last().unwrap() && refraction),
                "#ifdef SAMPLED_SPRITE" => enabled.push(*enabled.last().unwrap() && sampled),
                "#ifdef SAMPLED_RIBBON" => enabled.push(*enabled.last().unwrap() && ribbon),
                "#else" => {
                    let branch = enabled.pop().unwrap();
                    enabled.push(*enabled.last().unwrap() && !branch);
                }
                "#endif" => {
                    enabled.pop().unwrap();
                }
                _ if *enabled.last().unwrap() => {
                    source.push_str(line);
                    source.push('\n');
                }
                _ => {}
            }
        }
        let module =
            parse_str(&source).unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
        Validator::new(ValidationFlags::all(), Capabilities::all())
            .validate(&module)
            .unwrap();
        for (name, size) in [
            ("Particle", size_of::<AgeSpriteRecord>()),
            ("Emitter", size_of::<AgeSpriteUniform>()),
        ] {
            let ty = module
                .types
                .iter()
                .find(|(_, ty)| ty.name.as_deref() == Some(name))
                .unwrap()
                .1;
            let TypeInner::Struct { span, .. } = ty.inner else {
                panic!("expected struct")
            };
            assert_eq!(span as usize, size);
        }
    }
}

#[test]
fn native_particle_blends_keep_their_distinct_gpu_factors() {
    for blend in [ParticleBlend::Opaque, ParticleBlend::AlphaKey] {
        assert_eq!(blend_state(blend.raw() as u8), None);
    }
    for (blend, source, destination) in [
        (
            ParticleBlend::Alpha,
            BlendFactor::SrcAlpha,
            BlendFactor::OneMinusSrcAlpha,
        ),
        (
            ParticleBlend::NoAlphaAdd,
            BlendFactor::One,
            BlendFactor::One,
        ),
        (ParticleBlend::Add, BlendFactor::SrcAlpha, BlendFactor::One),
        (ParticleBlend::Modulate, BlendFactor::Zero, BlendFactor::Src),
        (
            ParticleBlend::Modulate2x,
            BlendFactor::Dst,
            BlendFactor::Src,
        ),
        (
            ParticleBlend::BlendAdd,
            BlendFactor::One,
            BlendFactor::OneMinusSrcAlpha,
        ),
    ] {
        let state = blend_state(blend.raw() as u8).unwrap();
        assert_eq!(state.color.src_factor, source);
        assert_eq!(state.color.dst_factor, destination);
        assert_eq!(state.alpha, BlendComponent::OVER);
    }
}

#[test]
fn refraction_composition_shader_validates() {
    let source = include_str!("../../shaders/particle_refraction.wgsl");
    let body = &source[source.find("struct RefractionView").unwrap()..];
    let source = format!("struct FullscreenVertexOutput {{ @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }};\n{body}");
    let module =
        parse_str(&source).unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));
    Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .unwrap();
}

#[test]
fn immutable_sparse_slots_sort_per_view_and_header_transition_preserves_old_extract() {
    use crate::effects::age_sprites_gpu::immutable::{
        BirthSlots, ImmutableAppearance, ImmutableSpriteData,
    };
    use bevy::math::Mat3;

    let appearance = Arc::new(ImmutableAppearance {
        source: String::new(),
        words_per_slot: 32,
        requires_time_horizon: false,
        shader: Default::default(),
    });
    let mut births = BirthSlots::new(96, 32).unwrap();
    for (slot, z) in [(91, -3f32), (4, -3.), (70, 2.), (13, -1.)] {
        let mut row = [0; 32];
        row[2] = z.to_bits();
        births.set(slot, 1, u64::from(slot) + 100, &row).unwrap();
    }
    // Live allocation order deliberately differs from slot order, including ties.
    births.set_live(&[91, 4, 70, 13]);
    let data = AgeSpriteData {
        source: AgeSpriteSource::Immutable(ImmutableSpriteData {
            appearance: appearance.clone(),
            births: births.snapshot.clone(),
        }),
        live_indices: births.live.clone(),
        uniform: Default::default(),
        style: AgeSpriteStyle {
            textures: [None, None, None],
            blend: ParticleBlend::Alpha,
            priority: 0.,
            sort: SortPolicy::Far,
            composition: Default::default(),
            uv_flip_v: false,
            unlit: true,
            head: true,
            tail: false,
        },
    };
    let opposite =
        Affine3A::from_mat3_translation(Mat3::from_diagonal(Vec3::new(-1., 1., -1.)), Vec3::ZERO);
    assert_eq!(sorted_indices(&data, &Affine3A::IDENTITY), [91, 4, 13, 70]);
    assert_eq!(sorted_indices(&data, &opposite), [70, 13, 91, 4]);

    // First evolution can replace an initialized center without changing identity.
    let before = births.snapshot.clone();
    let mut evolved = [0; 32];
    evolved[2] = 4f32.to_bits();
    evolved[31] = 1;
    births.set(91, 1, 191, &evolved).unwrap();
    assert!(births.matches(91, 1, 191));
    assert!(!Arc::ptr_eq(&before, &births.snapshot));
    assert!(Arc::ptr_eq(&before.blocks[0], &births.snapshot.blocks[0]));
    assert_eq!(before.position(91), Vec3::new(0., 0., -3.));
    assert_eq!(births.snapshot.position(91), Vec3::new(0., 0., 4.));
    let mut changed = data.clone();
    changed.source = AgeSpriteSource::Immutable(ImmutableSpriteData {
        appearance,
        births: births.snapshot.clone(),
    });
    assert!(Arc::ptr_eq(&data.live_indices, &changed.live_indices));
    assert_eq!(
        sorted_indices(&changed, &Affine3A::IDENTITY),
        [4, 13, 70, 91]
    );
    assert_eq!(sorted_indices(&changed, &opposite), [91, 70, 13, 4]);
    // The older extracted frame remains valid and retains its two-view order.
    assert_eq!(sorted_indices(&data, &Affine3A::IDENTITY), [91, 4, 13, 70]);
    assert_eq!(sorted_indices(&data, &opposite), [70, 13, 91, 4]);
}
