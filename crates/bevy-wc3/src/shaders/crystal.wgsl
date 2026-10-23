// Original Crystal surface interpretation inside the ordinary Bevy PBR pipeline.
#import "embedded://bevy_wc3/shaders/model_io.wgsl"::VertexOutput
#import "embedded://bevy_wc3/shaders/material_access.wgsl"::{load_hd,load_standard,sample_base,sample_orm_bias,sample_normal_bias,sample_team_bias}
#import bevy_pbr::{pbr_functions,pbr_types,mesh_view_bindings::view}

struct CrystalSurface {
    diffuse: vec3<f32>,
    normal: vec3<f32>,
    alpha: f32,
    rim: f32,
    rim_color: vec3<f32>,
};

fn prepare_crystal(in: VertexOutput, is_front: bool, include_rgb: bool) -> CrystalSurface {
    let hd=load_hd(in.instance_index);
    let material=load_standard(in.instance_index);
    var uv=vec2(0.0);
#ifdef VERTEX_UVS_A
    uv=(material.uv_transform*vec3(in.uv,1.0)).xy;
#endif
#ifdef PREPASS_PIPELINE
    let geometric=in.crystal_normal;
#ifdef VERTEX_TANGENTS
    let tangent=in.crystal_tangent;
#endif
#else
    let geometric=in.world_normal;
#ifdef VERTEX_TANGENTS
    let tangent=in.world_tangent;
#endif
#endif
    var normal=pbr_functions::prepare_world_normal(geometric,
        (material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT)!=0u,is_front);
#ifdef VERTEX_TANGENTS
    if hd.fresnel.z!=0.0 {
        let encoded=sample_normal_bias(uv,in.instance_index,view.mip_bias);
        let xy=encoded.rg*2.0-vec2(1.0);
        let raw=vec3(xy,sqrt(max(0.0,1.0-dot(xy,xy))))*0.5+vec3(0.5);
        normal=pbr_functions::apply_normal_mapping(
            material.flags & ~pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP,
            pbr_functions::calculate_tbn_mikktspace(geometric,tangent),
            (material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT)!=0u,is_front,raw);
    }
#endif
    normal=normalize(normal);
    let direction=pbr_functions::calculate_view(in.world_position,view.clip_from_view[3].w==1.0);
    var orm=vec4(1.0);
    if hd.maps.y!=0u { orm=sample_orm_bias(uv,in.instance_index,view.mip_bias); }
    let mask=orm.r;
    var tint=material.base_color;
#ifdef VERTEX_COLORS
    tint*=in.color;
#endif
    let center=sample_base(uv,in.instance_index);
    var diffuse=center.rgb;
    if include_rgb {
        // Geometric vertex normals are normalized by Bevy BEFORE interpolation.
        // Linear view rotation preserves that interpolation; do not normalize
        // here and do not substitute the normal-map shading normal.
        let view_geometric=(view.view_from_world*vec4(geometric,0.0)).xyz;
        // Explicit finite grazing policy; native denominator behavior unverified.
        let displaced=uv+0.02*view_geometric.xy/max(abs(view_geometric.z),0.0001);
        diffuse=mix(sample_base(displaced,in.instance_index).rgb,center.rgb,mask);
    }
    var team=vec3(1.0);
    if hd.maps.z!=0u {team=sample_team_bias(uv,in.instance_index,view.mip_bias).rgb;}
    if hd.maps.y!=0u && hd.maps.z!=0u {diffuse*=mix(vec3(1.0),team,orm.a);}
    let edge=1.0-clamp(dot(normal,direction),0.0,1.0);
    let rim=mix(0.2,hd.fresnel.x,mask)*edge*edge;
    var result:CrystalSurface;
    result.diffuse=diffuse*tint.rgb;
    result.normal=normal;
    result.alpha=clamp(center.a*(tint.a+rim),0.0,1.0);
    result.rim=rim;
    result.rim_color=mix(hd.fresnel_color.rgb,team,clamp(hd.fresnel.y,0.0,1.0))*tint.rgb;
    return result;
}
