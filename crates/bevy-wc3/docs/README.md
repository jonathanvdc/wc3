# bevy-wc3 documentation

Start with the [crate README](../README.md) for a minimal Bevy application. These
guides separate application setup, model inspection, rendering behavior, and
implementation details so you can choose the depth needed for your task.

## Application and architecture

The application guide covers runtime integration; the tool guide helps inspect
assets before integrating them. Architecture is intended for contributors and
applications that need explicit control over scheduling or preparation.

- [Using bevy-wc3](usage.md): load native/semantic instances, resolve resources and images, control typed
  playback, bake poses, mount child models, and manage replacement.
- [Viewer and captures](tools.md): inspect assets, override textures, and render
  PNGs at controlled simulation times, including synchronized frame measurements.
- [Architecture](architecture.md): semantic loading and preparation, shared and per-instance
  state, renderer modules, effect record storage, and the `Wc3Systems` scheduling
  contract.
- API reference: `cargo doc -p bevy-wc3 --no-deps --open`.

## Rendering

The files under `rendering/` describe how the renderer evaluates model data and
draws it through Bevy. Start with the [renderer overview](rendering/renderer.md),
then choose a topic:

- [Semantic compiler and native resources](rendering/universal.md): Universal compilation,
  native resource providers, semantic animation, and family-specific interpretation limits.
- [Model assembly](rendering/assembly.md): application-defined selections, typed
  geometry and texture choices, shared skeletons and rigid components.
- [WoW characters and equipment](rendering/characters.md): the optional native
  table adapter, geoset interpretation, joint matching and secondary motion.
- [Animation blending](rendering/animation-blending.md): typed clip controls,
  local-pose transitions, interruption, and effect/event sampling.
- [Node transforms and flags](rendering/node-flags.md): inheritance,
  billboarding, camera anchoring, and driving-camera selection.
- [Geometry LOD](rendering/lod.md): authored levels, automatic screen-size
  selection, quality controls, and camera policy.
- [Geoset animation](rendering/geoset-animation.md): color, opacity,
  material sharing, isolation, and sampling.
- [M2 materials](rendering/m2-materials.md): native texture combiners, generated
  coordinates, weighted crossfades, edge opacity and PBR integration limits.
- [M2 programme inventory](rendering/m2-programmes.md): complete known shader-table
  coverage, texture and palette requirements, and bounded unresolved families.
- [Reforged materials](rendering/reforged-materials.md): texture roles,
  surface animation, UV sets, and Bevy shading integration.
- [WMO materials](rendering/wmo-materials.md): layered texture programmes, native
  vertex-light preparation, regional ambient contributions and scene limits.
- [M3 materials](rendering/m3-materials.md): packed normals, basic occlusion,
  constant specular approximations and remaining layer programmes.
- [Model lights](rendering/lights.md): animated scene lights, conversion
  settings, and application customization.
- [Model cameras](rendering/cameras.md): authored views and opt-in playback.
- [Model events](rendering/events.md): messages, timing, and occurrence poses.
- [External-model particles](rendering/prem.md): child models,
  world-space motion, animation, and lifetime.
- [Sprite particles](rendering/pre2.md): emission, head/tail geometry,
  lifetime animation, M2 packed gravity/twinkle/secondary textures, and sorting.
- [Ribbon trails](rendering/ribbons.md): birth-time sections, connected
  geometry, atlas animation, gravity, and material passes.

Each guide describes the implemented behavior and relevant input, resource, or
integration limits. Update the relevant guide when an implementation changes,
keeping implemented behavior distinct from unverified game fidelity.
