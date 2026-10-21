#import "embedded://bevy_wc3/shaders/model_io.wgsl"::VertexOutput
#import "embedded://bevy_wc3/shaders/material_access.wgsl"::{World,load_combiner,load_world,load_standard,sample_base,sample_secondary,sample_tertiary}
#import bevy_pbr::{pbr_bindings,pbr_types}


struct WorldComposed { color: vec4<f32>, emission: vec3<f32>, };
// Programmes select texture composition; Bevy remains authoritative for lighting.
fn compose_world(
    a: vec4<f32>,
    b: vec4<f32>,
    c: vec4<f32>,
    q: f32,
    tint: vec4<f32>,
    world: World
) -> WorldComposed {
    var rgb=a.rgb;
    var alpha=a.a;
    var emission=vec3(0.0);
    switch world.composition.x {
        case 4u: {emission=b.rgb*a.a; alpha=1.0;}
        case 5u: {alpha=1.0;}
        case 6u: {emission=a.rgb*a.a*b.rgb; alpha=1.0;}
        case 7u: {rgb=mix(mix(a.rgb,b.rgb,b.a),a.rgb,q);}
        case 10u: {emission=b.rgb*b.a*q;}
        case 12u: {rgb=mix(mix(2.0*a.rgb*b.rgb,c.rgb,clamp(c.a*q,0.0,1.0)),a.rgb,a.a);}
        case 13u: {emission=a.rgb*a.a*b.rgb+c.rgb*c.a*q;}
        case 14u: {rgb=mix(b.rgb,a.rgb,q); alpha=1.0;}
        case 16u: {rgb=mix(b.rgb*(1.0-b.a),a.rgb,q); emission=b.rgb*b.a*(1.0-q);}
        case 18u: {rgb=mix(2.0*a.rgb*b.rgb+c.rgb*clamp(c.a*q,0.0,1.0),a.rgb,a.a); alpha=1.0;}
        case 19u: {rgb=mix(mix(a.rgb,b.rgb,b.a),a.rgb,q)*c.rgb*2.0;}
        case 20u: {rgb=mix(a.rgb,2.0*a.rgb*b.rgb,q);}
        case 21u: {rgb=mix(mix(a.rgb,b.rgb,b.a),a.rgb,c.a)*c.rgb*2.0;}
        default: {}
    }
    return WorldComposed(vec4(tint.rgb*rgb,tint.a*alpha),tint.rgb*emission);
}

fn world_color(in:VertexOutput,vertex_alpha:f32)->WorldComposed {
    let combiner = load_combiner(in.instance_index);
    let world = load_world(in.instance_index);
    let material = load_standard(in.instance_index);
    var uv0=vec2(0.0); var uv1=vec2(0.0); var uv2=vec2(0.0);
#ifdef VERTEX_UVS_A
    uv0=(material.uv_transform*vec3(in.uv,1.0)).xy;
    uv1=(combiner.uv_transform*vec3(in.uv,1.0)).xy;
    uv2=in.uv;
#endif
#ifdef VERTEX_UVS_B
    uv1=(combiner.uv_transform*vec3(in.uv_b,1.0)).xy;
    if world.composition.z==2u {uv2=in.uv_b;}
#endif
#ifdef VERTEX_UVS_C
    if world.composition.z==1u || world.composition.z==3u {uv2=in.uv_c;}
#endif
    uv2=(combiner.third_transform*vec3(uv2,1.0)).xy;
    var a=vec4(1.0); var b=vec4(1.0); var c=vec4(1.0);
    if (material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_BASE_COLOR_TEXTURE_BIT)!=0u {
        a = sample_base(uv0, in.instance_index);
    }
    if world.composition.y > 1u {
        b = sample_secondary(uv1, in.instance_index);
    }
    if world.composition.y > 2u {
        c = sample_tertiary(uv2, in.instance_index);
    }
    return compose_world(a, b, c, vertex_alpha, material.base_color, world);
}
