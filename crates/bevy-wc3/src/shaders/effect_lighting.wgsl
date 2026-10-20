#import bevy_pbr::{
    mesh_view_bindings::view,
    pbr_types::pbr_input_new,
    mesh_types::MESH_FLAGS_SHADOW_RECEIVER_BIT,
    pbr_functions::{apply_pbr_lighting, calculate_view, main_pass_post_lighting_processing},
}
#ifdef DISTANCE_FOG
#import bevy_pbr::{mesh_view_bindings::fog, pbr_functions::apply_fog}
#endif

// Procedural effect geometry joins the same lighting and output stages as meshes.
fn shade_effect(
    color: vec4<f32>,
    position: vec3<f32>,
    normal: vec3<f32>,
    frag_coord: vec4<f32>,
    unlit: bool,
    fog_enabled: bool,
    fog_blend: u32,
) -> vec4<f32> {
    var lighting = pbr_input_new();
    lighting.material.base_color = color;
    lighting.material.reflectance = vec3(0.0);
    lighting.material.perceptual_roughness = 1.0;
    // Fog is applied below with a blend-neutral target before output conversion.
    lighting.material.flags = 0u;
    lighting.flags = MESH_FLAGS_SHADOW_RECEIVER_BIT;
    lighting.frag_coord = frag_coord;
    lighting.world_position = vec4(position, 1.0);
    lighting.world_normal = normal;
    lighting.N = normal;
    lighting.is_orthographic = view.clip_from_view[3].w == 1.0;
    lighting.V = calculate_view(lighting.world_position, lighting.is_orthographic);
    var output = color;
    if !unlit {
        output = vec4(apply_pbr_lighting(lighting).rgb, color.a);
    }
#ifdef DISTANCE_FOG
    if fog_enabled && fog_blend == 0u {
        output = apply_fog(fog, output, position, view.world_position.xyz, frag_coord.xy);
    }
#endif
    output = main_pass_post_lighting_processing(lighting, output);
#ifdef DISTANCE_FOG
    if fog_enabled && fog_blend != 0u {
        // Framebuffer identities must survive camera tonemapping and dithering.
        var neutral = vec3(0.0);
        if fog_blend == 2u { neutral = vec3(1.0); }
        if fog_blend == 3u { neutral = vec3(0.5); }
        var parameters = fog;
        parameters.base_color = vec4(neutral, parameters.base_color.a);
        parameters.directional_light_color = vec4(0.0);
        output = apply_fog(parameters, output, position, view.world_position.xyz, frag_coord.xy);
    }
#endif
    return output;
}
