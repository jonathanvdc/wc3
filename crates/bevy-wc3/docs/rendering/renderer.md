# Renderer overview

[Documentation index](../README.md)

bevy-wc3 turns a decoded Warcraft III model into shared Bevy assets and an
independently animated entity hierarchy for each instance. The renderer uses
Bevy lighting, GPU skinning, and render phases; effect renderers generate their
geometry from immutable birth records.

## Geometry and skinning

Preparation builds meshes for LOD 0/default geosets and selects each layer’s
`CoordinateId` UV set. Layers using identical UV arrays can share mesh variants.
Authored UV0 tangents are preserved; other sets receive a matching generated
basis. GPU skinning supports up to eight influences per vertex. Joint ordering
and pivot-derived inverse bind matrices are shared across instances.

Spawning creates animated nodes and one mesh pass per material layer. Each
instance owns its rig, material handles, texture bindings, and animation clock.
[Architecture](../architecture.md) describes preparation and ownership in detail.

## Animation and node transforms

Sequence and global-sequence clocks sample node transforms and animated model
properties. Sequence changes blend authored local poses using model BlendTime;
translation and scale interpolate linearly and rotation uses quaternion slerp.
Materials and other non-transform properties sample the destination directly.
See [animation blending](animation-blending.md).

The CPU pose evaluator handles pivots, selective inheritance, full and
axis-locked billboards, and camera anchoring. Each instance selects one driving
camera, also used when sampling effect births and event poses. See
[node flags](node-flags.md). [Geoset animation](geoset-animation.md) applies
color and alpha across every layer with per-geoset material isolation.

## Materials and textures

Classic mesh layers use Bevy PBR with WC3 blend, alpha-test, culling, and depth
states. Unshaded/Unlit selects unlit shading; Unfogged disables Bevy fog.
Layer alpha, texture selection, and UV translation/rotation/scaling animate.
PriorityPlane provides a Bevy depth/sort bias.

[Reforged DefaultUnit materials](reforged-materials.md) bind diffuse, normal,
ORM, emissive, team-color, and environment roles in one normalized layer pass.
Surface controls animate on the instance clock. Normal and ORM maps use private
linear image variants, preserving the original image for other consumers.

Texture paths resolve beside the model and then at the asset root. At each
location, lookup tries the literal filename followed by `.blp`, `.dds`, `.png`,
and `.tga` alternatives. The first readable file wins; decoding errors do not
trigger further fallback. Applications
choose replaceable textures and exact bitmap/emitter overrides with
`Wc3TextureBindings`; slot overrides take precedence. See the
[texture guide](../usage.md#choose-textures).

## Attachments, lights, cameras, and events

Attachment paths spawn child models on animated mounts. Mount visibility gates
attached content, and children loop sequence zero with their own rigs and
materials. Showing a mount, changing the parent sequence, or seeking backward
restarts attachment playback. Points are also exposed by ID and name for
application-supplied models. Owned child models are cleaned up with their root.
See [attachments](../usage.md#mount-models-on-attachment-points).

[Point and directional lights](lights.md) become animated Bevy scene lights
with configurable power/range conversion and authored shadow-casting flags.
[Camera bindings](cameras.md) play authored eye/target, roll, FOV, and clipping
tracks on application-owned perspective cameras. [Event objects](events.md)
emit crossed-key messages with occurrence-time poses for application handling.

## Effects

- [Classic PREM](prem.md) spawns model particles with sampled birth transforms,
  world-space ballistic motion, independent animation, and lifetime cleanup.
- [PRE2](pre2.md) renders GPU-instanced head and tail quads. The CPU samples
  births; shaders evaluate motion, lifetime curves, atlas UVs, and view-facing
  geometry. ModelSpace and XYQuad control motion and orientation.
- [Ribbons](ribbons.md) retain world-space cross-sections sampled at subframe
  births. GPU passes construct connected trails, apply gravity, and animate
  atlas cells and material layers without rebuilding CPU meshes.

Root visibility and ownership integrate geometry and effects into Bevy’s entity
lifecycle. Each effect topic documents its playback and visibility rules.

## Implementation limits

| Area | Limit |
| --- | --- |
| Skinning | More than eight influences is rejected. Explicit model bind-pose records are not consumed. |
| LOD | Preparation selects LOD 0/default; there is no runtime LOD switching. |
| Materials and ordering | Classic sphere environment mapping and a dedicated WC3 ordering scheme across meshes/effects are absent. Mesh SortPrimsFarZ/NearZ is not applied. Crystal and unknown shaders use a warned diffuse fallback. |
| Lights and cameras | Ambient light records and several Reforged attenuation/shadow controls have no native mapping. Camera visibility and modern lens/DOF tracks do not control views. |
| Effects | Image-based PREM, Popcorn, and FaceFX are unsupported. PRE2/ribbons do not apply fog. Emission sampling limits are described in the effect topics. |

## Rendering checks

The existing `capture` example renders MDX and MDL through the plugin at
controlled simulation times and camera settings. See the
[application guide](../usage.md#inspect-and-capture-models) for commands and
[verification notes](../verification.md) for earlier test and capture reports.
The lighting, transition, and ordering rules described here are this renderer’s
semantics; exact Warcraft III appearance remains unverified.
