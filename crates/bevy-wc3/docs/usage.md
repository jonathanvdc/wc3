# Use bevy-wc3 in an application

Register `Wc3BevyPlugin` with `DefaultPlugins`, as shown in the
[crate quickstart](../README.md#add-it-to-an-application). This guide covers the
next steps: wait for loading, choose an animation, supply textures, and integrate
models with your application. The [viewer](tools.md) is useful for checking a
model before writing application code. Short fragments below assume an existing
Bevy system or setup routine with the named commands, handles, and asset stores
in scope; the animation system is complete.

## Load and spawn models

Place a model under a registered Bevy asset source and spawn a request:

```rust
use bevy::prelude::*;
use bevy_wc3::ModelInstance;

commands.spawn((
    ModelInstance(asset_server.load("units/footman.mdx")),
    Transform::default(),
));
```

After required dependencies load and geometry compiles, the root gains `ModelRig`
and `ModelSource`. Query `ModelRig` to find ready instances. Required companion
failures stop loading. `ModelInstanceError` on the request root reports model,
recursive-dependency, and cyclic-ownership failures. Asset failures retain the
request for retry or reload, and success clears the error. Bevy's `AssetServer`
also exposes load and recursive-dependency states. Optional missing images and
unsupported rendering behavior remain in asset and compiled diagnostics.

`ModelLoadError` distinguishes native decoding or import, missing required
companions, failed source reads, and compilation. Missing companions retain the
typed request and candidate paths; failed companion reads retain the request and
failing path. Native errors retain their root context and typed codec or
conversion cause through standard error sources. Compilation and image failures
use `ModelError`, whose `contexts()` identifies preparation operations, model or
part resources, and resolved asset paths. Its error source retains the concrete
validation, image, atlas or provider error. Construct callback failures with
`ModelError::from_source(error)`; reserve `ModelError::new(message)` for application
contract failures without an underlying error value. Use typed context and error
sources for policy, since displayed explanations can change.

Direct image failures retain Bevy's original load wrapper in `ImageLoadError`.
Its standard error source exposes BLP and Bevy image-loader causes. If an
application registers another image loader, inspect `ImageLoadError::load` and
Bevy's `AssetLoaderError::error()` downcast API for that loader's opaque error.

Rendering diagnostics carry a `kind`, `severity`, and typed semantic `source`.
Use those fields to select application policy; messages explain the limitation
and can change between releases. `Unsupported` identifies missing interpreters,
`Approximation` identifies changed or partial behavior, and `Unverified` identifies
implemented behavior whose fidelity still needs checking. `Resource` reports
unresolved resources and `Budget` describes bounded simulation or storage.
For example, a model inspection tool can report missing interpreters separately
from resource failures or fidelity limits:

```rust
use bevy_wc3::RenderDiagnosticKind;

for diagnostic in compiled.diagnostics() {
    if diagnostic.kind == RenderDiagnosticKind::Unsupported {
        eprintln!("{:?}: {}", diagnostic.source, diagnostic.message);
    }
}
```

Model axes and declared units convert beneath the root. Application transforms
use Bevy's Y-up frame; choose a root scale appropriate for your scene. Native
model metadata does not supply a calibrated physical scale.

### Replace or reload an instance

Insert another `ModelInstance` on a populated root to replace its renderer-owned
subtree. Application children, numeric playback clocks, and compatible overrides
survive. Asset changes also rebuild instances. Clip and node IDs belong to one
decoded graph, so selections migrate only through unique nonempty names.
`ModelRebind` reports migrations and resets. Authored camera bindings need explicit
retargeting after replacement.

## Control animation

Standalone instances begin in the authored rest state, with shared programmes.
Choose a typed clip from the ready rig and call `Playback::play` once when the
selection changes. For example, this system starts the first available clip on
new rigs:

```rust
use bevy::prelude::*;
use bevy_wc3::{ModelRig, Playback};

fn start_animation(mut models: Query<(&ModelRig, &mut Playback), Added<ModelRig>>) {
    for (rig, mut playback) in &mut models {
        if let Some((clip, _name)) = rig.model().clips().first() {
            playback.play(*clip);
        }
    }
}
```

Register it in `Update` after `Wc3Systems::SpawnInstances`. Repeated `play` calls
restart the clip. `play` uses the model's transition duration;
`play_immediately` switches directly, and `play_with_blend` accepts a `Duration`.

Playback clocks use seconds. Set `paused` or `speed` to control advancement.
`clip_time` drives the selected clip, while `shared_time` drives independent
clocks. `seek(seconds)` skips crossed events; direct clock edits keep normal
forward traversal. `restart` resets clocks and includes start events again.
Invalid selections produce `PlaybackError` during evaluation. Its optional
`wc3::model::Error` retains the typed reason, requested clip identity, and
structured context; nonfinite clocks distinguish `playback.clip_time` from
`playback.shared_time`. Inspect `kind()` and `location()` for application policy,
and format the error when presenting it. Successful evaluation clears the
component's stored failure. See [transitions](rendering/animation-blending.md) and
[event delivery](rendering/events.md) for timing details.

`PoseOverrides` maps source `NodeId`s to authored local transforms after animation
sampling. Use it for gameplay poses, IK, or an external solver. Removing an
override resumes authored animation.

## Resolve resources and textures

The default resolver searches beside the model, then under its asset root, trying
the literal filename and BLP, DDS, PNG, and TGA candidates. Native file-data IDs
and custom resource namespaces require an application `ResourceResolver`.
Insert `AssetSettings` with that resolver before adding the plugin. MPQ mounting
is available through [bevy-mpq](../../bevy-mpq/README.md).

Customize `texture_resource` for replacement selection and `texture_candidates`
for lookup policy. Each request includes the source declaration, sampled input,
role, effective sampler, color space, and structural address where applicable.
Inputs can share a source while using different sampling or color interpretation.
The loader derives shared variants without modifying source images. Color roles
default to sRGB; normal and numeric roles default to linear. Explicit declarations
take precedence. See [image requests](rendering/universal.md#image-requests) for
compiler details.

M2 companions, WMO groups, and external model references use this same resolver.
Providers must supply WMO dialect/layout provenance. Native import is strict by
default; `ResourceResolver::import_options` can explicitly accept conversion loss.
Accepted loss remains diagnostic and cannot repair malformed data or missing
required files.

`ResourceResolver::child_model_candidates` receives a `ChildModelRole` for
attachments, geometry particles, or adopted particle emitters. M2 emitters can
request both particle roles independently. With `compile_model_with_options`,
bind an already resolved model through `CompileOptions::child_models`:

```rust
options.child_models.insert(
    ChildModelKey::attachment(feature),
    ChildModelBinding::new(child_model).with_coordinates(CoordinatePolicy::Child),
);
```

Each binding contains its own coordinate policy. Keys belong to the source
model; foreign feature IDs and roles without an authored dependency are rejected.
M2 geometry and adopted-emitter resources resolve independently. See
[model particles](rendering/prem.md) for geometry children and
[sprite particles](rendering/pre2.md) for adopted emitter templates and their
remaining limits.

BLP0/1/2 decoding is included. BLP0 needs its sibling `.b00` in the same asset
source; compatibility with real beta-game BLP0 assets remains unverified. Only
base-level pixels are uploaded, rather than authored BLP mip chains.

### Replace images per instance

Insert `ImageReplacements` beside an asset-backed `ModelInstance`:

```rust
use bevy_wc3::{ImageReplacements, ModelInstance};

commands.spawn((
    ModelInstance::new(model),
    ImageReplacements::replaceable(1, team_color_image),
));
```

Its maps support three levels of choice, in descending precedence:

| Map | Replaces |
| --- | --- |
| `requests` | An exact `TextureRequestKey` interpretation. |
| `resources` | A source-model image declaration. |
| `categories` | A native replaceable category. |

`CompiledModel::image_requests()` lists requests, including unresolved defaults.
Supplied handles identify source images; the renderer derives sampler/color-space
variants and reuses geometry. Source images must remain CPU-resident for this work.
Changes to source images rebuild their variants.

Categories propagate to authored attachments and model particles, with child
choices taking precedence. Resource/request IDs stay local to their source model;
application-owned children do not inherit replacements. Changing bindings rebinds
the owned hierarchy, preserving root playback and overrides but restarting child
clocks, physics, particles, and ribbons. Keep bindings stable when these transient
states must persist.

## Choose quality and scene features

Use `Lod::Fixed` or `Lod::Automatic` to select authored geometry, configure global
`LodSettings` or a root `LodOverride`, and inspect `LodState`. Automatic selection
uses projected rest bounds and hysteresis. It does not simplify geometry or lower
effect budgets. See [LOD](rendering/lod.md).

Set ordinary Bevy `RenderLayers` on a model root to select cameras and lighting
layers for its renderer-owned meshes, effects, and authored lights. Changes apply
to existing draws and newly loaded or replaced content. Nested models inherit the
nearest explicit model root selection; removing it resumes the parent selection
or Bevy's default layer zero. A `RenderLayers` write on an individual renderer
entity takes precedence until removed. Application-owned children keep their own
Bevy layer choices.

Imported [lights](rendering/lights.md) are ordinary Bevy lights. Root
`ModelLightSettings` controls conversion and suppression; scene ambient resources
remain application-owned. Authored [cameras](rendering/cameras.md) are opt-in
bindings to application views. [Static-world APIs](rendering/worlds.md) expose
portal/group selection and environment samples.

`PreparedModel::features()` exposes typed feature/node identities and kinds,
including features without a visual interpreter. Inspect the source graph during
compilation when you need complete unsupported payloads.

## Mount models on sockets

`ModelAttachment` retains the semantic attachment definition on an animated mount.
Its `socket()` exposes identity, visibility, and controls; `child()` exposes an
authored child resource and placement. `AttachmentState::visibility` holds the
sampled value. Applications can spawn their own children beneath mount entities;
Bevy's hierarchy owns their lifetime.

Resolved authored children load automatically. They start their first clip and
restart when the mount activates, the parent changes clip, or parent time moves
backwards. Their clocks remain independent. Positive sampled visibility opens the
mount gate; fractional visibility does not multiply child opacity. Hiding a parent
does not implicitly pause every child clock.

Children default to `CoordinatePolicy::Host`, using the host frame and units.
Select `CoordinatePolicy::Child` on the corresponding `ChildModelBinding` to retain
a child's standalone conversion. WMO placement RGB is interior illumination and
remains an integration hook, rather than automatically tinting child albedo.
Cyclic dependencies are rejected. [Model particles](rendering/prem.md) have explicit
owner cleanup for children moving outside the parent hierarchy.

## Prepare custom model sources

Use [`wc3::model::io::asset::Reader`](../../wc3/docs/loading.md) to load native
bytes and required companions without a Bevy asset source. Choose the import
policy when creating the reader. Completion returns a checked, owned
`Model<'static>` and its conversion report, so the reader and source bytes can be
released. Format-specific `io::{mdlx, m2, wmo}` imports remain useful for already
decoded documents; widen those models to Universal and call `into_owned()` if
they must outlive those documents. Ownership conversion preserves graph-scoped
identities. You can also author a Universal graph directly. Compile, prepare,
and spawn it with the ordinary APIs:

```rust
let compiled = compile_model(&model, |request| resolve_image(request))?;
let prepared = compiled.prepare(&mut meshes, &mut inverse_bindposes);
let root = spawn_model(&mut commands, &mut meshes, &mut materials,
    &prepared, Transform::default());
```

Compilation owns its output, so the source graph and bytes can be released.
Reuse the prepared value for subsequent spawns; drop compiled CPU mesh copies when
you no longer need them. `CompileOptions` selects material appearances and external
child handles. Ambiguous slot-based appearances require an explicit choice.
See [compilation](rendering/universal.md) for input contracts and diagnostics.

## Compose models and appearances

Use `assembly::Assembly` for resource plans with geometry choices, image replacements,
atlases, and mounted or skinned parts. `assembly::compile_assembly` resolves the
plan into a reusable `CompiledModel`. Applications can implement
`assembly::Resolver<Selection>` for their own selection vocabulary. The
`assembly::Models` registry borrows model views, so several plans can use
the same graph and its typed identities without copying its contents or cloning
owner handles. Keep the graphs and any borrowed native data alive through
compilation; the compiled result owns its render data. The
[assembly guide](rendering/assembly.md) covers explicit skeleton bindings and
clip mappings; `CompiledModel::compose` is available for already compiled parts.

The optional `wow::compile_character` adapter translates native character geosets,
texture categories, and joint matching into the same pipeline. See
[WoW characters](rendering/characters.md) for those rules.

Add `PhysicsSimulation` to enable supported authored secondary motion. Rigid parts
inherit settings unless overridden. `PreparedModel::physics_bodies()` and
`physics_joints()` expose owned inputs for external solvers, and `PoseOverrides`
retains application authority over simulated bones. `PhysicsContactPolicy` selects
authored grants or prepared source-local unconnected dynamic contacts; inspect
`ModelRig::physics_contact_error()` when opting into the latter. Solver limits belong in the
[secondary-motion reference](rendering/physics.md).

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
posed bounds. Geometry provenance includes `ModelDraw::purpose`: a dedicated
`DrawPurpose::ShadowOnly` caster can be visible to lights while excluded from camera
passes. When spawning baked meshes yourself, call its `configure_visibility`
method on the entity's `VisibilityClass` after inserting `Mesh3d`, whose hook adds
the camera class. The capture tool applies this routing for baked poses.
Aggregate bounds cover visible surfaces. Default output includes the
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
updates, pose evaluation, propagation, effects, and dispatch. Read events after
`DispatchEvents` or in the next `Update`, and make whole-step playback changes
before `AdvanceAnimation`.
