# Using bevy-wc3

All rendering starts from the Universal semantic graph. The asset loader supplies
native adapters and resource lookup; applications can also compile graphs they
construct or import themselves. Register `Wc3BevyPlugin` alongside `DefaultPlugins`
to install the renderer and its scheduling contract.

## Load and spawn models

Place a model under a registered Bevy asset source and spawn its typed request:

```rust
use bevy::prelude::*;
use bevy_wc3::ModelInstance;

commands.spawn((ModelInstance(asset_server.load("units/footman.mdx")),
    Transform::default()));
```

The request is replaced with `ModelRig` and `ModelSource` after
required dependencies load and geometry compiles. Querying `ModelRig`
therefore distinguishes ready models. Optional missing images and unsupported
rendering behavior remain inspectable on the asset and compiled diagnostics.
Required companion failures stop loading.

Inspect Bevy's `AssetServer` load and recursive-dependency states for loader
failures; `ModelInstanceError` reports rejected instance requests such as
cyclic dependencies, rather than duplicating asset-loader errors.

Instances share prepared assets. Inserting another `ModelInstance` on a
populated root replaces its renderer-owned subtree while preserving application
children, numeric playback clocks, and compatible overrides. Asset changes rebuild
instances. Typed identities belong to one decoded graph: clip and node selections
migrate only through unique nonempty names, and `ModelRebind` reports reset
or migrated selections. Camera feature IDs need explicit application retargeting
after replacement.

Authored axes and meters-per-unit are converted beneath the application root.
Application transforms use Bevy's Y-up coordinate frame. External children default
to `CoordinatePolicy::Host`, cancelling their standalone frame conversion;
To retain a child's own units and frame, select `CoordinatePolicy::Child`
for its feature in `CompileOptions::child_coordinates`.

## Resolve resources and textures

`ResourceResolver` maps paths, namespaced resources, and file-data IDs to
registered asset sources. Insert `AssetSettings` with the resolver before
adding the plugin; `ModelLoaderSettings` carries loader-managed dependency
ancestry. The default path resolver checks beside the model before its
asset root, trying the literal texture filename and BLP, DDS, PNG, and TGA
candidates. Named sources remain intact. Applications can register MPQ sources
through the separate [bevy-mpq crate](../../bevy-mpq/README.md); archive mounting is
outside this renderer.

The resolver's `texture_resource` and `texture_candidates` hooks select replacement
images and lookup policy. A request carries its structural input address where applicable, sampled-image
value, resolved resource declaration, role, effective color space, and effective
`request.sampler`; provider callbacks receive a source-list index separately.
Each material input owns its sampling settings. Inputs sharing one image dependency
can request different sampler or color interpretations, and the loader shares equal resolved image variants after replacement lookup.
Input addresses distinguish equal material or effect inputs during replacement
lookup. Native metadata remains available to source resolution; physical image
reuse depends only on the resolved source and effective sampling and color settings.
An unspecified color space
follows the role: color uses sRGB, normal and numeric maps use linear sampling.
Distinct interpretations receive distinct handles without mutating shared images.
Explicit semantic color-space declarations take precedence.

BLP0/1/2 decoding is included. BLP0 requires its sibling `.b00` payload in the same
asset source. Only base-level pixels are uploaded; authored BLP mip chains are not.
Compatibility with real beta-game BLP0 assets remains unverified.

Native imports preserve unsupported data strictly by default. Providers can
override `ResourceResolver::import_options` for an explicit loss policy.
Accepted dropped or approximated native interpretations remain compiled
diagnostics; permissive policy does not repair malformed references or resolve
missing required companion files.

Native M2 skins and external animation data, WMO groups, and external model
references use the same resolver boundary. WMO dialect/layout provenance requires provider knowledge. The [semantic rendering
guide](rendering/universal.md) explains native adapter limits.

## Control animation

