# Renderer shader sources

The material and vertex entry points adapt Bevy **v0.19.1** sources under
MIT OR Apache-2.0. Keep this pinned attribution when changing local shaders.

| Local file | Upstream source |
| --- | --- |
| `material.wgsl` | [`bevy_pbr/src/render/pbr.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/pbr.wgsl) |
| `material_prepass.wgsl` | [`bevy_pbr/src/render/pbr_prepass.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/pbr_prepass.wgsl) |
| `wc3_mesh.wgsl` | [`bevy_pbr/src/render/mesh.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/mesh.wgsl) |
| `model_io.wgsl` | [`bevy_pbr/src/render/forward_io.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/forward_io.wgsl) and [`bevy_pbr/src/prepass/prepass_io.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/prepass/prepass_io.wgsl) |
| `wc3_prepass.wgsl` | [`bevy_pbr/src/prepass/prepass.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/prepass/prepass.wgsl) |

## Local integration

The model material extension preserves Bevy's PBR/light/post-processing
stages. `material.wgsl` evaluates supported native combiners and WMO
baked-light interpretation; `hd.wgsl` supplies DefaultUnit role sampling,
RG normal reconstruction, team masking, Fresnel, and environment approximation.
The prepass fragment retains matching alpha/UV/normal interpretation.

The vertex entry points retain their filenames while serving Universal geometry.
`extra_skinning.wgsl` shares the extra four joint influences between main and
prepass paths, including previous-frame deformation for motion vectors. Shader
locations 8 and 9 carry the second index/weight set when enabled by specialization.
`mesh_attributes.rs` defines the corresponding mesh attributes and
`material.rs` specializes their layouts. An optional third UV stream uses
attribute and varying location 10. `model_io.wgsl` retains the pinned Bevy
conditional output members, adds that stream, and explicitly converts ordinary
fields back into Bevy's input type before calling its PBR functions. Edge-opacity
variants add a facing factor at varying location 11; ordinary variants omit it.

Extension binding declarations live with `ModelMaterialExtension` and must
match the WGSL uniform/texture declarations. These shaders do not replace global
Bevy assets. Particle/ribbon transport has separate shader-generated geometry,
record layouts, and material bindings. `effect_lighting.wgsl` shares their Bevy
lighting, blend-aware distance fog, tonemapping, and debanding stages. Unlit effects
still follow camera output processing, and authored unfogged controls remain
independent of lighting. See the [material guide](../../docs/rendering/reforged-materials.md)
and [architecture](../../docs/architecture.md) for runtime ownership and limits.

`combiner.wgsl` contains the M2 texture-composition equations shared by forward
and alpha-discard prepasses. The vertex wrappers generate sphere environment
coordinates, including the normal stream in depth-only layouts when required.
Native additive inputs enter PBR emissive; ordinary diffuse stays in Bevy albedo.

`wc3_age_particle.wgsl` expands exactly sampled M2 live records against each
camera and evaluates independent two-color/three-alpha or three-color/three-alpha
particle composition. The primary image retains atlas cells; additional images
use their own offsets and scales. RGB and alpha gains remain independent, and
multi-texture cutout tests texture alpha before vertex opacity. Opaque TXAC
selectors do not override recognized composition flags. Shared records retain
appearance and analytic motion sampling on the CPU while view-specific indices select head/tail geometry
and sorting. All eight native particle blend states remain explicit.

## Bevy upgrades

Compare each adapted entry point with its pinned upstream source when upgrading.
Include shadow/depth, normal, deferred, and motion-vector paths so auxiliary
passes agree with visible deformation and alpha. Update the attribution table
when the adaptations move to a new Bevy version.
