# Renderer shader sources

The material and vertex entry points adapt Bevy **v0.19.1** sources under
MIT OR Apache-2.0. Keep this pinned attribution when changing local shaders.

| Local file | Upstream source |
| --- | --- |
| `material.wgsl` | [`bevy_pbr/src/render/pbr.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/pbr.wgsl) |
| `material_prepass.wgsl` | [`bevy_pbr/src/render/pbr_prepass.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/pbr_prepass.wgsl) |
| `wc3_mesh.wgsl` | [`bevy_pbr/src/render/mesh.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/mesh.wgsl) |
| `wc3_prepass.wgsl` | [`bevy_pbr/src/prepass/prepass.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/prepass/prepass.wgsl) |

## Local integration

The model material extension preserves Bevy's PBR/light/post-processing
stages. `material.wgsl` evaluates supported native combiners and WMO
baked-light interpretation; `hd.wgsl` supplies DefaultUnit role sampling,
RG and M3 AG normal reconstruction, team masking, Fresnel, and environment approximation.
The prepass fragment retains matching alpha/UV/normal interpretation.

The vertex entry points retain their filenames while serving Universal geometry.
`extra_skinning.wgsl` shares the extra four joint influences between main and
prepass paths, including previous-frame deformation for motion vectors. Shader
locations 8 and 9 carry the second index/weight set when enabled by specialization.
`mesh_attributes.rs` defines the corresponding mesh attributes and
`material.rs` specializes their layouts.

Extension binding declarations live with `ModelMaterialExtension` and must
match the WGSL uniform/texture declarations. These shaders do not replace global
Bevy assets. Particle/ribbon transport has separate shader-generated geometry,
record layouts, and material bindings. See the [material guide](../../docs/rendering/reforged-materials.md)
and [architecture](../../docs/architecture.md) for runtime ownership and limits.

`combiner.wgsl` contains the M2 texture-composition equations shared by forward
and alpha-discard prepasses. The vertex wrappers generate sphere environment
coordinates, including the normal stream in depth-only layouts when required.
Native additive inputs enter PBR emissive; ordinary diffuse stays in Bevy albedo.

## Bevy upgrades

Compare each adapted entry point with its pinned upstream source when upgrading.
Include shadow/depth, normal, deferred, and motion-vector paths so auxiliary
passes agree with visible deformation and alpha. Update the attribution table
when the adaptations move to a new Bevy version.
