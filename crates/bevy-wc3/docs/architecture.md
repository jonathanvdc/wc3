# Renderer architecture

The renderer turns a `Model<Universal>` into reusable Bevy assets and independent
instance state. Native decoding and semantic conversion belong to `wc3`; image
resolution, GPU preparation, and live rendering belong here. Read the
[application guide](usage.md) for public workflows and
[compilation guide](rendering/universal.md) for input contracts.

## From asset to instance

Loading, compilation, preparation, and spawning have separate ownership boundaries:

| Stage | Responsibility | Main modules |
| --- | --- | --- |
| Load | Resolve native models, required companions, and image variants into a `ModelAsset`. | `assets/model.rs`, `textures.rs` |
| Compile | Build owned geometry, animation, materials, effects, and scene definitions. | `compiler.rs`, `geometry.rs`, `material/compile.rs` |
| Prepare | Insert shared meshes and inverse binds into Bevy asset stores. | `compiled.rs` |
| Spawn | Populate application roots and maintain independent live state. | `instances.rs`, `runtime/` |

Source bytes and the semantic graph can be released after compilation. Complete
equal meshes and bind palettes share storage; constant materials share handles.
Animated materials remain private to an instance and only change their Bevy assets
when sampled values change. Channel indexes avoid repeatedly sampling properties
with no effective animation while retaining every authored node for overrides.

`instances.rs` waits for required dependencies and rejects cyclic ownership.
Replacement removes renderer-owned descendants while preserving application
children. Reloaded graph IDs are new; playback and pose selections migrate through
unique names, with results exposed through `ModelRebind`.

`composition.rs` validates shared-skeleton bindings and mounted components.
`assembly/` resolves resource plans, selections, and texture atlases; `wow/`
translates native character choices into those general operations. Rigid components
retain independent clocks and physics state while sharing prepared assets.

## Pose and scene evaluation

`Model::prepare_pose` in `wc3` prepares traversal and channel information.
`pose.rs` adapts the shared evaluator to Bevy transforms, occurrence sampling, and
`baking.rs`. Ordinary hierarchy preserves affine shear. Nodes requiring special
inheritance or camera compensation use finite rotation/scale factors, including
signed and collapsed scales. CPU baking shares skin-matrix and deformation
operations with `wc3`; live geometry deforms on the GPU.

`transition.rs` freezes the source local pose and blends toward the advancing
destination. It retains that source for within-update occurrence sampling.
`physics.rs` optionally applies authored secondary motion after sampled local poses
and before final transforms. Mounted rigs evaluate parent-first so inherited
forces use the current carrier pose.

Scene integration is divided by purpose. `features.rs` handles lights and mounts;
`camera.rs` binds authored views; `events.rs` traverses event windows; `lod.rs`
selects geometry. `skin_bounds.rs` updates conservative joint-contribution bounds
when their inputs change. `world.rs` handles world selections and environment data,
while `world_lighting.rs` prepares vertex illumination and regional contributions
without replacing Bevy scene lighting.

## Effects

`effects/` interprets native emitter families. Sprite and trail transport share
record storage, extraction, upload, and ordering machinery where their behavior
agrees; native definitions remain separate.

Atlas sprites and trails store immutable births in bounded rings. GPU updates
upload changed ranges, and per-view sorting uses separate indices. Effect draws
bind their own emitter records after Bevy's view bindings, leaving application
cameras and their GPU mesh preprocessing settings unchanged. Queueing, per-view
sorting, and uploads follow Bevy's visible mesh selections, including camera
render layers. Ribbon passes share section records. M2 age-driven sprites sample
exact authored curves and motion on the CPU into one compact record per particle.
The GPU expands shared heads and tails for each view; camera motion updates their
billboards and sorting without rebuilding particle vertices. Paused, unchanged
samples reuse GPU records. All effect transports cache bindings, identical
uniform data, and per-view draw orders. The shared `effects/view_orders.rs` cache
invalidates ordering when its source, uniforms, policy, or camera frame changes;
unchanged views avoid sorting and index uploads. Empty or hidden effects disable
draws while retaining resident capacity, and removed views release their order
buffers. Ring lineage distinguishes reset simulations from older snapshots with
the same write cursor.

Model particles create independent child models with explicit cleanup when their
owner disappears.
Population, birth, and long-update budgets bound work and expose omitted-work
counters. The focused [effect guides](README.md#rendering-reference) describe their
semantics and limits.

## Scheduling contract

`Wc3Systems` is the public scheduling interface. Ordering applies only within the
schedule containing the set; use the appropriate schedule when integrating an
application system.

| Schedule | Set | Contract |
| --- | --- | --- |
| `Update` | `SpawnInstances` | Populate ready requests and replace invalidated definitions. |
| `Update` | `AdvanceAnimation` | Synchronize mount controls, advance clocks, and update existing model-particle playback. |
| `PostUpdate` | `AnimateCameras` | Sample bound views before Bevy projection updates and node poses. |
| `PostUpdate` | `EvaluateNodePoses` | Evaluate attachment restarts, poses, secondary motion, materials, and scene features before propagation. |
| `PostUpdate` | `SelectLod` | Select geometry after camera updates and transform propagation, before visibility. |
| `PostUpdate` | `SimulateEffects` | Sample births and advance effects after propagation. |
| `PostUpdate` | `DispatchEvents` | Emit crossed events with their occurrence poses after propagation. |

Move roots and camera parents before `AnimateCameras` when using authored camera
bindings, and before `EvaluateNodePoses` for ordinary pose inputs. Read events after
`DispatchEvents` or in the next `Update`. Zero simulation delta allows loading,
pipeline warmup, and readback without advancing animation or effects.

The public facade is `src/lib.rs`, plugin registration is in `src/plugin.rs`, and
ordering is in `src/schedule.rs`. Private tests live beside their modules; public
API tests and fixtures live under `tests/`. The [capture tool](tools.md) exercises
the same plugin path as applications.