`Playback` uses seconds and typed `ClipId` selections.
Obtain available clips from `ModelRig::model().clips()` and call `play`
when the desired clip changes. `play` restarts the destination and uses the model's
transition duration; `play_immediately` switches directly, and `play_with_blend`
accepts an explicit `Duration`. A default standalone root samples authored rest
state and shared programmes until a clip is selected.

Set `paused` or `speed` to control advancement. `clip_time` drives ordinary
animation through the selected clip source interval; `shared_time` independently
drives shared clocks. `seek(seconds)` skips events over the seek
interval, while direct clock edits retain normal forward event traversal. `restart`
resets clocks and includes start events again. Invalid typed selections produce
`PlaybackError` on evaluation. See [animation
blending](rendering/animation-blending.md) and [events](rendering/events.md).

`PoseOverrides` maps source `NodeId` to authored local transforms after
animation sampling. Remove an override to resume authored animation. This is the
integration boundary for gameplay, physics, and procedural solvers.

## Choose geometry quality and scene features

`Lod::Fixed` or `Automatic` selects authored geometry. Configure global
`LodSettings`, or a root `LodOverride`, and inspect
`LodState`. Automatic selection uses projected rest bounds, pixel
thresholds, quality bias, and hysteresis. It does not simplify geometry or reduce
effect budgets. See [geometry LOD](rendering/lod.md).

`ModelLightSettings` on a root controls imported light power, range conversion,
and shadow permission. Imported lights remain ordinary Bevy lights and illuminate
other assets. `ModelLight` retains authored definitions and per-light
suppression; scene ambient resources remain application-owned. See
[lights](rendering/lights.md).

`PreparedModel::features()` exposes typed feature/node identities and
kinds, including unsupported interpreters. Complete unsupported payloads are not
retained as owned hook components; inspect the graph or provider compile-options
callback when an application needs those definitions.

## Mount models on sockets

`ModelAttachment` retains the complete Universal `Attachment` definition on an
animated mount. The wrapped definition's `socket()` accessor exposes socket identity,
authored scalar visibility, and controls; `child()` exposes any child resource and
placement appearance. `AttachmentState::visibility` holds the current sampled visibility
separately. Resolved external model references load automatically. Child
instances start their first clip and restart when the mount activates, the parent
changes clip, or parent time moves backwards. Their rigs and clocks remain
independent. Hierarchy visibility gates rendered content; hidden parents do not
implicitly pause every child animation clock. A positive sampled visibility opens
the mount gate; fractional values are retained but do not multiply child opacity.

WMO placement RGB is interior illumination, not a generic albedo tint; the renderer
does not multiply it into child color or overwrite scene lighting. Applications
interpreting it also need interior/exterior region membership, ambient lighting,
interior light direction, and any linked world-light definition. Uninterpreted
illumination and controls produce diagnostics.

Applications may spawn their own children beneath mount entities. The Bevy hierarchy
owns their lifetime. Cyclic automatic model dependencies are rejected. World-space
model particles maintain explicit owner cleanup despite their detached motion; see
[model particles](rendering/prem.md).

## Prepare custom model sources

Import through `wc3::model::io::{mdlx,m2,wmo}` as appropriate, widen to
`Universal`, or author a Universal graph directly. `compile_model` borrows
the model and resolves image requests into owned runtime definitions. It does not
export through a native model version.

```rust
let compiled = compile_model(&model, |request| resolve_image(request))?;
let prepared = compiled.prepare(&mut meshes, &mut inverse_bindposes);
let root = spawn_model(&mut commands, &mut meshes, &mut materials,
    &prepared, Transform::default());
```

The source graph and bytes may be released after compilation. Keep the prepared
value to spawn repeatedly; drop the compiled value when its CPU mesh copies are no
longer useful. `CompileOptions` selects typed material appearances and
external child handles. Ambiguous slot-based appearances require an explicit
selection rather than choosing an arbitrary first material.

