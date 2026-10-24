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
| Compile | Build owned geometry, animation, materials, effects, and scene definitions. | `model_compilation/`, `geometry.rs`, `material/compile.rs` |
| Prepare | Insert shared meshes and inverse binds into Bevy asset stores. | `compiled.rs`, `compiled/prepared_geometry.rs` |
| Spawn | Populate application roots and maintain independent live state. | `instances.rs`, `runtime/` |

Source bytes and the semantic graph can be released after compilation. Complete
equal meshes and bind palettes share storage; constant materials share handles.
Asset-backed instances also share prepared mesh and inverse-bind assets across
different `ModelAsset` identities in one World. The geometry lookup keeps only
candidate asset IDs, checks exact content in the current asset stores, and
acquires a strong handle from those stores on a hit. Its metadata owns no buffers
or strong handles; prepared models retain their ordinary asset ownership.
Per-model draws, identities, materials, animation and live state remain
independent. Standalone `CompiledModel::prepare` retains its
local preparation behavior; manual clients can reuse a `PreparedGeometryCache`
with `CompiledModel::prepare_with_cache` across calls.

Animated materials remain private to an instance and only change their Bevy assets
when sampled values change. On capable devices, `ModelMaterial` uses Bevy's
bindless material allocator for combined extension data and image bindings.
The ordinary `StandardMaterial` extension, public asset values and render phases
remain the same. Devices without that support use bounded caches of immutable
extension uniform buffers; cache hits reuse them across base-opacity changes.
An event-maintained image dependency index refreshes affected material allocations
when image contents change. Image preparation precedes material preparation, so
both binding paths see the current GPU view. Channel indexes avoid repeatedly
sampling properties with no effective animation while retaining every authored
node for overrides.

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
sorting, and uploads follow Bevy's per-view visible-entity selections, including camera
render layers. Ribbon passes share section records. M2 age-driven sprites sample
exact authored curves and motion on the CPU into one compact record per particle.
The GPU expands shared heads and tails for each view; camera motion updates their
billboards and sorting without rebuilding particle vertices. Paused, unchanged
samples reuse GPU records. All effect transports cache bindings, identical
uniform data, and per-view draw orders. The shared `effects/view_orders.rs` cache
invalidates ordering when its record source, center or index-expansion inputs,
policy, or camera frame changes. Appearance-only time changes need not invalidate
an unchanged center dependency; unchanged views avoid sorting and index uploads. Empty or hidden effects disable
draws while retaining resident capacity, and removed views release their order
buffers. Ring lineage distinguishes reset simulations from older snapshots with
the same write cursor.

External programmes can retain immutable birth slots for closed billboard
expressions. `effects/programmes/immutable.rs` prepares a shared core projection
of the renderer's changing fields. That exact compact recipe drives WGSL,
capture layout and finite-domain admission; it does not replace full source
execution authorization. All supported billboard geometries share the ordinary
fragment and per-view geometry path. Moving centers use a separate position-only
CPU projection and reusable scratch for sort keys, while changing axes and other
appearance remain shader expressions. Stable center snapshots can reuse view
orders independently of appearance time. Generated shader assets are interned by
equal WGSL source, with strong ownership held by live appearances.

M2 template adoption shares immutable birth/motion kernels, age curves and child
image bindings with ordinary M2 sprites. `effects/m2_particles/templates.rs`
prepares a first-four-slot catalogue from Universal declarations; `sources.rs`
retains bounded motion snapshots across parent endpoint retirement, and
`adopted_pool.rs` owns one phase and child pool per source/template pair.
`evaluation.rs` samples ordinary occurrence frames and admission, while
`update.rs` owns the shared parent intervals and publication. Adoption never
instantiates or evaluates the template model's skeleton or other scene features.

Model particles create ordinary independently animated child models. Their
linked lifetime ownership is independent of world-space Transform parenting.
`runtime/model_driver.rs` evaluates only roots with model-particle emitters and
their automatic actor subtrees before propagation. It orders detached ownership
and mounted dependencies explicitly. `NodePoseSource` identifies each node's
owning rig and pose index, so sparse current-frame resolution uses the complete
posed matrix once instead of recomposing informational node Transforms. The
ordinary and targeted paths share the same factory and pose/material evaluator;
owned actors are excluded from the global pose pass and do not integrate physics
twice. Ready newborns populate within that early step, while pending requests
retain the same checked lifetime.
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
| `Update` | `AdvanceAnimation` | Synchronize mount controls and advance clocks. |
| `PostUpdate` | `AnimateCameras` | Sample bound views before Bevy projection updates and node poses. |
| `PostUpdate` | `EvaluateNodePoses` | Evaluate attachment restarts, poses, secondary motion, materials, and scene features before propagation. |
| `PostUpdate` | `SimulateModelParticles` | Project model-particle sources, populate ready births, and evaluate owned actor subtrees before propagation. |
| `PostUpdate` | `SelectLod` | Select geometry after camera updates and transform propagation, before visibility. |
| `PostUpdate` | `SimulateEffects` | Advance sprite-only roots and publish remaining effects from already observed model-particle clocks. |
| `PostUpdate` | `DispatchEvents` | Emit crossed events with their occurrence poses after propagation. |

Move roots and camera parents before `AnimateCameras` when using authored camera
bindings, and before `EvaluateNodePoses` for ordinary pose inputs. Read events after
`DispatchEvents` or in the next `Update`. Zero simulation delta allows loading,
pipeline warmup, and readback without advancing animation or effects.

The public facade is `src/lib.rs`, plugin registration is in `src/plugin.rs`, and
ordering is in `src/schedule.rs`. Private tests live beside their modules; public
API tests and fixtures live under `tests/`. The [capture tool](tools.md) exercises
the same plugin path as applications.
