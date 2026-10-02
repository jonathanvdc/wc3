# bevy-wc3 documentation

Start with the [crate README](../README.md) for a minimal Bevy application.

## Application and architecture

- [Using bevy-wc3](usage.md): load instances, choose textures, control playback,
  mount child models, configure lights, and use the examples.
- [Architecture](architecture.md): loading and preparation, shared and per-instance
  state, renderer modules, and the `Wc3Systems` scheduling contract.
- API reference: `cargo doc -p bevy-wc3 --no-deps --open`.

## Renderer implementation

The files under `visual-fidelity/` describe how the renderer evaluates model data
and draws it through Bevy. Start with the [renderer overview](visual-fidelity/renderer.md),
then choose a topic:

- [Animation blending](visual-fidelity/animation-blending.md): sequence controls,
  local-pose transitions, interruption, and effect/event sampling.
- [Node transforms and flags](visual-fidelity/node-flags.md): inheritance,
  billboarding, camera anchoring, and driving-camera selection.
- [Geoset animation](visual-fidelity/geoset-animation.md): color, opacity,
  material isolation, and sampling.
- [Reforged materials](visual-fidelity/reforged-materials.md): texture roles,
  surface animation, UV sets, and Bevy shading integration.
- [Model lights](visual-fidelity/lights.md): animated scene lights, conversion
  settings, and application customization.
- [Model cameras](visual-fidelity/cameras.md): authored views and opt-in playback.
- [Model events](visual-fidelity/events.md): messages, timing, and occurrence poses.
- [Classic model particles (PREM)](visual-fidelity/prem.md): child models,
  world-space motion, animation, and lifetime.
- [Quad particles (PRE2)](visual-fidelity/pre2.md): emission, head/tail geometry,
  lifetime animation, GPU records, and sorting.
- [Ribbon trails](visual-fidelity/ribbons.md): birth-time sections, connected
  geometry, atlas animation, gravity, and material passes.

Topic notes separate implementation limits from behavior whose game fidelity is
unverified. Capture reports record earlier checks and their scope, rather than
claiming new verification. When behavior changes, update its topic and this index.
