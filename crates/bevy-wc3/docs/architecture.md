# bevy-wc3 architecture

[Documentation index](README.md)

The crate keeps an explicit public facade in `src/lib.rs`. Implementation modules
are private; the facade exports model assets, instance components, preparation
and spawning APIs, material types, and scheduling integration points.
`src/plugin.rs` registers assets, embedded shaders, render plugins, and runtime
systems. `src/schedule.rs` owns the cross-feature execution order.

## Loading, preparation, and instantiation

- `assets/` adapts `wc3::model` decoding and strict version conversion to Bevy,
  loads MDX/MDL and BLP assets, resolves dependency paths, and preserves child
  model resource slots. The format definitions and codecs remain in `wc3`.
- `preparation/` builds shared meshes, UV variants, joint mappings, inverse bind
  poses, and material templates. `PreparedModel` also retains the source model
  and resolved image/child-model dependencies. Preparation does not spawn entities.
- `instance/` waits for assets, caches prepared models, checks recursive child
  references, and manages ownership. `instance/spawn/` creates the root state,
  rig, geosets, and feature entities from a prepared model.

Meshes and inverse bind poses are shared between instances. Spawned materials,
texture bindings, animation clocks, rigs, and emitter simulation states belong to
individual instances. Preparation stores material values as templates; spawning
inserts private material assets. Material assets are needed only when spawning
instances, so preparation takes only mesh and inverse bind pose asset stores.

`Wc3ModelOwner` retains lifetime ownership independently of `ChildOf` transform
and visibility inheritance. Attachments use both relationships. Model particles
can remain in world space while still being cleaned up with their owner.

## Animation and rendering features

`animation/playback.rs` owns the sequence clock and playback controls.
`Wc3Animation::time()` exposes a borrowed `wc3::model::animation::AnimationTime`.
`wc3` resolves sequence/global-sequence clocks and samples tracks and properties;
`animation/sampling.rs` adapts those operations for Bevy effect callers.
`animation/pose/` owns animated node data, camera selection, pose evaluation,
and birth-time transform sampling. The ECS node system and effect birth sampling
reuse the same evaluation rules.

`event.rs` dispatches crossed event keys as `Wc3ModelEvent` messages. Prepared
instances share event definitions and retain independent cursors. Interval
traversal lives under `wc3::model::animation`; explicit playback restart/seek
metadata lives with `Wc3Animation`. Event names are interpreted by applications.

`materials/` owns Bevy material specialization, layer/geoset animation, surface
and UV animation, texture binding precedence, and private linear image variants.
`attachment.rs` and `light.rs` own their spawning and animation behavior.
`camera.rs` exposes authored views on instance roots and plays explicit bindings
on application-owned Bevy cameras without automatically creating active views.

`effects/` contains Classic model particles, PRE2 quad particles, and ribbons.
Shared simulation helpers provide clocks, emission arithmetic, and live record
management; emitter-specific modules retain capacity, expiry, and connectivity
policy. The PRE2 texture update lives beside PRE2 simulation and rendering rather
than inside the general binding API.

PRE2 and ribbons maintain immutable birth records in the main world. Their render
plugins extract snapshots, prepare resident GPU buffers, queue per-view draws,
and render shader-generated geometry. Shared `effects/records.rs` and
`effects/render.rs` implement record storage and GPU upload/allocation mechanics.
Each renderer retains its own shader layouts, pipeline specialization, sorting,
and pass semantics. Classic particles instead spawn independently animated model
instances. The current shading and ordering limitations remain documented in
[visual fidelity](visual-fidelity/renderer.md).

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
| `PostUpdate` | `SimulateEffects` | Updates PRE2 and ribbons after propagation, using current global transforms. |
| `PostUpdate` | `DispatchEvents` | Emits crossed model event keys after propagation, with occurrence-time node poses. Read messages after this set or in the next `Update`. |

Applications moving driving cameras or model roots in `PostUpdate` should run
before `Wc3Systems::EvaluateNodePoses`; when playing model-camera bindings, move
source roots and camera parents before `Wc3Systems::AnimateCameras`. This enum covers both
node poses and the other runtime stages.

Feature systems are registered in the top-level plugin so the ordering across
features stays visible. PRE2 and ribbon GPU registration remains in their render
plugins. The module boundaries do not promise independently installable runtime
feature plugins.

## Tests and rendering checks

Private implementation tests live beside their modules, using dedicated test
files for larger suites. Public decoding/loading/material tests and their fixtures
remain under `tests/`. `examples/capture.rs` is the shared offscreen capture tool;
use the same simulation FPS, times, and camera when comparing revisions. Captures
verify controlled fixture behavior, not equivalence to Warcraft III.
