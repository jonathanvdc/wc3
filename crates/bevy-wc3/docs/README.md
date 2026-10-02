# bevy-wc3 documentation

Start with the [crate README](../README.md) for a minimal Bevy application.

## Application and architecture

- [Using bevy-wc3](usage.md): load instances, resolve texture extension fallbacks, choose textures, control playback,
  mount child models, configure lights, and use the examples.
- [Architecture](architecture.md): loading and preparation, shared and per-instance
  state, renderer modules, and the `Wc3Systems` scheduling contract.
- API reference: `cargo doc -p bevy-wc3 --no-deps --open`.

## Rendering

The files under `rendering/` describe how the renderer evaluates model data
and draws it through Bevy. Start with the [renderer overview](rendering/renderer.md),
then choose a topic:

- [Animation blending](rendering/animation-blending.md): sequence controls,
  local-pose transitions, interruption, and effect/event sampling.
- [Node transforms and flags](rendering/node-flags.md): inheritance,
  billboarding, camera anchoring, and driving-camera selection.
- [Geoset animation](rendering/geoset-animation.md): color, opacity,
  material isolation, and sampling.
- [Reforged materials](rendering/reforged-materials.md): texture roles,
  surface animation, UV sets, and Bevy shading integration.
- [Model lights](rendering/lights.md): animated scene lights, conversion
  settings, and application customization.
- [Model cameras](rendering/cameras.md): authored views and opt-in playback.
- [Model events](rendering/events.md): messages, timing, and occurrence poses.
- [Classic model particles (PREM)](rendering/prem.md): child models,
  world-space motion, animation, and lifetime.
- [Quad particles (PRE2)](rendering/pre2.md): emission, head/tail geometry,
  lifetime animation, GPU records, and sorting.
- [Ribbon trails](rendering/ribbons.md): birth-time sections, connected
  geometry, atlas animation, gravity, and material passes.

Each guide describes the implemented behavior and relevant input, resource,
or integration limits. [Verification notes](verification.md) retain earlier test
and capture reports, including the scope of those checks. Update the behavior
guide when an implementation changes and record new verification separately.
