// Shared HD PBR interpretation, extracted from this project's WC3 material shader.
#import bevy_pbr::pbr_types::PbrInput
#import "embedded://bevy_wc3/shaders/model_io.wgsl"::VertexOutput
#import "embedded://bevy_wc3/shaders/material_access.wgsl"::{load_hd,load_standard,sample_normal_bias,sample_team_bias,sample_orm_bias,sample_environment_bias}
#import bevy_pbr::{pbr_bindings, pbr_functions, pbr_types, mesh_view_bindings::view}

fn apply_hd(in: VertexOutput, is_front: bool, input: PbrInput) -> PbrInput {
    var result = input;
    let hd = load_hd(in.instance_index);
    let material = load_standard(in.instance_index);
    // HD maps use a shared, animated UV transform. Normal RG reconstructs Z
    // even when the source texture has RGB(A) channels.
#ifdef VERTEX_UVS_A
    if hd.maps.x != 0u {
        let uv = (material.uv_transform * vec3(in.uv, 1.0)).xy;
#ifdef STANDARD_MATERIAL_NORMAL_MAP
#ifdef VERTEX_TANGENTS
        let sampled = sample_normal_bias(uv, in.instance_index, view.mip_bias);
        let packed=select(sampled.rg,sampled.ag,hd.maps.x==2u);
        let xy = packed * 2.0 - 1.0;
        let nt = vec3(xy, sqrt(max(0.0, 1.0 - dot(xy, xy)))) * 0.5 + vec3(0.5);
        let flags = result.material.flags & ~pbr_types::STANDARD_MATERIAL_FLAGS_TWO_COMPONENT_NORMAL_MAP;
        let double_sided = (flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;
        let tbn = pbr_functions::calculate_tbn_mikktspace(in.world_normal, in.world_tangent);
        result.N = pbr_functions::apply_normal_mapping(flags, tbn, double_sided, is_front, nt);
#endif
#endif
        if hd.maps.x == 1u {
        var team = vec3(1.0);
        if hd.maps.z != 0u {
            team = sample_team_bias(uv, in.instance_index, view.mip_bias).rgb;
        }
        if hd.maps.y != 0u && hd.maps.z != 0u {
            let mask = sample_orm_bias(uv, in.instance_index, view.mip_bias).a;
            // Team color continuously modulates diffuse RGB through the ORM
            // alpha mask while preserving diffuse alpha.
            result.material.base_color = vec4(result.material.base_color.rgb
                * mix(vec3(1.0), team, mask), result.material.base_color.a);
        }
        let rim = pow(1.0 - clamp(dot(result.N, result.V), 0.0, 1.0), 5.0);
        let fresnel_color = mix(hd.fresnel_color.rgb, team, clamp(hd.fresnel.y, 0.0, 1.0));
        var reflection = vec3(0.0);
        if hd.maps.w != 0u {
            let world_direction = normalize(reflect(-result.V, result.N));
            // Environment maps use Z-up coordinates; the scene uses Bevy Y-up.
            let direction = vec3(world_direction.x, -world_direction.z, world_direction.y);
            let env_uv = vec2(atan2(direction.x, direction.y), -asin(clamp(direction.z, -1.0, 1.0)))
                * vec2(0.15915494, 0.31830989) + vec2(0.5);
            let environment = sample_environment_bias(
                env_uv,
                in.instance_index,
                view.mip_bias + result.material.perceptual_roughness * 5.0
            ).rgb;
            let f0 = mix(vec3(0.04), result.material.base_color.rgb, result.material.metallic);
            reflection = environment * mix(f0, vec3(1.0), rim)
                * pow(1.0 - result.material.perceptual_roughness, 2.0);
        }
        // The stored controls drive an explicit approximation, independent of
        // Bevy's physical reflectance. Exact game Fresnel/IBL remains unverified.
        result.material.emissive = vec4(result.material.emissive.rgb
            + (fresnel_color * rim * hd.fresnel.x + reflection)
                * material.base_color.rgb, result.material.emissive.a);
        }
    }
#endif

    return result;
}
