# Geometry level of detail

The renderer switches between geometry levels already authored in a model. Each
instance selects one level while keeping its animation clock, rig, and material
state. This reduces the geometry submitted for rendering when a model occupies fewer
pixels. It does not generate simplified geometry for models that have only one
level, throttle animation, or reduce particle and ribbon budgets.

## Prepared geometry and visibility

Preparation builds every drawable authored level, including material coordinate
variants, while retaining typed mesh and draw identities. Typed draw metadata
retains authored levels; larger level numbers mean coarser geometry. Empty geometry
does not contribute levels; invalid references fail validation. Unsupported geometry
in any prepared level causes preparation to fail, even if that level would initially
be hidden.

Semantic draws without a level are common geometry and remain visible across
selected levels. Native adapters preserve the corresponding sentinel convention.
Models containing only common geometry expose level zero for selection. Exact native
behavior for mixed common/numbered geometry remains unverified.

Each pass retains its authored level and shares the instance's joints. LOD
visibility intersects material alpha and world-group visibility. Root and attachment
visibility still gate the whole hierarchy, and root cleanup removes every level.
Switching levels never restarts animation or recreates assets. Meshes and inverse
bind poses remain shared across instances; material handles and runtime selection
belong to each instance.

## Choose a policy and quality

The plugin initializes `UniversalLodSettings` with `default_lod:
UniversalLod::Fixed(0)`, so instances initially use the finest authored level
available at or above zero. Add `UniversalLod::Automatic` to a root to opt that
instance in, or insert an automatic preset resource to enable it globally. `low()`,
`medium()`, and `high()` use quality biases of 0.5, 1, and 2 respectively. They all
enable automatic selection; larger bias retains detailed geometry longer.

Applications can customize the global resource or put a complete
`UniversalLodOverride(UniversalLodSettings)` on an individual root. An explicit
`UniversalLod` component takes precedence over the effective settings' default
policy. Overrides apply only to their root; attached models and model particles use
the global configuration unless given their own override.

```rust
use bevy::prelude::*;
use bevy_wc3::{UniversalLod, UniversalLodOverride, UniversalLodSettings};

// Configure after adding Wc3BevyPlugin, or insert before it is added.
app.insert_resource(UniversalLodSettings::medium());

// Retain more detail on this instance, or force a particular authored level.
commands.entity(root).insert(UniversalLodOverride(UniversalLodSettings::high()));
commands.entity(other_root).insert(UniversalLod::Fixed(2));
```

`thresholds` contains descending pixel diameters for transitions through the model's
sorted available levels, rather than indexing raw level numbers. For levels `[0, 2,
5]`, the first threshold controls 0 to 2, and the second controls 2 to 5. Defaults
are 240, 120, and 60 physical pixels. When a model has more transitions, further
thresholds halve the last value. An empty list keeps the finest permitted level.
`quality_bias` multiplies measured pixel diameter before these comparisons.

`minimum_level` caps detail by prohibiting finer authored levels. A fixed request or
cap absent from the model resolves to the next available coarser level; requests
beyond all available levels select the coarsest. For example, requesting level 1
from `[0, 2, 5]` selects 2. Common geometry is unaffected by the cap.

`hysteresis` defines a fractional dead band around each threshold. With the default
0.15, a 240-pixel boundary switches to coarser geometry below 204 pixels and returns
to finer geometry at 276 pixels. Initial automatic selection uses unmodified
boundaries; subsequent automatic frames use the dead band. Fixed selection and
detail caps take effect immediately.

Call `UniversalLodSettings::validate()` when accepting application or user input.
Bias must be finite and positive, thresholds finite, positive, and strictly
decreasing, and hysteresis finite and in `[0, 1)`. Invalid effective settings use
the built-in defaults. An explicit root policy still applies in that fallback.

## Camera measurement and scheduling

Automatic selection measures a conservative sphere built from prepared geometry and
supported authored bounds. The sphere is fixed across selected levels and sequences
so selection cannot oscillate because its own geometry changed. World transforms
account for scale and shear. Authored bounds must encompass animation; the system
does not recompute bounds from the current skinned pose or include emitted particles
and attached models.

Perspective measurement responds to camera distance and field of view; orthographic
measurement responds to projection zoom. Both use the viewport's physical pixel
height. Spheres intersecting a perspective camera's near plane and custom
projections request the finest permitted level. Missing usable camera measurements
also fall back to that level.

Automatic selection considers active 3D cameras with intersecting render layers.
When `UniversalPlayback::driving_camera` is set, only that camera contributes;
otherwise the largest projected diameter across matching views is used. Missing
usable measurements select the finest permitted level. Every view, including
shadows, renders the same selected geometry; independent per-view selection is
absent.

`Wc3Systems::SelectLod` runs in `PostUpdate` after camera updates and transform
propagation, before visibility propagation and view checks. Change root policies or
quality settings in `Update`, or order `PostUpdate` changes before this set.
Applications that move cameras or roots must also respect the transform and
node-pose scheduling contract in
[architecture](../architecture.md#scheduling-contract).

## Inspection

`UniversalLodState::selected_level()` exposes the current selection. The shared
[viewer/capture tool](../tools.md) accepts fixed and automatic policy controls.
Thresholds are configurable renderer policy rather than native game quality presets.
