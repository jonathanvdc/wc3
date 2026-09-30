# WC3 skinning shader sources

These shaders are adapted from Bevy **v0.19.1** (MIT OR Apache-2.0):

| Local file | Upstream source |
| --- | --- |
| `wc3_mesh.wgsl` | [`bevy_pbr/src/render/mesh.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/mesh.wgsl) |
| `wc3_prepass.wgsl` | [`bevy_pbr/src/prepass/prepass.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/prepass/prepass.wgsl) |

The blocks marked `WC3 BEGIN` and `WC3 END` replace Bevy's vertex input with one that accepts a second joint index and weight set at shader locations 8 and 9, and add functions that sum those four extra matrices. The shader entry points call these functions when `WC3_EXTRA_INFLUENCES` is set; otherwise they call Bevy's original four-influence functions. The previous-frame function keeps motion vectors in sync with the visible animation. All other rendering logic follows the pinned Bevy sources.

On a Bevy upgrade, diff each local file against the new upstream version and reapply these changes. Update the prepass shader for changes to depth, shadow, normal, deferred, and motion vector paths. `material.rs` maps the extra attributes into both Bevy vertex layouts; `mesh.rs` supplies eight influence rows and Bevy's per-joint bounds, including the second set.
