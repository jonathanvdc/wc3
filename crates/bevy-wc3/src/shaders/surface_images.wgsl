// Shared extension image bindings, reused by mutually exclusive surface families.
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var orm_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(105) var orm_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(106) var team_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(107) var team_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(108) var environment_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(109) var environment_sampler: sampler;
