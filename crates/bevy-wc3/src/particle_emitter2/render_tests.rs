use super::*;
use naga::front::wgsl::parse_str;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::TypeInner;
use wc3::model::scene::Node;

fn emitter() -> ParticleEmitter2 {
    ParticleEmitter2::new(Node::new("Particles", 0).unwrap())
}

fn instances(records: Vec<ParticleInstance>, uniform: ParticleEmitterUniform) -> ParticleInstances {
    ParticleInstances {
        live_indices: Arc::new((0..records.len() as u32).collect()),
        records: Arc::new(records.into()),
        uniform,
        texture: None,
        filter: Particle2FilterMode::Blend,
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
fn xy_facing_survives_gravity_and_tail_conversion() {
    let record = ParticleInstance::new(
        Vec3::ZERO,
        Vec3::new(0.0, 2.0, 3.0),
        20.0,
        0.0,
        4.0,
        Vec3::new(2.0, 0.5, 3.0),
        false,
    );
    let facing = record.scale_facing[3];
    assert!((facing - (-PI * 0.5 + FRAC_PI_8)).abs() < 0.00001);
    let uniform = ParticleEmitterUniform::new(&emitter(), &GlobalTransform::default(), 1.0);
    assert!(record.center(&uniform).z < 0.0);
    assert_eq!(record.as_tail().scale_facing, record.scale_facing);
    for velocity in [Vec3::ZERO, Vec3::Z, -Vec3::Z] {
        let stationary =
            ParticleInstance::new(Vec3::ZERO, velocity, 0.0, 0.0, 4.0, Vec3::ONE, false);
        assert!(stationary.scale_facing[3].is_finite());
    }
}

#[test]
fn unshaded_is_a_render_flag_independent_of_clock_and_tail_length() {
    let mut definition = emitter();
    let shaded = ParticleEmitterUniform::new(&definition, &GlobalTransform::default(), 12.0);
    definition.node.flags.set_unshaded(true);
    let unshaded = ParticleEmitterUniform::new(&definition, &GlobalTransform::default(), 12.0);
    assert_eq!(shaded.render_flags[0], 0);
    assert_eq!(unshaded.render_flags[0], 1);
    assert_eq!(shaded.clock_tail, unshaded.clock_tail);
}

#[test]
fn analytic_motion_and_model_space_follow_current_transform() {
    let record = ParticleInstance::new(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3::new(4.0, 5.0, 6.0),
        2.0,
        10.0,
        5.0,
        Vec3::ONE,
        false,
    );
    let mut definition = emitter();
    let transform = GlobalTransform::from_translation(Vec3::new(100.0, 200.0, 300.0));
    let uniform = ParticleEmitterUniform::new(&definition, &transform, 12.0);
    assert_eq!(record.center(&uniform), Vec3::new(9.0, 12.0, 11.0));
    definition.node.flags.set_model_space(true);
    let uniform = ParticleEmitterUniform::new(&definition, &transform, 12.0);
    assert_eq!(record.center(&uniform), Vec3::new(109.0, 212.0, 311.0));
}

#[test]
fn split_clock_preserves_short_ages_after_long_runtime() {
    let birth = 1_000_000_000.0;
    let record = ParticleInstance::new(Vec3::ZERO, Vec3::X, 0.0, birth, 1.0, Vec3::ONE, false);
    let uniform =
        ParticleEmitterUniform::new(&emitter(), &GlobalTransform::default(), birth + 0.125);
    assert_eq!(record.center(&uniform), Vec3::new(0.125, 0.0, 0.0));
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
    let uniform = ParticleEmitterUniform::new(&emitter(), &GlobalTransform::default(), 1.0);
    let data = instances(records, uniform);
    assert_eq!(sorted_indices(&data, &Affine3A::IDENTITY), vec![1, 0]);
    let reverse = Affine3A::from_quat(Quat::from_rotation_y(PI));
    assert_eq!(sorted_indices(&data, &reverse), vec![0, 1]);
}

#[test]
fn particle_shader_validates_and_matches_host_buffer_layouts() {
    let shader = include_str!("../shaders/wc3_particle.wgsl");
    // Bevy supplies these imports. Stubs let Naga validate the complete
    // particle shader without starting a renderer or requiring a GPU.
    let body = &shader[shader.find("@group(3)").unwrap()..];
    let imports = r#"
struct View { world_from_view: mat4x4<f32>, clip_from_view: mat4x4<f32> };
@group(0) @binding(0) var<uniform> view: View;
fn position_world_to_clip(p: vec3<f32>) -> vec4<f32> { return vec4(p, 1.0); }
struct Material { base_color: vec4<f32>, reflectance: vec3<f32>, perceptual_roughness: f32 };
struct Lighting {
    material: Material, frag_coord: vec4<f32>, world_position: vec4<f32>,
    world_normal: vec3<f32>, N: vec3<f32>, V: vec3<f32>, is_orthographic: bool,
};
fn pbr_input_new() -> Lighting { var result: Lighting; return result; }
fn calculate_view(p: vec4<f32>, orthographic: bool) -> vec3<f32> { return vec3(0.0, 0.0, 1.0); }
fn apply_pbr_lighting(p: Lighting) -> vec4<f32> { return p.material.base_color; }
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
