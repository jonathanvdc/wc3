# bevy-wc3 guides

Start with the [crate README](../README.md) to view a model or build a minimal
Bevy application. Then use the guides below for your next task.

## Use the renderer

The application and tool guides cover the usual workflow, from resource lookup to
animation and visual checks. Assembly is useful when one visible object consists
of several models.

- [Application integration](usage.md): loading, textures, playback, replacement, sockets, and pose baking.
- [Viewer and captures](tools.md): inspect local models and produce repeatable PNGs.
- [Model assembly](rendering/assembly.md): appearance choices, prepared dependencies, texture atlases, and mounted or skinned parts.
- [WoW characters and equipment](rendering/characters.md): translate native appearance choices into an assembly.
- [Performance measurements](performance.md): completed-frame timing and model or character crowds.

## Rendering reference

The [renderer overview](rendering/renderer.md) explains broad coverage and its
limits. The focused guides describe implemented behavior; support for a feature
does not by itself establish exact game fidelity.

| Topic | Guides |
| --- | --- |
| Motion and geometry | [Animation and transitions](rendering/animation-blending.md), [node transforms](rendering/node-flags.md), [secondary motion](rendering/physics.md), [LOD](rendering/lod.md), [mesh color and opacity](rendering/geoset-animation.md) |
| Materials | [Reforged](rendering/reforged-materials.md), [M2](rendering/m2-materials.md), [M2 shader inventory](rendering/m2-programmes.md), [WMO](rendering/wmo-materials.md) |
| Scene integration | [Lights and activation](rendering/lights.md), [camera poses and projection](rendering/cameras.md), [events](rendering/events.md), [static worlds](rendering/worlds.md) |
| Effects | [Model particles](rendering/prem.md), [sprite particles, sampled bursts and adopted templates](rendering/pre2.md), [ribbons](rendering/ribbons.md), [external programmes](rendering/popcorn.md) |

## Extend or contribute

The compiler accepts `Model<Universal>` regardless of its source format. These
guides explain where native interpretation ends and renderer ownership begins.

- [Compilation and native resources](rendering/universal.md): geometry, image requests, import policy, and diagnostics.
- [Architecture and scheduling](architecture.md): shared assets, instance state, module boundaries, and system order.
- [CPU model evaluation](../../wc3/docs/evaluation.md): pose, deformation, and surface operations without Bevy.
- [Shader sources](../src/shaders/README.md): upstream attribution and the Bevy upgrade checklist.

Build the API reference with `cargo doc -p bevy-wc3 --no-deps --open`.
