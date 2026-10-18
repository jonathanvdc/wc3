# Renderer overview

bevy-wc3 renders Universal semantic models through Bevy's PBR, lighting, GPU
skinning, shadows, and render phases. Native adapters provide MDLX, M2, and WMO input; a valid imported model can still contain shaders or effect programmes
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
Models convert their native coordinate conventions into Bevy's frame beneath
the application root. `ModelDraw` preserves typed instance, mesh, material, draw-source, pass,
and LOD identities.

`Playback` selects typed clips in seconds. Their curves
and event occurrences live in reusable semantic `Animation` definitions. Each
occurrence references a model-owned source defining its action, anchor, and clock. The shared sampler
adds each clip's source start to local playback time and selects keys within its
inclusive source interval. Shared
clocks remain independently sampleable. Local-pose transitions retain signed scale
and shortest-path quaternion rotation; surface and visibility properties sample the
destination directly. The shared `wc3` pose evaluator handles hierarchy, pivots, explicit transform and facing
semantics, camera anchoring, effect births, event poses, and offline baking. See
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
and [WMO surfaces](wmo-materials.md) map into that pipeline.
MDLX steps separate pass state from typed shader inputs, using the same step
vocabulary in the MDLX profile and Universal. Native selector decoding belongs
to the model adapters for every dialect. M2 native layers become ordered material steps with
independent animation targets. Unsupported combinations remain diagnostic fallbacks. Unlit and unfogged controls use
Bevy's corresponding behavior; mesh priority is a sort/depth bias, not a complete
game-specific pass ordering scheme.

[ReforgedDefault material steps](reforged-materials.md) support normal/ORM, team color,
emissive, Fresnel, and approximate reflection. Role-aware image requests preserve
samplers and color-space distinctions without changing shared source images.
Material and effect inputs reference model-owned image declarations through typed
handles and own their sampling and coordinate transforms. Accompanying native
filenames live in those resource declarations. Native shared texture
and transform records become independent input values during import. Image
requests expose the selected value and resolved declaration to the application;
equal resolved resources, samplers, and transfer functions share GPU images.
Resource replacement belongs to the application resolver; there is no implicit game
database or team-selection policy. Materials share assets between instances when
their compiled appearance is constant. Unanimated slots and scalar tracks
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

Universal retains complete MDLX and M2 particle and ribbon definitions. The
renderer interprets each family and shares transport mechanics where their
behavior agrees. [MDLX model particles](prem.md) instantiate child models. [MDLX atlas sprites](pre2.md)
construct view-facing heads/tails on the GPU; M2 particles currently use
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
| Surfaces | M2 illumination/constant programmes and the modern six-binding guild layout, specialized WMO terrain/window/LOD and extended combinations need additional interpretation. |
| Scene features | Bevy lighting differs from native falloff and regional illumination. Unsupported feature hooks expose IDs/kinds, not complete owned definitions. |
| Effects | Spline/bone generators, advanced M2 texture programmes and modulation-gain controls remain unsupported; external effects need providers. |
| Fidelity | Implemented motion, material, billboard, and native shader behavior is not a promise of exact game rendering. |

The shared [viewer and capture tool](../tools.md) uses this same renderer and
fixed-step simulation path.
