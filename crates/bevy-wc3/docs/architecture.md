# bevy-wc3 architecture

bevy-wc3 separates model loading and shared asset preparation from per-instance
animation and rendering. This guide describes the module responsibilities,
asset ownership, and scheduling contracts used when extending the renderer or
integrating it with an application.

The crate keeps an explicit public facade in `src/lib.rs`. Implementation modules
are private; the facade exports model assets, instance components, preparation
and spawning APIs, material types, and scheduling integration points.
`src/plugin.rs` registers assets, embedded shaders, render plugins, and runtime
systems. `src/schedule.rs` owns the cross-feature execution order.

## Loading, preparation, and instantiation

A model moves through three stages: decoding its source and dependencies,
preparing reusable render assets, and creating an entity hierarchy for each
instance. The corresponding modules keep file handling separate from asset
construction and runtime ownership:

- `assets/` decodes MDX/MDL with strict version conversion, resolves images and
  child models, and compiles the model before returning `Wc3ModelAsset`. The asset
  retains owned CPU meshes, runtime definitions, and dependency handles rather
  than a decoded source container. Format definitions and codecs remain in `wc3`.
- `preparation/` separates CPU compilation from Bevy asset insertion.
  `compile_model` consumes a normalized `Model` and produces `CompiledModel`
  without an ECS world or asset stores. `CompiledModel::prepare` inserts meshes
  and inverse bind poses into Bevy and returns `PreparedModel`. Both forms share
  owned runtime definitions and preserve texture provenance. Offline pose baking
  evaluates the prepared data using explicit sequence and global clocks.
- `instance/` waits for compiled assets, caches their Bevy preparation, checks
  recursive child references, and manages ownership. Source asset events
  invalidate cached entries for future instances. Existing instances retain
  their runtime definitions and render assets. Compilation failures are reported
  by the loader, before an asset reaches the preparation cache.

`DecodedModel::decode` borrows compatible MDX arrays from the encoded input.
Strict version conversion moves unchanged chunks and allocates only where the
conversion requires a new representation. Compilation reads borrowed geometry
while constructing meshes, then consumes the decoded records. Retained animation
and feature records move already-owned buffers and copy borrowed buffers into
owned storage. MDL decoding already owns its buffers, so consuming compilation
moves those buffers directly. Geometry records and unrelated chunks are discarded
without converting their source arrays to owned data.

`CompiledModel` is the ownership boundary: it can outlive the input bytes and the
decoded container. Nodes belonging to scene features project their tracks from
the same owned records used by attachments, lights, events, or emitter simulation.
Preparation shares these definition buffers rather than copying them again.
Compiled meshes are shared between passes with identical UV variants; insertion
copies each unique CPU mesh into Bevy once per prepared model. The compiled asset
keeps its CPU meshes until it is unloaded. Spawning and offline baking use only
`PreparedModel`; no decoded source is retained at runtime.

Tools that need full source records can explicitly use `Wc3Model::decode` or
`decode_mdl`. Those owned inspection APIs remain separate from the renderer's
borrowed loading path. `prepare_model` and `prepare_model_with_resources` borrow
an inspected source, while `compile_model` consumes a model when retaining it is
unnecessary.

The prepared container is independent of the source container's layout. Its
current animation, material, and effect definitions still express WC3 semantics
and reuse types from `wc3::model`; another format needs a preparation adapter and
appropriate runtime behavior for features with different semantics.

After asset-backed spawning, the root's `Wc3ModelInstance` loading request is
removed and replaced by `ModelSource`, an asset ID that preserves provenance and
recursive child-model detection without retaining the compiled asset. Prepared
meshes, images, bind poses, and child-model dependencies retain their own handles.
Applications that want to reuse cached preparation should keep a source asset
handle themselves; unloading a source removes its cache entry while existing
instances continue using their prepared definitions and render assets.

Meshes and inverse bind poses are shared between instances. Texture bindings, animation clocks, rigs, and emitter simulation states belong to
individual instances. Preparation stores material values as templates. Constant
layer and surface states share material handles across instances of a prepared
model; independently animated states retain private handles. Changing bindings
on a constant pass detaches its material before sampling, preserving other
instances. The sharing cache retains only states used by live passes. Material assets are needed only when spawning
instances, so preparation takes only mesh and inverse bind pose asset stores.

`Wc3ModelOwner` retains lifetime ownership independently of `ChildOf` transform
and visibility inheritance. Attachments use both relationships. Model particles
can remain in world space while still being cleaned up with their owner.

## Animation and rendering features

`animation/playback.rs` owns the sequence clock, playback controls, and per-instance
pose transition state. `play` uses model BlendTime; immediate and duration-override
controls use the same playback path. The node system caches authored local poses
before inheritance/camera corrections. Transitions retain a frozen source and
blend into the advancing destination, including effect births and event poses.
`Wc3Animation::time()` exposes a borrowed `wc3::model::animation::AnimationTime`.
`wc3` resolves sequence/global-sequence clocks and samples tracks and properties;
`animation/sampling.rs` adapts those operations for Bevy effect callers.
`animation/pose/` owns animated node data, camera selection, pose evaluation,
and birth-time transform sampling. Live evaluation reuses sampling and hierarchy
buffers across frames and follows the prepared node order. Reparented nodes and
attachments use the hierarchy resolver when their parents are not yet evaluated.
The authored snapshot reuses its storage while exclusively owned; a transition
retains an immutable shared source, so subsequent live writes copy that snapshot
once. The ECS node system, effect birth sampling, and offline pose baking reuse
the same hierarchy composition rules.

