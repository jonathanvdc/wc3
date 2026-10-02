# General renderer visual fidelity

[Documentation index](../README.md)

These notes cover the current MDX/MDL renderer, including missing behavior for
Classic and Reforged models. Decoding a record or creating its animated node
does not imply that its rendering behavior is implemented. This is an
implementation inventory, not a claim of visual equivalence to Warcraft III.

## Missing or partial general rendering behavior

| Area | Current behavior and remaining work |
| --- | --- |
| Classic materials | Meshes use Bevy PBR shading with WC3 layer blend and depth states, rather than Warcraft's lighting equation. Mesh Unshaded/Unlit and Unfogged flags select Bevy's unlit/fog behavior; Classic sphere environment mapping is absent. PRE2 and ribbons do support unshaded rendering. |
| Reforged materials | HD DefaultUnit binds all six texture roles, animates material controls, and uses Bevy forward PBR. Fresnel/environment shading is approximate, game fidelity is unverified, and Crystal/unknown shaders use a warned diffuse fallback. See [Reforged materials](reforged-materials.md). |
| Geoset animation | Static and animated geoset color and alpha apply to all material layers, including visibility changes and per-geoset/instance material isolation. See [geoset animation](geoset-animation.md) for captures and remaining color-space, shadow, and game-fidelity checks. |
| Geoset UVs | Layer alpha and texture selection animate. Texture-animation translation, rotation, and scaling are supported for geosets and ribbons. Geoset layers select any available UV set through CoordinateId mesh variants, with matching tangent bases. Game UV-transform fidelity remains unverified. |
| Node flags | CPU evaluation supports selective translation/rotation/scale inheritance, full/axis-locked billboards, and camera anchoring through an ordinary Bevy hierarchy. Each instance uses one configurable driving camera; per-view skeletal poses are absent. Game semantics remain unverified, especially anchoring and combined flags. See [node flags](node-flags.md). |
| WC3 pass ordering | Geometry uses Bevy render phases and WC3 layer blend/depth states. Warcraft's ordering across material layers and effects is not implemented as a dedicated ordering scheme. Mesh PriorityPlane uses an approximate Bevy depth/sort bias; SortPrimsFarZ/SortPrimsNearZ are not applied. PRE2 and ribbons have their own sorting/bias behavior, with remaining fidelity checks described below. |
| Skinning limits | GPU skinning supports up to eight influences per vertex. Larger Classic matrix groups are rejected; they need a rendering strategy before they can be displayed. |
| Explicit bind poses | Inverse bind matrices are constructed from bone pivots. Model bind-pose matrix records are not consumed. |
| LOD | Preparation keeps LOD 0/default geosets and skips other levels. Runtime LOD selection is not implemented. |
| Replaceable textures | Explicit per-instance replaceable and slot bindings work. Automatic team-color/glow and other game-specific replaceable texture selection are not implemented; the consumer supplies images. |
| Model lights | Animated point/directional records create ordinary Bevy scene lights, with configurable power/range scales and authored shadow-casting flags. Ambient contributions, attenuation start, Reforged falloff, shadow intensity/ranges, and unknown types have no mapping. See [model lights](lights.md) for integration semantics, captures, and fidelity limits. |
| Model cameras | Opt-in bindings drive application-owned perspective cameras from authored eye/target animation, roll, FOV, and clips. Visibility switching and modern lens/DOF tracks are not mapped; game fidelity is unverified. See [model cameras](cameras.md). |
| Event objects | Event nodes exist, but event tracks are not dispatched. Event-driven visual effects are not implemented. |
| Reforged effects | Popcorn emitters and FaceFX playback are not implemented. |

## Effects and remaining fidelity checks

The effect implementations have dedicated notes distinguishing missing behavior
from implemented behavior that still needs comparison against Warcraft III:

- [PRE2 particles](pre2.md): instanced head/tail quads are implemented. Fog is
  absent, squirt updates can miss crossed keys or loops, and continuous emission
  still needs interval integration of emission rate and visibility. Lighting,
  priority ordering, gravity space, orientation, and mirrored transforms need
  fidelity checks.
- [Classic PREM particles](prem.md): model particles spawn and simulate
  world-space motion, independent animation, and lifetime cleanup. Image-based
  EmitterUsesTga particles remain unsupported. Longitude is ignored, and emission
  rate/visibility use update-endpoint sampling. Exact game fidelity remains
  unverified.
- [Ribbons](ribbons.md): GPU trails support birth-time node poses, animated
  heights/color/alpha/atlas slots, UV transforms, and material layers. Brief hidden
  intervals can go undetected, and every pass uses the transparent phase. Fog and
  environment mapping are absent; lighting, gravity, density, UV direction, and
  material semantics still need game comparisons.

Attachments are implemented: resolved paths spawn independently animated child
models, visibility tracks gate separate mounts, and children follow node
transforms, loop their first sequence, restart on show/parent sequence changes,
and clean up with their owner. Points are exposed by ID/name for
consumer-supplied models. Attached and PREM child models inherit the general
renderer limitations above.

Geoset color/alpha, eight-influence skinning, attachments, PRE2 heads/tails, model-based
PREM, and ribbons should not be listed as entirely missing features.

The existing `capture` example produces images for visual verification.
Synthetic fixtures establish controlled behavior, while Warcraft III captures
are needed to verify the exact game appearance.
