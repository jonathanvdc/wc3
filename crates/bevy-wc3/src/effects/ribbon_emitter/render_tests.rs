use super::*;
use naga::front::wgsl::parse_str;
use naga::valid::{Capabilities, ValidationFlags, Validator};
use naga::TypeInner;
#[test]
fn sorting_preserves_endpoint_pairs_and_uv_ranks_with_split_clock_gravity() {
    let clock = 1_000_000_000.0;
    let records = vec![
        RibbonSection::new(Vec3::Y, -Vec3::Y, clock),
        RibbonSection::new(Vec3::Y + Vec3::X, -Vec3::Y + Vec3::X, clock + 0.25),
        RibbonSection::new(
            Vec3::Y + Vec3::X * 2.0,
            -Vec3::Y + Vec3::X * 2.0,
            clock + 0.5,
        ),
    ];
    let [high, low] = split_time(clock + 0.5);
    let uniform = RibbonUniform {
        clock_gravity: [high, low, 8.0, 0.0],
        ..default()
    };
    assert_eq!(records[0].center(&uniform), Vec3::new(0.0, 0.0, -1.0));
    assert_eq!(records[1].center(&uniform), Vec3::new(1.0, 0.0, -0.25));
    let first = RibbonSegment([0, 1, 0, 2]);
    let second = RibbonSegment([1, 2, 1, 2]);
    let mut data = RibbonInstances {
        records: Arc::new(records.into()),
        live_indices: Arc::new(vec![second, first]),
        uniform,
        texture: None,
        filter: LayerFilterMode::Blend,
        priority_plane: -1,
        emitter: Entity::PLACEHOLDER,
        layer_index: 0,
        sort_far: true,
        sort_near: false,
        no_depth_test: false,
        no_depth_set: true,
        two_sided: true,
    };
    assert_eq!(
        sorted_indices(&data, &Affine3A::IDENTITY),
        vec![first, second]
    );
    data.sort_near = true;
    assert_eq!(
        sorted_indices(&data, &Affine3A::IDENTITY),
        vec![second, first]
    );
    assert!(RibbonInstances::extract_component((&data, &InheritedVisibility::HIDDEN)).is_none());
    assert!(RibbonInstances::extract_component((&data, &InheritedVisibility::VISIBLE)).is_some());
}

#[test]
fn ribbon_shader_validates_and_matches_host_buffer_layouts() {
    let shader = include_str!("../../shaders/wc3_ribbon.wgsl");
    // Bevy supplies these imports. Stubs let Naga validate the complete
    // ribbon shader without starting a renderer or requiring a GPU.
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
            ("Section", size_of::<RibbonSection>()),
            ("Emitter", size_of::<RibbonUniform>()),
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
