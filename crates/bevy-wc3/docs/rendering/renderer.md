# Renderer coverage

`bevy-wc3` renders Warcraft III MDX/MDL and World of Warcraft M2/WMO through Bevy's
PBR, lighting, skinning, and render phases. Each native adapter supplies a
`Model<Universal>` to the same compiler. Geometry coverage is broader than native
shader and effect coverage: a model can load successfully and still need an
additional interpreter. Inspect its rendering diagnostics when checking support.

## Geometry and motion

The compiler retains authored draw ranges, seams, topology, and attribute domains.
GPU skinning accepts up to eight active influences per vertex and rejects larger
distributions. Equal compiled meshes and compact bind palettes share assets;
instances retain independent poses and clocks. Native axes and declared units
convert beneath the application root.

[Playback](animation-blending.md) selects clips in seconds, including their source
intervals, loop/clamp policies, and independent shared clocks. Transitions blend
local translation, signed scale, and shortest-path rotation. Materials, visibility,
and other scalar properties sample the destination directly. The shared `wc3`
pose evaluator handles pivots, [node inheritance and facing](node-flags.md), effect
births, event poses, and offline baking.

[LOD](lod.md) selects authored geometry; it does not generate simplified meshes or
reduce effect budgets. [Mesh color and opacity](geoset-animation.md) animate
independently across passes. The [pose baker](../usage.md#bake-a-static-pose)
produces geometry and material snapshots without a live ECS hierarchy.

## Materials

Authored material steps compile into Bevy material passes. Supported native texture
combinations run within `ModelMaterial`, an extension of `StandardMaterial`.
Blend/depth state, animated images, UV transforms, and color/opacity remain
explicit. Faded M2 opaque and cutout surfaces enter Bevy's transparent phase,
retaining their authored depth operations and texture coverage. Constant materials
share assets; animated appearances retain independent instance state.

Each material or effect input owns its sampling settings. Image requests preserve
sampler and color-space distinctions, and equal resolved interpretations share GPU
images. Applications supply replacements and resource lookup; the renderer supplies
no game database or team-selection policy.

Choose the family-specific guide for supported equations and fallbacks:

- [Reforged](reforged-materials.md): DefaultUnit and Crystal HD roles, view-dependent diffuse composition, Fresnel, and PBR approximations.
- [M2](m2-materials.md): texture combiners, generated coordinates, weighted crossfades, and edge opacity; the [shader inventory](m2-programmes.md) lists bounded coverage.
- [WMO](wmo-materials.md): layered surfaces, baked vertex illumination, and regional ambient contributions.

Bevy render phases may reorder draws across authored steps. Priority is a sort/depth
bias; a complete native ordering scheme across meshes and effects is absent.
Lighting uses Bevy's scene model and can differ from native falloff, specular
response, and indoor/outdoor illumination.

## Scene integration

[Assemblies](assembly.md) combine shared skeletons, mounted parts, geometry choices,
and composed textures. The optional [WoW adapter](characters.md) interprets native
appearance choices. [Authored secondary motion](physics.md) is opt-in: the
approximate solver retains compound fixtures, analytical capsule contacts, bounded
affine support-map contacts for hulls and other primitive pairs, tensor inertia
and contact friction. Numerical and work-limit failures omit support queries with
typed per-instance statistics; stable manifolds and native stepping remain partial.
Contact admission defaults to authored grants; the explicit unconnected-dynamic
policy derives bounded source-local pairs without inventing native filtering or
ground.

Attachment mounts expose animated sockets and can load child models automatically.
[Lights](lights.md) preserve activation independently of shadow parameters and
become ordinary Bevy lights. [Cameras](cameras.md) retain independent projection,
role, visibility, and lens parameters, and bind to
application-owned views on request. [Events](events.md) publish occurrences
for application handling. [Static-world APIs](worlds.md) expose groups, portal
selection, placement sets, liquids, and regional environment samples while leaving
scene lighting, fog, and camera containment to the application. Dedicated WMO
shadow batches remain geometry for Bevy's light-shadow views and are excluded from
camera color and prepasses.

## Effects

The renderer keeps native emitter families distinct while sharing transport where
possible. Sprite and ribbon emitters share immutable vertex-selector geometry;
external capsule billboards use a shared four-triangle selector. They participate
in Bevy visibility and render layers through a procedural geometry component.
Their resident records define the world geometry, so these emitters do not create
ordinary mesh-instance uniforms. Model-particle child geometry uses the usual
mesh pipeline. The effect guides describe simulation, playback, and resource needs:

- [Model particles](prem.md) instantiate independent animated MDLX PREM and M2 geometry-model children.
- [Sprite particles](pre2.md) use GPU-generated heads/tails from immutable MDLX births or compact M2 age-driven samples.
  Billboards and draw ordering follow each view independently; M2 emitter-template resources drive shared child pools without creating model actors.
- [Ribbons](ribbons.md) retain birth-time cross sections and generate connected geometry on the GPU.
- [External programmes](popcorn.md) execute supported baked graphs and share effect geometry, lighting and ordering.
  Outputs use CPU samples, or explicitly selected immutable appearance for closed billboard expressions,
  including moving centers and authored axes. CPU position projections retain per-view ordering.

Each family bounds populations and per-update work and exposes omitted-work
counters. Nested model particles also share configurable actor and emission-work
limits with their particle-owned attachments and rigid parts. M2 multi-texture
colors and alpha masks retain their independent
modulation gains. Bone generators use declared section palettes and posed joint
spans; their exact game distribution remains unverified. Particle refraction uses
conditional per-view masks and scene composition. Native TXAC selectors still
require further interpretation. External effects need resource providers; their
programme and renderer interpreters have partial coverage.

## Check a particular model

Use the [viewer and capture tools](../tools.md) to inspect loading diagnostics and
compare controlled poses. Compilation and nonempty images do not establish native
game fidelity. Some implemented behavior, including native billboard, lighting,
and effect conventions, still needs broader validation against game content.
Focused guides distinguish implemented behavior, partial coverage, and missing
features. [Compilation and native resources](universal.md) explains strict import
policy and diagnostics in more detail.
