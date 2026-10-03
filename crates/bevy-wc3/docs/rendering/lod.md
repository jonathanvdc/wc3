# Geometry level of detail

The renderer switches between geometry levels already authored in a model.
Each instance selects one level while keeping its animation clock, rig, and
material state. This reduces the geometry submitted for rendering when a model
occupies fewer pixels. It does not generate simplified geometry for models that
have only one level, throttle animation, or reduce particle and ribbon budgets.

## Prepared geometry and visibility

Preparation builds every drawable authored level, including each material layer's
UV mesh variants, and preserves original geoset IDs for geoset animation.
`PreparedModel::lod_levels()` exposes the sorted levels with drawable geometry;
larger level numbers mean coarser geometry. Empty geometry, invalid material
references, and materials without layers do not contribute levels. Unsupported
geometry in any prepared level causes preparation to fail, even if that level
would initially be hidden.

Geosets with the unsigned sentinel `u32::MAX` (signed `-1`) are common geometry
and remain visible at every selected level. A classic geoset without LOD metadata
also belongs to common geometry before version conversion; the runtime conversion
normally supplies level zero. Models with only common geometry expose level zero
for selection. This is the renderer's explicit policy; exact game behavior for
mixed sentinel and numbered levels remains unverified.

Each level has a visibility parent beneath the instance root. Its mesh passes
share that instance's joints and continue sampling their own material and geoset
animation. LOD hides the parent, while zero animated alpha hides individual
passes. Root and attachment visibility still gate the whole hierarchy, and
root cleanup removes every level. Switching levels never restarts animation or
recreates assets. Meshes and inverse bind poses remain shared across instances;
material handles and runtime selection belong to each instance.

## Choose a policy and quality

The plugin initializes `Wc3LodSettings` with `default_lod: Wc3Lod::Fixed(0)`,
preserving the previous default view. Add `Wc3Lod::Automatic` to a root to opt
that instance in, or insert an automatic preset resource to enable it globally.
`low()`, `medium()`, and `high()` use quality biases of 0.5, 1, and 2 respectively.
They all enable automatic selection; larger bias retains detailed geometry longer.

Applications can customize the global resource or put a complete
`Wc3LodOverride(Wc3LodSettings)` on an individual root. An explicit `Wc3Lod`
component takes precedence over the effective settings' default policy.
Overrides apply only to their root; attached models and model particles use the
global configuration unless given their own override.

```rust
use bevy::prelude::*;
use bevy_wc3::{Wc3Lod, Wc3LodOverride, Wc3LodSettings};

// Configure after adding Wc3BevyPlugin, or insert before it is added.
app.insert_resource(Wc3LodSettings::medium());

// Retain more detail on this instance, or force a particular authored level.
commands.entity(root).insert(Wc3LodOverride(Wc3LodSettings::high()));
commands.entity(other_root).insert(Wc3Lod::Fixed(2));
```

`thresholds` contains descending pixel diameters for transitions through the
model's sorted available levels, rather than indexing raw level numbers. For
levels `[0, 2, 5]`, the first threshold controls 0 to 2, and the second controls
2 to 5. Defaults are 240, 120, and 60 physical pixels. When a model has more
transitions, further thresholds halve the last value. An empty list keeps the
finest permitted level. `quality_bias` multiplies measured pixel diameter before
these comparisons.

`minimum_level` caps detail by prohibiting finer authored levels. A fixed request
or cap absent from the model resolves to the next available coarser level; requests
beyond all available levels select the coarsest. For example, requesting level 1
from `[0, 2, 5]` selects 2. Common geometry is unaffected by the cap.

`hysteresis` defines a fractional dead band around each threshold. With the
default 0.15, a 240-pixel boundary switches to coarser geometry below 204 pixels
and returns to finer geometry at 276 pixels. Initial automatic selection uses
unmodified boundaries; subsequent automatic frames use the dead band. Fixed
selection and detail caps take effect immediately.

Call `Wc3LodSettings::validate()` when accepting application or user input.
Bias must be finite and positive, thresholds finite, positive, and strictly
decreasing, and hysteresis finite and in `[0, 1)`. Invalid effective settings use
the built-in defaults. An explicit root policy still applies in that fallback.

## Camera measurement and scheduling

Automatic selection measures a conservative sphere built from all geoset
vertices and authored model, sequence, and geoset bounds. The sphere is fixed
across selected levels and sequences so selection cannot oscillate because its
own geometry changed. World transforms account for scale and shear. Authored
bounds must encompass animation; the system does not recompute bounds from the
current skinned pose or include emitted particles and attached models.

Perspective measurement responds to camera distance and field of view;
orthographic measurement responds to projection zoom. Both use the viewport's
physical pixel height. Spheres intersecting a perspective camera's near plane
and custom projections request the finest permitted level. Missing usable
camera measurements also fall back to that level.

Camera precedence shares the node-pose rules: the nearest ancestor
`Wc3NodeCamera` wins, followed by a sole active `Wc3DefaultNodeCamera`, a sole
active window camera, or a sole active 3D camera. If the default selection is
ambiguous, LOD uses the largest projected diameter among active 3D views whose
render layers intersect the model's mesh passes. An explicit but inactive or
unavailable camera falls back to finest detail. Every view, including shadows,
renders the same selected geometry; independent per-view selection is absent.

`Wc3Systems::SelectLod` runs in `PostUpdate` after camera updates and transform
propagation, before visibility propagation and view checks. Change root policies
or quality settings in `Update`, or order `PostUpdate` changes before this set.
Applications that move cameras or roots must also respect the transform and
node-pose scheduling contract in [architecture](../architecture.md#scheduling-contract).

## Capture levels

The existing offscreen capture tool accepts fixed and automatic selection plus
quality overrides. The fixture has a red level-zero quad, a green level-two
triangle with animated alpha, and a blue common quad sharing a moving bone.

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/lod_capture.mdl /tmp/wc3-lod \
  --lod 2 --times 0,0.5,1 --fps 60 --size 640x480 \
  --eye 0,-6,8 --target 0,0,0
```

Use `--lod auto`, `--lod-bias`, `--lod-thresholds`, `--lod-hysteresis`, and
`--lod-minimum` to compare automatic policies with the same times and camera.
The tool prints the selected geometry level for each frame. Selection thresholds
are configurable renderer policy, not a reproduction of Warcraft III's quality
settings.
