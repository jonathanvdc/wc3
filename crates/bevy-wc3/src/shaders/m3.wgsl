#import "embedded://bevy_wc3/shaders/model_io.wgsl"::{VertexOutput,to_bevy}
#import "embedded://bevy_wc3/shaders/combiner.wgsl"::{secondary,secondary_sampler,tertiary,tertiary_sampler,fourth,fourth_sampler}
#import bevy_pbr::{pbr_bindings,pbr_types,pbr_functions}

struct Layer { uv:mat3x3<f32>, color:vec4<f32>, controls:vec4<f32>, flags:vec4<u32>, route:vec4<u32>, };
struct M3 { settings:vec4<u32>, blends:vec4<u32>, diffuse_team:vec4<f32>, emissive_team:vec4<f32>, response:vec4<f32>, normal_weights:array<vec4<f32>,2>, layers:array<Layer,15>, };
@group(#{MATERIAL_BIND_GROUP}) @binding(116) var<uniform> m3:M3;
#import "embedded://bevy_wc3/shaders/surface_images.wgsl"::{orm_map,orm_sampler,team_map,team_sampler,environment_map,environment_sampler}
@group(#{MATERIAL_BIND_GROUP}) @binding(117) var normal_secondary:texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(119) var normal_secondary_mask:texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(121) var light_map:texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(123) var ambient_occlusion:texture_2d<f32>;

fn layer_uv(in:VertexOutput,slot:u32)->vec2<f32> {
    let layer=m3.layers[slot];
    var uv=vec2(0.0);
#ifdef VERTEX_UVS_A
    uv=in.uv;
#endif
#ifdef VERTEX_UVS_B
    if layer.route.x==1u {uv=in.uv_b;}
#endif
#ifdef VERTEX_UVS_C
    if layer.route.x==2u {uv=in.uv_c;}
#endif
#ifdef VERTEX_UVS_D
    if layer.route.x==3u {uv=in.uv_d;}
#endif
    return (layer.uv*vec3(uv,1.0)).xy;
}
// Native wrap states share samplers independently of image-role count.
fn sample_m3(image:texture_2d<f32>,uv:vec2<f32>,selector:u32)->vec4<f32> {
    switch selector {
        case 0u: {return textureSample(image,secondary_sampler,uv);}
        case 1u: {return textureSample(image,orm_sampler,uv);}
        case 2u: {return textureSample(image,team_sampler,uv);}
        case 3u: {return textureSample(image,environment_sampler,uv);}
        case 4u: {return textureSample(image,tertiary_sampler,uv);}
        default: {return textureSample(image,fourth_sampler,uv);}
    }
}
fn raw_layer(in:VertexOutput,slot:u32)->vec4<f32> {
    let layer=m3.layers[slot];
    if (layer.flags.w&1024u)!=0u {return layer.color;}
    if layer.flags.y==0u {return vec4(1.0);}
    let uv=layer_uv(in,slot);
    switch slot {
        case 0u: {return sample_m3(pbr_bindings::base_color_texture,uv,layer.route.y);}
        case 1u: {return sample_m3(pbr_bindings::metallic_roughness_texture,uv,layer.route.y);}
        case 2u: {return sample_m3(pbr_bindings::occlusion_texture,uv,layer.route.y);}
        case 3u: {return sample_m3(pbr_bindings::emissive_texture,uv,layer.route.y);}
        case 4u: {return sample_m3(secondary,uv,layer.route.y);}
        case 5u: {return sample_m3(orm_map,uv,layer.route.y);}
        case 6u: {return sample_m3(team_map,uv,layer.route.y);}
        case 7u: {return sample_m3(environment_map,uv,layer.route.y);}
        case 8u: {return sample_m3(pbr_bindings::normal_map_texture,uv,layer.route.y);}
        case 9u: {return sample_m3(tertiary,uv,layer.route.y);}
        case 10u: {return sample_m3(fourth,uv,layer.route.y);}
        case 11u: {return sample_m3(normal_secondary,uv,layer.route.y);}
        case 12u: {return sample_m3(normal_secondary_mask,uv,layer.route.y);}
        case 13u: {return sample_m3(light_map,uv,layer.route.y);}
        default: {return sample_m3(ambient_occlusion,uv,layer.route.y);}
    }
}
fn evaluated_layer(in:VertexOutput,slot:u32)->vec4<f32> {
    let layer=m3.layers[slot];
    var color=raw_layer(in,slot);
    let raw_alpha=color.a;
    switch layer.flags.z {
        case 0u: {color.a=1.0;}
        case 2u: {color=vec4(color.a);}
        case 3u: {color=vec4(color.r);}
        case 4u: {color=vec4(color.g);}
        case 5u: {color=vec4(color.b);}
        default: {}
    }
    color.a*=layer.controls.z;
    if slot==0u && layer.flags.z!=0u {
        let team_mask=clamp(pow(max(raw_alpha,0.0),m3.diffuse_team.w),0.0,1.0);
        color=vec4(mix(m3.diffuse_team.rgb,color.rgb,team_mask),1.0);
    }
    if (layer.flags.w&16u)!=0u {color=vec4(1.0)-color;}
    if (layer.flags.w&128u)!=0u {color*=layer.controls.x;}
    if (layer.flags.w&64u)!=0u {color+=vec4(layer.controls.y);}
    if (layer.flags.w&32u)!=0u {color=clamp(color,vec4(0.0),vec4(1.0));}
    return color;
}
fn layer_blend(base:vec3<f32>,color:vec4<f32>,operation:u32)->vec3<f32> {
    switch operation {
        case 0u: {return base*color.rgb;}
        case 1u: {return base*color.rgb*2.0;}
        case 2u: {return base+color.rgb*color.a;}
        case 3u: {return mix(base,color.rgb,color.a);}
        case 4u: {return base+m3.emissive_team.rgb*clamp(pow(max(color.a,0.0),m3.emissive_team.w),0.0,1.0);}
        case 5u: {return base+m3.diffuse_team.rgb*color.a;}
        case 6u: {return base+color.rgb;}
        default: {return base;}
    }
}
fn m3_alpha(in:VertexOutput)->f32 {
    var alpha=pbr_bindings::material.base_color.a;
#ifdef VERTEX_COLORS
    if m3.settings.z!=0u {alpha*=in.color.a;}
#endif
    if m3.layers[6].flags.x!=0u {alpha*=evaluated_layer(in,6u).a;}
    if m3.layers[7].flags.x!=0u {alpha*=evaluated_layer(in,7u).a;}
    return alpha;
}
fn tangent_normal(in:VertexOutput,slot:u32)->vec3<f32> {
    let sample=raw_layer(in,slot);
    let xy=sample.ag*2.0-vec2(1.0);
    return vec3(xy,sqrt(max(0.0,1.0-dot(xy,xy))));
}
fn m3_normal(in:VertexOutput,normal:vec3<f32>,is_front:bool)->vec3<f32> {
    if m3.layers[8].flags.x==0u && ((m3.blends.w&4u)==0u || (m3.layers[9].flags.x==0u && m3.layers[11].flags.x==0u)) {return normalize(normal);}
    var nt=vec3(0.0,0.0,1.0);
    if m3.layers[8].flags.x!=0u {nt=tangent_normal(in,8u);}
    if (m3.blends.w&4u)!=0u {
        var factor=0.0;
        if m3.layers[10].flags.x!=0u {factor+=dot(evaluated_layer(in,10u),m3.normal_weights[0]);}
        if (m3.blends.w&134217728u)!=0u && m3.layers[12].flags.x!=0u {factor+=dot(evaluated_layer(in,12u),m3.normal_weights[1]);}
        factor=clamp(factor,-1.0,1.0);factor*=abs(factor);
        var chosen=nt;
        if factor<0.0 && m3.layers[9].flags.x!=0u {chosen=tangent_normal(in,9u);}
        if factor>=0.0 && m3.layers[11].flags.x!=0u {chosen=tangent_normal(in,11u);}
        nt=normalize(mix(nt,chosen,abs(factor)));
    }
#ifdef VERTEX_TANGENTS
#ifdef PREPASS_PIPELINE
#ifdef NORMAL_PREPASS_OR_DEFERRED_PREPASS
    let tbn=pbr_functions::calculate_tbn_mikktspace(normal,in.world_tangent);
    let flags=pbr_bindings::material.flags&~pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP;
    let double_sided=(flags&pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT)!=0u;
    return pbr_functions::apply_normal_mapping(flags,tbn,double_sided,is_front,nt*0.5+vec3(0.5));
#else
    return normalize(normal);
#endif
#else
    let tbn=pbr_functions::calculate_tbn_mikktspace(normal,in.world_tangent);
    let flags=pbr_bindings::material.flags&~pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP;
    let double_sided=(flags&pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT)!=0u;
    return pbr_functions::apply_normal_mapping(flags,tbn,double_sided,is_front,nt*0.5+vec3(0.5));
#endif
#else
    return normalize(normal);
#endif
}
struct M3Result {input:pbr_types::PbrInput,emission_a:vec4<f32>,emission_b:vec4<f32>,};
fn apply_m3(in:VertexOutput,is_front:bool,input:pbr_types::PbrInput)->M3Result {
    var result=input;
    var diffuse=vec3(0.0);
    if m3.layers[0].flags.x!=0u {diffuse=evaluated_layer(in,0u).rgb;}
    if m3.layers[1].flags.x!=0u {
        let decal=evaluated_layer(in,1u);
        diffuse=layer_blend(diffuse,decal,m3.blends.x);
    }
    var tint=pbr_bindings::material.base_color.rgb;
#ifdef VERTEX_COLORS
    if m3.settings.y!=0u {tint*=in.color.rgb;}
#endif
    result.material.base_color=vec4(diffuse*tint,m3_alpha(in));
    result.material.reflectance=vec3(0.0);
    result.material.perceptual_roughness=1.0;
    if m3.layers[2].flags.x!=0u {
        let sample=evaluated_layer(in,2u);
        var specular=sample.rgb;
        if m3.settings.w==1u {specular=vec3(sample.a);}
        result.material.reflectance=clamp(specular*m3.response.y,vec3(0.0),vec3(1.0));
        var exponent=m3.response.x;
        if m3.layers[3].flags.x!=0u {let gloss=evaluated_layer(in,3u).a;exponent*=gloss*gloss;}
        result.material.perceptual_roughness=pow(2.0/(max(exponent,0.0)+2.0),0.25);
    }
    result.N=m3_normal(in,result.world_normal,is_front);
    if m3.layers[14].flags.x!=0u {let ao=evaluated_layer(in,14u).a;result.diffuse_occlusion=vec3(ao);result.specular_occlusion=ao;}
    if m3.layers[13].flags.x!=0u {result.material.emissive=vec4(diffuse*tint*evaluated_layer(in,13u).rgb*2.0,1.0);}
    var emission_a=vec4(0.0);var emission_b=vec4(0.0);
    if m3.layers[4].flags.x!=0u {emission_a=evaluated_layer(in,4u);}
    if m3.layers[5].flags.x!=0u {emission_b=evaluated_layer(in,5u);}
    return M3Result(result,emission_a,emission_b);
}
fn modulating_emission(operation:u32)->bool {return operation==0u||operation==1u||operation==3u;}
fn m3_after_lighting(lit:vec4<f32>,a:vec4<f32>,b:vec4<f32>)->vec4<f32> {
    var rgb=lit.rgb;var emission=vec3(0.0);var additive=false;
    if m3.layers[4].flags.x!=0u {
        if modulating_emission(m3.blends.y) {rgb=layer_blend(rgb,a,m3.blends.y);}
        else {emission=layer_blend(emission,a,m3.blends.y);additive=true;}
    }
    if m3.layers[5].flags.x!=0u {
        if !additive && modulating_emission(m3.blends.z) {rgb=layer_blend(rgb,b,m3.blends.z);}
        else {emission=layer_blend(emission,b,m3.blends.z);additive=true;}
    }
    if additive {rgb+=emission*m3.response.z;}
    return vec4(rgb,lit.a);
}
