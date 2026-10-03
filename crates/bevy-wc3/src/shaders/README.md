# WC3 shader sources

These shaders are adapted from Bevy **v0.19.1** (MIT OR Apache-2.0):

| Local file | Upstream source |
| --- | --- |
| `wc3_material.wgsl` | [`bevy_pbr/src/render/pbr.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/pbr.wgsl) |
| `wc3_material_prepass.wgsl` | [`bevy_pbr/src/render/pbr_prepass.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/pbr_prepass.wgsl) |
| `wc3_mesh.wgsl` | [`bevy_pbr/src/render/mesh.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/render/mesh.wgsl) |
| `wc3_prepass.wgsl` | [`bevy_pbr/src/prepass/prepass.wgsl`](https://github.com/bevyengine/bevy/blob/v0.19.1/crates/bevy_pbr/src/prepass/prepass.wgsl) |

## Local changes

The shaders extend Bevy's skinning inputs and material evaluation while retaining
its render paths. Blocks marked `WC3 BEGIN` and `WC3 END` identify adaptations
that need to be carried forward on an upgrade.

The vertex input accepts a second joint index and weight set at shader locations
8 and 9. Additional functions sum those four extra matrices when
`WC3_EXTRA_INFLUENCES` is set; otherwise entry points use Bevy's original
four-influence functions. The previous-frame function keeps motion vectors in
sync with visible animation. `materials/mod.rs` maps the extra attributes into
both Bevy vertex layouts, and `preparation/mesh.rs` supplies eight influence rows
and Bevy's per-joint bounds, including the second set.

The material fragment shader adds HD team masking, RG normal reconstruction,
and explicit Fresnel/environment approximations around Bevy PBR. Its prepass
fragment uses the same normal reconstruction while retaining upstream alpha
discard and motion vectors. Bindings 100–106 are local to the WC3 material
extension. Neither shader replaces Bevy global shaders. See
[Reforged materials](../../docs/rendering/reforged-materials.md) for texture roles,
animated controls, and render-pass behavior.

## Bevy upgrades

Diff each local file against the new upstream version and reapply the local
changes above. Include the prepass paths when reviewing changes to depth,
shadow, normal, deferred, and motion-vector rendering so visible geometry and
auxiliary passes stay consistent. Update the source-version table when the
adaptations move to a new Bevy release.
