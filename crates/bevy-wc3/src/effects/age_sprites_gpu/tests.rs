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
        records: Arc::new(records.into()),
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
    assert_eq!(data.records[0].position[2], -3.);
}

#[test]
fn age_shader_validates_and_matches_host_record_layout() {
    let shader = include_str!("../../shaders/wc3_age_particle.wgsl");
    let body = &shader[shader.find("struct Particle").unwrap()..];
    let imports = r#"
struct View { world_from_view: mat4x4<f32>, world_position: vec3<f32> };
@group(0) @binding(0) var<uniform> view: View;
fn position_world_to_clip(p: vec3<f32>) -> vec4<f32> { return vec4(p, 1.0); }
fn shade_effect(color: vec4<f32>, position: vec3<f32>, normal: vec3<f32>,
    frag_coord: vec4<f32>, unlit: bool, fog_enabled: bool, fog_blend: u32) -> vec4<f32> { return color; }
"#;
    for alpha_key in [false, true] {
        let mut enabled = true;
        let mut source = imports.to_owned();
        for line in body.lines() {
            match line.trim() {
                "#ifdef ALPHA_KEY" => enabled = alpha_key,
                "#endif" => enabled = true,
                _ if enabled => {
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
