# Renderer architecture

The renderer compiles `Model<Universal>` into owned definitions, inserts reusable
Bevy assets, then creates independent instance state. Native codecs and semantic
adapters belong to `wc3`; rendering does not lower the graph into a particular
native file version. The public facade lives in `src/lib.rs`, plugin registration in
`src/plugin.rs`, and cross-feature ordering in `src/schedule.rs`.

## Compilation, preparation, and ownership

`assets/model.rs` loads native models and required companions through an
application resource resolver. It resolves role-specific image variants and compiles
the semantic model before returning `ModelAsset`. Assets retain compiled
definitions and dependency handles rather than the source graph.

`compiler.rs` assembles geometry, material, animation, feature, world, and
effect interpreters. `geometry.rs` projects selected connectivity ranges and
attribute domains, deduplicating complete compiled vertex records. Complete mesh
outputs share assets when their attributes, indices, and topology agree.
`material.rs` defines `ExtendedMaterial<StandardMaterial, ModelMaterialExtension>`;
`material/compile.rs` interprets surfaces and `material/animation.rs` samples their
properties. `textures.rs` identifies role, color-space, and effective sampler requests.
Unsupported semantics produce source-associated diagnostics or fail where no valid
geometry interpretation exists.

`compiled.rs` owns the compiled definitions and shared asset preparation.
`composition.rs` validates rest-space joint bindings and combines skinned equipment
with the host rig. Rigid component definitions retain independent animation and
physics state. Shared preparation interns mesh and inverse-bind handles across the
whole composition. `assembly/` owns resource plans, typed selections, atlas
composition, image caching and source compilation without game vocabulary.
`wow/` translates native geosets, replacement categories and joint-matching
instructions into that pipeline. Applications can supply their own selection
schemas through `assembly::Resolver<Selection>`.
`CompiledModel::prepare` inserts unique meshes and inverse bind palettes
into asset stores and returns `PreparedModel`. Source bytes and the graph
may be released before preparation. Mesh, bind, animation, and definition buffers
remain shared. Constant materials share handles; animated materials remain private
per instance and mark assets modified only when sampled values change.

`instances.rs` caches prepared source assets, waits for required dependencies,
rejects cyclic ownership, and populates application roots. Replacements remove only
renderer-owned descendants. Reloaded semantic identities are new: playback and
overrides migrate through unique names, with observable reset counts.

## Pose, surfaces, and hooks

`runtime/` owns independent clocks and evaluation state. `pose.rs` composes a
parent-first affine hierarchy while retaining signed rotation and scale factors,
including collapsed axes. Ordinary hierarchy preserves shear; exception nodes
requiring inheritance or camera compensation use finite factor-based transforms.
Live poses, effect births, camera sampling, event occurrence poses, and `baking.rs`
share the evaluator. Optional authored secondary motion in `physics.rs` runs after
sampled local poses and before final ECS transforms are published. The default
path does not allocate physics buffers. Mounted models evaluate parent-first, and
inherited forces use the current carrier pose in the same frame.

`transition.rs` retains a frozen authored local source pose and blends it into the advancing destination. Completed fades retain that source for
occurrence-time rewinds and skip live interpolation.

Live instances retain a complete model-space pose cache for scene features and
effects. Ordinary skeleton entities carry local transforms through Bevy's hierarchy.
Occurrence-time sampling and offline baking evaluate their required authored
hierarchy independently.

`features.rs` interprets common lights, cameras, and attachment mounts. `camera.rs`
binds authored views to application-owned cameras before projection updates.
`events.rs` indexes typed event programmes and performs bounded chronological
interval traversal. `lod.rs` selects authored geometry from projected rest bounds;
`skin_bounds.rs` refreshes joint-contribution bounds only for changed input
transforms or matching mesh/bind asset events, deferring hidden work until reveal;
`world.rs` retains groups, placement sets, portals, liquid grids, and environmental
integration data. `world_lighting.rs` prepares static vertex illumination and
portal-distance transitions before geometry projection; draw-local regional lighting
participates in material sharing without taking ownership of Bevy scene lighting.

Generic feature hooks preserve source identities and kinds. Unsupported complete
payloads are not retained on spawned entities; applications needing them must
inspect the semantic graph during compilation. Attachment placement appearance and
supported light definitions have dedicated owned components.

## Effect record storage

`effects/` owns semantic emission, atlas sprites, age-driven sprites, model
particles, and trails. Its `sprites/` and `trails/` modules provide shared GPU
transport, with common record storage, extraction, upload, draw ordering, and
shader-generated geometry without native model-record dependencies.

Atlas sprites and trails retain immutable births in bounded ring storage. GPU
updates upload changed birth ranges; per-view sorting uses separate indices. Ribbon
material passes share section storage. Age-driven sprites currently reuse CPU mesh
buffers and upload only when population, view, or evaluated appearance changes.
Model particles instantiate independently animated child models with explicit owner
cleanup. Each interpreter exposes population and omitted-work counters and applies
bounded birth/long-update budgets.

## Scheduling contract

`Wc3Systems` remains the plugin's scheduling integration vocabulary. Ordering only
applies within the schedule containing that set. The main stages are:

| Schedule | Set | Contract |
| --- | --- | --- |
| `Update` | `SpawnInstances` | Populate ready requests and replace invalidated source definitions. |
| `Update` | `AdvanceAnimation` | Synchronize composed mount controls, advance clocks, and synchronize existing model-particle child playback. |
| `PostUpdate` | `AnimateCameras` | Sample bound views before Bevy projection updates and node poses. |
| `PostUpdate` | `EvaluateNodePoses` | Apply attachment restarts, authored poses, optional secondary motion, material animation, and scene features before propagation. |
| `PostUpdate` | `SelectLod` | Select geometry after camera updates and transform propagation, before visibility. |
| `PostUpdate` | `SimulateEffects` | Sample births and advance effects after propagation. |
| `PostUpdate` | `DispatchEvents` | Emit crossed events with occurrence poses after propagation. |

Move roots and camera parents before `AnimateCameras` when using authored camera
bindings, and before `EvaluateNodePoses` for ordinary pose inputs. Read events after
`DispatchEvents` or in the next `Update`. Zero simulation delta permits
asset/pipeline/readback work without advancing model or effect clocks.

Private tests live beside their implementation modules; public API tests and
fixtures live under `tests/`. The shared [capture tool](tools.md) exercises the same
plugin path as applications.
