use naga::front::wgsl::parse_str;
use naga::valid::{Capabilities, ValidationFlags, Validator};

#[test]
fn shared_crystal_surface_typechecks_for_forward_and_depth_coverage() {
    // Test the SAME source function used for color and coverage, with typed
    // stubs only for Bevy/image IO. This validates shader structure, not pixels.
    let body = include_str!("../shaders/crystal.wgsl");
    let stubs = r#"
struct VertexOutput { uv:vec2<f32>, crystal_normal:vec3<f32>, crystal_tangent:vec4<f32>, world_normal:vec3<f32>, world_tangent:vec4<f32>, world_position:vec4<f32>, color:vec4<f32>, instance_index:u32 };
struct Hd { fresnel:vec4<f32>,fresnel_color:vec4<f32>,maps:vec4<u32> };
struct Standard { flags:u32,base_color:vec4<f32>,uv_transform:mat3x3<f32> };
struct View { mip_bias:f32,clip_from_view:mat4x4<f32>,view_from_world:mat4x4<f32> };
@group(0) @binding(0) var<uniform> view:View;
const STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT:u32=1u;
const STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP:u32=2u;
fn load_hd(i:u32)->Hd { return Hd(vec4(0.2,0.0,1.0,0.0),vec4(1.0),vec4(3u,1u,1u,0u)); }
fn load_standard(i:u32)->Standard {return Standard(0u,vec4(1.0),mat3x3<f32>(vec3(1.0,0.0,0.0),vec3(0.0,1.0,0.0),vec3(0.0,0.0,1.0)));}
fn sample_base(uv:vec2<f32>,i:u32)->vec4<f32>{return vec4(uv,0.5,1.0);}
fn sample_orm_bias(uv:vec2<f32>,i:u32,b:f32)->vec4<f32>{return vec4(0.5);}
fn sample_normal_bias(uv:vec2<f32>,i:u32,b:f32)->vec4<f32>{return vec4(0.5,0.5,1.0,1.0);}
fn sample_team_bias(uv:vec2<f32>,i:u32,b:f32)->vec4<f32>{return vec4(1.0);}
fn prepare_world_normal(n:vec3<f32>,two:bool,front:bool)->vec3<f32>{return n;}
fn calculate_tbn_mikktspace(n:vec3<f32>,t:vec4<f32>)->mat3x3<f32>{return mat3x3<f32>(t.xyz,cross(n,t.xyz),n);}
fn apply_normal_mapping(f:u32,t:mat3x3<f32>,two:bool,front:bool,n:vec3<f32>)->vec3<f32>{return normalize(t*(n*2.0-vec3(1.0)));}
fn calculate_view(p:vec4<f32>,o:bool)->vec3<f32>{return vec3(0.0,0.0,1.0);}
"#;
    for prepass in [false, true] {
        for tangent in [false, true] {
            for color in [false, true] {
                let mut source = stubs.to_owned();
                let mut conditions = Vec::new();
                let mut enabled = true;
                for line in body.lines() {
                    let text = line.trim();
                    if text.starts_with("#import") {
                        continue;
                    }
                    if let Some(name) = text.strip_prefix("#ifdef ") {
                        conditions.push((
                            enabled,
                            match name {
                                "PREPASS_PIPELINE" => prepass,
                                "VERTEX_TANGENTS" => tangent,
                                "VERTEX_COLORS" => color,
                                "VERTEX_UVS_A" => true,
                                _ => panic!("unhandled test permutation {name}"),
                            },
                        ));
                        enabled = conditions.iter().all(|(parent, branch)| *parent && *branch);
                    } else if text == "#else" {
                        let (parent, branch) = conditions.last_mut().unwrap();
                        *branch = !*branch;
                        enabled = *parent && *branch;
                    } else if text == "#endif" {
                        let (parent, _) = conditions.pop().unwrap();
                        enabled = parent;
                    } else if enabled {
                        source.push_str(line);
                        source.push('\n');
                    }
                }
                assert!(conditions.is_empty());
                source = source
                    .replace("pbr_functions::", "")
                    .replace("pbr_types::", "");
                let module =
                    parse_str(&source).unwrap_or_else(|e| panic!("{}", e.emit_to_string(&source)));
                Validator::new(ValidationFlags::all(), Capabilities::all())
                    .validate(&module)
                    .unwrap();
            }
        }
    }
}
