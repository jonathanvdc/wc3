# Renderer overview

bevy-wc3 renders Universal semantic models through Bevy's PBR, lighting, GPU
skinning, shadows, and render phases. Native adapters provide MDLX, M2, WMO, M3, and
Diablo input; a valid imported model can still contain shaders or effect programmes
without a complete interpreter. Source-associated diagnostics make those boundaries
inspectable.

## Geometry and animation

The compiler projects authored draw ranges, topology, and vertex/face/corner/
whole-mesh attributes into shared Bevy meshes. Complete vertex records retain
seams. Explicit and identity inverse binds remain distinct; skinning accepts up to
eight active influences and rejects larger distributions rather than truncating.
Each draw uses a compact palette containing its contributing joints, with matching
inverse binds. This reduces repeated skin preparation without removing authored
nodes or changing deformation.
Complete compiled mesh contents are interned so equivalent draws share GPU geometry,
while different tangent bases, attributes, and indices remain distinct.
Models convert declared axes and units into Bevy's frame beneath the application
root. `ModelDraw` preserves typed instance, mesh, material, draw-source, pass,
and LOD identities.

`Playback` selects typed clips and controller programmes in seconds. Shared
clocks remain independently sampleable. Local-pose transitions retain signed scale
and shortest-path quaternion rotation; surface and visibility properties sample the
destination directly. The shared `wc3` pose evaluator handles hierarchy, pivots, native billboard
conventions, camera anchoring, effect births, event poses, and offline baking. See
[animation](animation-blending.md) and [node transforms](node-flags.md).

Fixed or [automatic LOD](lod.md) selects authored geometry without recreating the
rig. [Mesh color and opacity](geoset-animation.md) remain independent across passes.
`PoseBaker` provides typed geometry/material snapshots without an ECS
world; its limits are described in the [application
guide](../usage.md#bake-a-static-pose).

## Materials and textures

Authored material programs contain dialect-specific steps, which the compiler
interprets into rendering passes. `ModelMaterial` extends Bevy `StandardMaterial` with authored blend/depth states
and supported native combiners. Classic layered surfaces, [M2 texture combinations](m2-materials.md),
[WMO surfaces](wmo-materials.md), [M3 common layers](m3-materials.md), and known Diablo roles map into that pipeline.
Unsupported combinations remain diagnostic fallbacks. Unlit and unfogged controls use
Bevy's corresponding behavior; mesh priority is a sort/depth bias, not a complete
game-specific pass ordering scheme.

[ReforgedDefault material steps](reforged-materials.md) support normal/ORM, team color,
emissive, Fresnel, and approximate reflection. Role-aware image requests preserve
samplers and color-space distinctions without changing shared source images.
Textures reference model-owned image declarations through typed handles; image
requests expose the resolved declaration to the application.
Resource replacement belongs to the application resolver; there is no implicit game
database or team-selection policy. Materials share assets between instances when
their compiled appearance is constant. Missing-channel bindings and scalar tracks
that always equal their authored defaults preserve sharing; changing UV transforms
and other animated appearance retain independent instance state.

WMO surfaces support established layered diffuse/emissive programmes, independent
UV streams and scroll clocks, and generated reflection coordinates. Separate
vertex-color streams control texture mixtures and indoor/outdoor lighting.
Native color preparation and regional ambient contributions feed the composed
surface through Bevy's emissive input, alongside ordinary scene lighting. Complex
terrain, window, LOD and modern extended paths remain partial or unsupported; see
[WMO materials](wmo-materials.md) for the bounded inventory and lighting limits.

## Scene features and effects

[Model assembly](assembly.md) combines shared skeletons, rigid mounted parts,
typed geometry choices and composed textures independently of game vocabulary.
The [WoW adapter](characters.md) supplies native character interpretation. Authored rigid-body
secondary motion is optional; native solver response remains approximate.

Attachment mounts retain the semantic definition, including the typed child
resource handle, in `ModelAttachment`; its separate resource field retains the
resolved child declaration after the source model is released.
`AttachmentState` contains sampled visibility and playback lifecycle state.
Authored children and assembly parts share `ModelMount`, with explicit node or
socket targets and authored or physical coordinate interpretation.
Common [lights](lights.md) become Bevy scene lights;
[cameras](cameras.md) bind opt-in to application-owned views. [Events](events.md)
publish typed occurrences for gameplay rather than executing commands. [Static-world
metadata](universal.md#static-worlds) exposes groups, portal-frustum selection, liquid grids, and
environment sampling without taking over scene fog or ambient lighting.

[Model particles](prem.md) instantiate child models. [Atlas sprites](pre2.md)
construct view-facing heads/tails on the GPU; M2 age-driven particles currently use
reusable CPU meshes with authored twinkle and independent secondary UV motion.
Supported M2 two/three-color particle programmes use the shared Bevy material
transport with unlit composition and matching alpha-discard prepasses. [Trails](ribbons.md) retain birth-time cross sections and share
GPU records across passes. Effect families have explicit simulation budgets and
separate unsupported controls.

## Coverage and limits

Support is assessed per concept, rather than inferred from successful native
loading. The [semantic guide](universal.md) explains resource providers and family
coverage; focused guides state the implemented behavior and remaining limits.

| Area | Current boundary |
| --- | --- |
| Geometry | More than eight skin influences fails; skinned bounds update conservatively from contributing joints. No generated simplification or per-view skeletal pose. |
| Surfaces | M2 illumination/constant programmes and the modern six-binding guild layout, specialized WMO terrain/window/LOD and extended combinations, specialized M3 families and advanced Standard layer controls, and external Diablo shaders need additional interpretation. |
| Scene features | Bevy lighting differs from native falloff and regional illumination. Unsupported feature hooks expose IDs/kinds, not complete owned definitions. |
| Effects | Spline/bone generators, advanced M2 texture programmes and modulation-gain controls, and several programmed simulations remain unsupported; external effects need providers. |
| Fidelity | Implemented motion, material, billboard, and native shader behavior is not a promise of exact game rendering. |

The shared [viewer and capture tool](../tools.md) uses this same renderer and
fixed-step simulation path.