## Compose models and appearances

`CompiledModel::compose` combines independently compiled components into a reusable
model definition. A `PartBinding::Skinned` supplies an explicit `JointMap` and
part-to-host rest transform. Mapped joints reuse the host skeleton; inverse binds
retain the equipment's authored rest space. Unmatched weighted joints must descend
from a mapped joint, so extra chains can animate without silently creating another
host skeleton. `PartBinding::Mounted` takes the semantic `ModelMount`
definition and retains an independent child model. `MountTarget::Node` selects
a host node; `MountTarget::Socket` follows an exposed socket and its authored
activation gate. Resource-only placements cannot be selected as sockets.
The same source asset can be instantiated more than once.

Prepare and spawn the combined definition through the ordinary APIs. Meshes and
bind palettes are shared across rigid occurrences. `PreparedModel::mounts()` and
`ModelRig::mounts()` expose matching ordered definitions and live child roots.
Explicit clip mappings synchronize a child's clip to host seconds; independent
rigid clips retain their own clocks while following host pause and playback rate.
Skinned clip mappings require matching clip source starts.

For resource-driven setup, `assembly::Assembly` contains root and component
sources, typed draw selections, image dependency and per-input replacements and explicit bindings.
`assembly::compile_assembly` resolves images and compiles the complete plan.
Applications can implement `assembly::Resolver<Selection>` for their own typed
configuration schema. Atlas composition and repeated-source compilation happen
once during setup; see [model assembly](rendering/assembly.md).

The optional `wow::compile_character` adapter translates native character geosets,
texture categories and joint-matching rules into that same pipeline. Its race,
body-category and inventory vocabulary stays within the adapter. See
[WoW characters](rendering/characters.md) for those interpretation rules.

Add `PhysicsSimulation` to the root to enable authored secondary motion. Rigid
components inherit simulation settings unless the application overrides them.
`PreparedModel::physics_bodies()` and `physics_joints()` retain owned authored
inputs for external solvers. Application `PoseOverrides` retain authority over
simulated bones; unsupported native controls remain diagnostic.

## Bake a static pose

`PoseBaker` freezes geometry and materials without an ECS world or GPU. It
borrows prepared mesh and inverse-bind assets, samples the same signed pose
evaluator, and deforms up to eight influences. Resolved image handles remain
unchanged; the baker does not rewrite images.

```rust
let mut baker = PoseBaker::default();
let pose = baker.bake(&prepared, &meshes, &inverse_bindposes,
    PoseOptions { playback: Some(&playback), ..default() })?;
```

Output includes every prepared geometry pass and generated liquid grid, with typed
source identity, material snapshot, visibility, clocks, optional LOD selection, and
posed bounds. Aggregate bounds cover visible surfaces. Default output includes the
Bevy frame conversion; `BakeSpace::Authored` retains authored units.
Camera-sensitive nodes require an explicit camera in authored coordinates.

`PoseOptions::world_selection` intersects authored visibility with world
group/portal selection; absence uses the runtime default. Baking does not simulate
transitions, events, effects, physics, or automatically resolved attachment children.
Composed rigid mounts are included recursively. Their default snapshot clocks use
host elapsed seconds and any explicit clip mapping; `PoseOptions::mounted_playback`
accepts live child clock snapshots, keyed by ordinal mount paths, after independent
seeks or clock edits. Invalid clocks, missing assets and malformed skin streams
return errors.
`PreparedModel::parts()` exposes this rig's undeformed shared mesh, material
defaults, and the same `ModelDraw` provenance used by live passes.

## Integrate with application systems

Use `Wc3Systems` in the same schedule as the renderer stage being targeted.
[Architecture](architecture.md#scheduling-contract) explains ordering around camera
updates, pose evaluation, propagation, effects, and dispatch. Camera bindings and
event consumers are described in their focused guides; the [tool guide](tools.md)
covers viewer and capture commands.
