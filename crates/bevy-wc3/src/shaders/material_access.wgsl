// One access boundary for Bevy-managed bindless slabs and cached uniform fallback.
#import bevy_pbr::{pbr_bindings, mesh_bindings::mesh, pbr_types::StandardMaterial}
#ifdef BINDLESS
#import bevy_render::bindless::{bindless_textures_2d,bindless_samplers_filtering}
#endif

struct Combiner {
    uv_transform: mat3x3<f32>,
    third_transform: mat3x3<f32>,
    fourth_transform: mat3x3<f32>,
    additional_coordinates: vec4<u32>,
    coordinates: vec4<u32>,
    weights: vec4<f32>,
    guild_background: vec4<f32>,
    guild_emblem: vec4<f32>,
    guild_border: vec4<f32>,
    mode: vec4<u32>,
    clip: vec4<f32>,
};

struct HdUniform {
    fresnel_color: vec4<f32>,
    fresnel: vec4<f32>,
    maps: vec4<u32>,
};

struct World {
    composition: vec4<u32>,
    flags: vec4<u32>,
    ambient: vec4<f32>,
};

struct MaterialData {
    combiner: Combiner,
    hd: HdUniform,
    world: World,
};
#ifdef BINDLESS

struct MaterialIndices {
    data: u32,
    secondary: u32,
    secondary_sampler: u32,
    orm: u32,
    orm_sampler: u32,
    team: u32,
    team_sampler: u32,
    environment: u32,
    environment_sampler: u32,
    tertiary: u32,
    tertiary_sampler: u32,
    fourth: u32,
    fourth_sampler: u32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<storage> extension_indices: array<MaterialIndices>;
@group(#{MATERIAL_BIND_GROUP}) @binding(116) var<storage> extension_data: array<MaterialData>;

fn material_slot(instance: u32) -> u32 {
    return mesh[instance].material_and_lightmap_bind_group_slot & 0xffffu;
}

fn load_combiner(instance: u32) -> Combiner {
    return extension_data[extension_indices[material_slot(instance)].data].combiner;
}

fn load_hd(instance: u32) -> HdUniform {
    return extension_data[extension_indices[material_slot(instance)].data].hd;
}

fn load_world(instance: u32) -> World {
    return extension_data[extension_indices[material_slot(instance)].data].world;
}

fn load_standard(instance: u32) -> StandardMaterial {
    return pbr_bindings::material_array[pbr_bindings::material_indices[material_slot(instance)].material];
}
#else
@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> combiner: Combiner;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var<uniform> hd: HdUniform;
@group(#{MATERIAL_BIND_GROUP}) @binding(110) var<uniform> world: World;

fn load_combiner(instance: u32) -> Combiner {
    return combiner;
}

fn load_hd(instance: u32) -> HdUniform {
    return hd;
}

fn load_world(instance: u32) -> World {
    return world;
}

fn load_standard(instance: u32) -> StandardMaterial {
    return pbr_bindings::material;
}

@group(#{MATERIAL_BIND_GROUP}) @binding(101) var secondary_image: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var secondary_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var orm_image: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(105) var orm_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(106) var team_image: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(107) var team_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(108) var environment_image: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(109) var environment_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(111) var tertiary_image: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(112) var tertiary_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(114) var fourth_image: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(115) var fourth_sampler: sampler;
#endif

fn sample_secondary(uv: vec2<f32>, instance: u32) -> vec4<f32> {
#ifdef BINDLESS
    let indices = extension_indices[material_slot(instance)];
    return textureSample(
        bindless_textures_2d[indices.secondary],
        bindless_samplers_filtering[indices.secondary_sampler],
        uv
    );
#else
    return textureSample(secondary_image, secondary_sampler, uv);
#endif
}

fn sample_orm_bias(uv: vec2<f32>, instance: u32, bias: f32) -> vec4<f32> {
#ifdef BINDLESS
    let indices = extension_indices[material_slot(instance)];
    return textureSampleBias(
        bindless_textures_2d[indices.orm],
        bindless_samplers_filtering[indices.orm_sampler],
        uv,
        bias
    );
#else
    return textureSampleBias(orm_image, orm_sampler, uv, bias);
#endif
}

fn sample_team_bias(uv: vec2<f32>, instance: u32, bias: f32) -> vec4<f32> {
#ifdef BINDLESS
    let indices = extension_indices[material_slot(instance)];
    return textureSampleBias(
        bindless_textures_2d[indices.team],
        bindless_samplers_filtering[indices.team_sampler],
        uv,
        bias
    );
#else
    return textureSampleBias(team_image, team_sampler, uv, bias);
#endif
}

fn sample_environment_bias(uv: vec2<f32>, instance: u32, bias: f32) -> vec4<f32> {
#ifdef BINDLESS
    let indices = extension_indices[material_slot(instance)];
    return textureSampleBias(
        bindless_textures_2d[indices.environment],
        bindless_samplers_filtering[indices.environment_sampler],
        uv,
        bias
    );
#else
    return textureSampleBias(environment_image, environment_sampler, uv, bias);
#endif
}

fn sample_tertiary(uv: vec2<f32>, instance: u32) -> vec4<f32> {
#ifdef BINDLESS
    let indices = extension_indices[material_slot(instance)];
    return textureSample(
        bindless_textures_2d[indices.tertiary],
        bindless_samplers_filtering[indices.tertiary_sampler],
        uv
    );
#else
    return textureSample(tertiary_image, tertiary_sampler, uv);
#endif
}

fn sample_fourth(uv: vec2<f32>, instance: u32) -> vec4<f32> {
#ifdef BINDLESS
    let indices = extension_indices[material_slot(instance)];
    return textureSample(
        bindless_textures_2d[indices.fourth],
        bindless_samplers_filtering[indices.fourth_sampler],
        uv
    );
#else
    return textureSample(fourth_image, fourth_sampler, uv);
#endif
}

fn sample_base(uv: vec2<f32>, instance: u32) -> vec4<f32> {
#ifdef BINDLESS
    let indices = pbr_bindings::material_indices[material_slot(instance)];
    return textureSample(
        bindless_textures_2d[indices.base_color_texture],
        bindless_samplers_filtering[indices.base_color_sampler],
        uv
    );
#else
    return textureSample(pbr_bindings::base_color_texture, pbr_bindings::base_color_sampler, uv);
#endif
}

fn sample_normal_bias(uv: vec2<f32>, instance: u32, bias: f32) -> vec4<f32> {
#ifdef BINDLESS
    let indices = pbr_bindings::material_indices[material_slot(instance)];
    return textureSampleBias(
        bindless_textures_2d[indices.normal_map_texture],
        bindless_samplers_filtering[indices.normal_map_sampler],
        uv,
        bias
    );
#else
    return textureSampleBias(pbr_bindings::normal_map_texture, pbr_bindings::normal_map_sampler, uv, bias);
#endif
}