`event.rs` dispatches crossed event keys as `Wc3ModelEvent` messages. Prepared
instances share event definitions and retain independent cursors. Interval
traversal lives under `wc3::model::animation`; explicit playback restart/seek
metadata lives with `Wc3Animation`. Event names are interpreted by applications.

`materials/` owns Bevy material specialization, layer/geoset animation, surface
and UV animation, texture binding precedence, and private linear image variants.
Shared layer and surface evaluators serve both
live animation systems and the offline baker; `animation/clocks.rs` supplies
independent clocks for offline sampling while live playback retains its shared clock.
`lod.rs` selects authored geometry per instance from fixed policies or projected
size, using shared driving-camera precedence and independent visibility groups.
`attachment.rs` and `light.rs` own their spawning and animation behavior.
`camera.rs` exposes authored views on instance roots and plays explicit bindings
on application-owned Bevy cameras without automatically creating active views.

`effects/` contains Classic model particles, PRE2 quad particles, and ribbons.
Shared simulation helpers provide clocks, emission arithmetic, and live record
management; emitter-specific modules retain capacity, expiry, and connectivity
policy. The PRE2 texture update lives beside PRE2 simulation and rendering rather
than inside the general binding API.

PRE2 and ribbons keep birth records, shader parameters, and immutable extraction
snapshots in their emitter-specific `records.rs` modules. Simulation produces
these snapshots without depending on pipeline or GPU allocation code. Their render
plugins extract snapshots, prepare resident GPU buffers, queue per-view draws,
and render shader-generated geometry. Shared `effects/records.rs` and
`effects/render.rs` implement record storage and GPU upload/allocation mechanics.
The shared renderer helpers also define texture layouts, record-index vertex
attributes, direct-draw camera setup, and transient transparent-phase entries.
Each renderer retains its own shader geometry, pipeline specialization, sorting,
and pass semantics. Classic particles instead spawn independently animated model
instances. The [renderer overview](rendering/renderer.md) connects these paths to
materials, animation, and the supported rendering features.

## Effect record storage

PRE2 and ribbons use shared ring storage to keep immutable birth records resident
on the GPU while the shader evaluates their changing appearance. Records retire
chronologically without a free-slot list. Capacity starts at 16 and doubles with
the live population, up to each emitter's 8,192-record limit. Growth rebases live
records. Immutable extraction snapshots share blocks of up to 64 records, so new
births copy only the blocks they touch.

GPU preparation tracks the last uploaded birth cursor. Normal updates write at
most two contiguous ranges across ring wraparound; retirement alone requires no
record upload. Growth or a full ring lap between rendered frames uploads the
current buffer in full. Per-view sorting uses a separate draw-order index buffer,
leaving the birth records unchanged. Ribbon material layers share the section
buffer while retaining their own uniforms and texture bindings.

## Scheduling contract

Applications can use the root-exported `Wc3Systems` sets to order their systems.
Set ordering applies only within the schedule where the set is configured.

| Schedule | Set | Contract |
| --- | --- | --- |
| `Update` | `SpawnInstances` | Creates ready instances before the other WC3 update sets. |
| `Update` | `BindTextures` | Updates PRE2 bindings after spawning; independent of the animation chain. |
| `Update` | `AdvanceAnimation` | Advances root clocks after spawning and before instance animation. |
| `Update` | `AnimateInstances` | Updates model-particle animation, attachments, lights, layers, and surfaces, in that order. |
| `PostUpdate` | `AnimateCameras` | Samples bound model cameras using current root/parent transforms, before Bevy camera projection updates and node poses. |
| `PostUpdate` | `EvaluateNodePoses` | Evaluates current node poses before model-particle simulation and transform propagation. |
| `PostUpdate` | `SimulateModelParticles` | Updates model particles using local poses before `TransformSystems::Propagate`. |
| `PostUpdate` | `SelectLod` | Selects authored geometry after camera updates and transform propagation, before Bevy visibility propagation and view checks. |
| `PostUpdate` | `SimulateEffects` | Updates PRE2 and ribbons after propagation, using current global transforms. |
| `PostUpdate` | `DispatchEvents` | Emits crossed model event keys after propagation, with occurrence-time node poses. Read messages after this set or in the next `Update`. |

Applications moving driving cameras or model roots in `PostUpdate` should run
before `Wc3Systems::EvaluateNodePoses`; when playing model-camera bindings, move
source roots and camera parents before `Wc3Systems::AnimateCameras`.

Feature systems are registered in the top-level plugin so the ordering across
features stays visible. PRE2 and ribbon GPU registration remains in their render
plugins. The module boundaries do not promise independently installable runtime
feature plugins.

## Tests and rendering checks

Private implementation tests live beside their modules, using dedicated test
files for larger suites. Public decoding/loading/material tests and their fixtures
remain under `tests/`. `examples/capture.rs` is the shared offscreen capture tool;
the [viewer and capture guide](tools.md) explains repeatable simulation and
visual comparison. Captures verify controlled fixture behavior, not equivalence
to Warcraft III.
