use super::*;
use crate::effects::sprites::StagedSpriteParameters;
use bevy::render::extract_component::ExtractComponent;
use naga::front::wgsl::parse_str;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::TypeInner;
use std::f32::consts::PI;
use std::sync::Arc;

fn emitter() -> StagedSpriteParameters {
    StagedSpriteParameters {
        colors: [[1.; 4]; 3],
        scales: [1.; 3],
        middle: 0.5,
        intervals: [[0, 0, 1]; 4],
        atlas: [1, 1],
        model_space: false,
        xy_quad: false,
        tail_length: 0.,
        unlit: false,
        fog_enabled: true,
        fog_blend: 0,
        plane_axes: [Vec3::X, Vec3::Y],
    }
}

fn instances(records: Vec<ParticleInstance>, uniform: ParticleEmitterUniform) -> ParticleInstances {
    ParticleInstances {
        live_indices: Arc::new((0..records.len() as u32).collect()),
        records: Arc::new(records.into()),
        uniform,
        texture: None,
        filter: EffectBlend::Alpha,
        priority_plane: 0,
        sort_far: true,
    }
}

#[test]
fn hidden_emitters_are_removed_from_render_extraction_and_can_reappear() {
    let data = instances(Vec::new(), ParticleEmitterUniform::default());
    assert!(ParticleInstances::extract_component((&data, &InheritedVisibility::VISIBLE)).is_some());
    assert!(ParticleInstances::extract_component((&data, &InheritedVisibility::HIDDEN)).is_none());
    assert!(ParticleInstances::extract_component((&data, &InheritedVisibility::VISIBLE)).is_some());
}

#[test]
fn wrapped_particle_indices_sort_by_depth_instead_of_physical_position() {
    let mut records = RecordRing::default();
    let record =
        |z: f32| ParticleInstance::new(Vec3::Z * z, Vec3::ZERO, 0.0, 0.0, 1.0, Vec3::ONE, false);
    for z in [0.0, 1.0, 2.0, 3.0] {
        records.push(record(z), 4);
    }
    records.pop_front();
    records.pop_front();
    records.push(record(4.0), 4);
    records.push(record(5.0), 4);
    let mut data = instances(Vec::new(), ParticleEmitterUniform::default());
    data.live_indices = Arc::new((0..records.len()).map(|i| records.index(i)).collect());
    data.records = Arc::new(records);
    assert_eq!(*data.live_indices, vec![2, 3, 0, 1]);
    assert_eq!(sorted_indices(&data, &Affine3A::IDENTITY), vec![2, 3, 0, 1]);
    let reverse = Affine3A::from_quat(Quat::from_rotation_y(PI));
    assert_eq!(sorted_indices(&data, &reverse), vec![1, 0, 3, 2]);
}

#[test]
fn sorting_uses_analytic_camera_depth_for_each_view() {
    let records = vec![
        ParticleInstance::new(
            Vec3::new(100.0, 0.0, -1.0),
            Vec3::ZERO,
            0.0,
            0.0,
            5.0,
            Vec3::ONE,
            false,
        ),
        ParticleInstance::new(
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, -2.0),
            0.0,
            0.0,
            5.0,
            Vec3::ONE,
            false,
        ),
    ];
    let uniform = ParticleEmitterUniform::from_staged(&emitter(), Mat4::IDENTITY, 1.0);
    let data = instances(records, uniform);
    assert_eq!(sorted_indices(&data, &Affine3A::IDENTITY), vec![1, 0]);
    let reverse = Affine3A::from_quat(Quat::from_rotation_y(PI));
    assert_eq!(sorted_indices(&data, &reverse), vec![0, 1]);
}

#[test]
fn particle_shader_validates_and_matches_host_buffer_layouts() {
    let shader = include_str!("../../shaders/wc3_particle.wgsl");
    // Bevy supplies these imports. Stubs let Naga validate the complete
    // particle shader without starting a renderer or requiring a GPU.
    let body = &shader[shader.find("@group(2)").unwrap()..];
    let imports = r#"
struct View { world_from_view: mat4x4<f32>, clip_from_view: mat4x4<f32> };
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
            ("Particle", size_of::<ParticleInstance>()),
            ("Emitter", size_of::<ParticleEmitterUniform>()),
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
