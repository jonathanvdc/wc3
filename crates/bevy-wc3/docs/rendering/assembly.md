# Model assembly

The `bevy_wc3::assembly` module builds reusable model assemblies from imported or
application-authored semantic graphs. A character with equipment, a modular robot
and a building with attached machinery use the same assembly operations. The
application owns the vocabulary used to choose those resources and appearances.

## Resolve application choices

`Resolver<Selection>` accepts an application-defined selection type and
the available `Models`. Applications can use their own enums, identifiers,
body variants, collections of components or continuous parameters. The resolver
translates those choices into supported semantic geometry, images and bindings;
the renderer supplies no race, sex, customization-slot or inventory schema.

Load or author the required models before resolving graph-scoped identities.
`Models<'view, 'data>` maps resource identities to `&'view Model<'data>` views.
Keep each source graph and any native data it borrows alive through compilation.
The application can own graphs directly or retain them in an `Arc` cache; building
a registry borrows those owners without copying graphs or cloning their handles.
The native [asset reader](../../../wc3/docs/loading.md) returns an independent
`Model<'static>` suitable for caching, while format-specific imports can borrow
native records for immediate compilation. All views retain the same graph-scoped
node, look, draw and feature identities.
An `Assembly` has a root `Source` and an ordered list of `Part`s.
Each source selects its model resource, material look, `DrawSelection` and image
replacements by dependency or structural input address. Each part supplies a
`PartBinding` and optional host-to-part clip mappings. Public assembly plans use
concrete `PartBinding` values, so their attachment and skeleton relationships are
explicit.

For example, a resolver can attach a sensor resource to an application-selected
node without introducing an inventory slot:

```rust
use bevy_wc3::assembly::{compile_assembly, Assembly, Part, Source};
use bevy_wc3::PartBinding;
use wc3::model::{ModelMount, MountTarget};

let mut plan = Assembly::new(Source::new(chassis_resource));
plan.parts.push(Part::new(Source::new(sensor_resource), PartBinding::Mounted(
    ModelMount {
        transform: mounting_transform,
        ..ModelMount::new(MountTarget::Node(mounting_node))
    },
)));
let compiled = compile_assembly(&plan, &models, &mut images, decode_image)?;
```

The [WoW adapter](characters.md) is one application of this machinery. It interprets
native geoset groups, replacement categories and bone identities before entering
the same assembly compiler. Its character database vocabulary is optional.

## Select appearance, geometry and images

`Source::material` uses the same `wc3::model::MaterialSelection` as direct
compilation. Its default uses the graph's authored default look; choose
`MaterialSelection::Draw(look)` for an explicit complete draw-material assignment.
Draws in a source without a default look require an explicit selection. Look identities
belong to that source model, so the plan rejects an absent or foreign look even
when its geometry is hidden, before decoding images. The source choice is passed
to the ordinary Universal compiler alongside its draw selection.

`DrawSelection` chooses typed draws and suppresses complete instances. Its default
retains authored geometry. Assembly validation rejects foreign draw and instance
identities, image dependencies and input addresses absent from the source graph. The compiler resolves
materials and images for selected geometry, together with retained scene features
and effects; a hidden hair draw therefore does not require its unused image.

`Source::images` maps `ImageResourceId` to a replacement for every use of that
image dependency. `Source::inputs` maps `TextureInput` to a replacement for one
material-step slot or sprite-emitter slot. An input replacement takes precedence
over a resource replacement and applies throughout that input's animated image
selections. Both maps replace only the image source: color interpretation,
sampler and UV transforms remain owned by the authored input. Editing a sampler
therefore does not change which replacement matches.

Use `texture_requests(model)` to inspect supported input addresses. Material
addresses contain the owning `MaterialId`, step index and image slot; particle
addresses contain the owning `FeatureId` and image slot. Equal input values remain
independently addressable. Addresses refer to structural positions, so changing
the order of material steps or image slots requires updating the plan.

`TextureReplacement::Image` selects a decoded resource.
`TextureReplacement::Atlas` defines dimensions, a clear color and ordered
`TextureLayer` contributions. Layers carry resource identities, optional source
crops, destination rectangles, byte opacity and an `AtlasBlend` equation. The generic compiler uses explicit
dependencies and input addresses; it does not infer native replacement categories.

`compose_atlas` blends in the decoded sRGB byte domain with nearest-neighbor
scaling. `AtlasBlend::SourceOver` is the default normalized straight-alpha equation,
including destination alpha. `Replace` copies source RGB and opacity-adjusted alpha,
including RGB beneath transparent pixels. `Alpha` interpolates RGB by source alpha
and uses Sa² + Da(1-Sa) for alpha; `AlphaStraight` instead uses Sa + Da(1-Sa).
`Multiply`, source-conditioned `Overlay` and `Screen` first combine encoded colors,
then use the `Alpha` equation. Native appearance adapters select these equations
explicitly; generic layers retain the default.
Invalid dimensions, pixel storage and rectangles are errors. Image callbacks
supply immutable `RgbaImage` values whose clones share pixel storage; source decoding
is cached within an assembly compilation.
Across the assembly's source compilations, equal resolved image sources, effective
color spaces and samplers share a Bevy image. Different sampling or color interpretation receives a separate
image; native metadata and input addresses do not enter the physical cache key. Atlas composition happens during setup, rather than animation updates.

## Supply prepared external dependencies

Use `compile_assembly_with_dependencies` when a source has resolved child models
or an external [particle programme](popcorn.md). Its additional callback receives
the resource identity and owning `Model<Universal>` and returns `SourceDependencies`
containing the existing `ChildModelKey` bindings and prepared `ParticleEffect` values.
Load and prepare those dependencies before compiling the assembly. For example,
an application can keep its already prepared bindings in a resource-keyed map:

```rust
use bevy_wc3::assembly::{compile_assembly_with_dependencies, SourceDependencies};
use std::collections::HashMap;

let prepared: HashMap<_, SourceDependencies> = prepared_dependencies;
let compiled = compile_assembly_with_dependencies(
    &plan,
    &models,
    &mut images,
    |resource, _model| Ok(prepared.get(resource).cloned().unwrap_or_default()),
    decode_image,
)?;
```

Resolution runs once per distinct resource, root first and then in first occurrence
order, after plan validation. All returned feature keys and child roles are checked
against their owning source before model images are decoded, even when geometry is
hidden. Empty bindings retain ordinary missing-resource or missing-interpreter
diagnostics. Resolver failures return no partial assembly; any callback-side assets
remain owned by the application.

Bindings are resource-scoped within this compilation: repeated occurrences use the
same prepared dependencies, while each live rig retains independent clocks, particles
and controls. Different dependencies for two occurrences of the same resource require
distinct application resource identities. Programme images, fixed attributes and
finite limits stay in `ParticleEffect`; `Source::images` and `Source::inputs` do not
replace those separately prepared images. Child coordinate and recurrence policies
remain those of `ChildModelBinding` and the ordinary child loader. Other compiler
settings, including the physics contact-pair budget, retain their defaults.

## Bind components

`PartBinding::Mounted` accepts `wc3::model::ModelMount`, the same placement
vocabulary used to describe authored attachment children. `MountTarget::Node`
retains an independent rig beneath a host node. `MountTarget::Socket`
follows an exposed socket, including its authored visibility gate. A resource-only
child does not expose a socket and is rejected as a socket target.

The placement carries a semantic `Transform` with translation, rotation, scale and
pivot. `CoordinatePolicy::Host`, the default, uses the carrying model's
axes and numerical units. `CoordinatePolicy::Child` retains the child's
standalone frame and units using the same basis correction as automatically loaded children. The
local transform applies in the target frame before that correction. Resource
selection and clip synchronization remain independent of placement.

`PartBinding::Skinned` supplies an explicit `JointMap` and part-to-host rest
transform. Mapped joints reuse the host skeleton, while corrected inverse binds
retain the source bind space. Additional weighted joints must descend from a
mapped joint. Extra chains, effects and nested rigid children remain available.
No generic bone-name, joint-index or native checksum matching is inferred.

Clip mappings use host seconds without stretching curves. Independently animated
rigid components follow the host's pause state and playback rate. Skinned clip mappings
require matching clip source starts. `PoseBaker` includes
composed components and attachment visibility; explicit mounted playback snapshots
can supply independent child clocks.

Assemblies inside [model particles](prem.md) use the same owner-ordered startup
path. Ready parts resolve their initial placement through the owner's birth pose.
Constant-active automatic attachments apply their independent playback rate once
over that initial particle-age interval; mapped rigid clocks keep their existing
mapping. Animated attachment activation does not replay initial crossings and
reports an approximation. Pending automatic attachments can retain the initial interval across unchanged
owner clocks; elapsed owner history and reload do not reconstruct old effects.

## Compile and reuse

`compile_assembly` accepts the resolved plan, semantic model map, image sink
and decoded-image callback. Pass `Assets<Image>` to insert interpreted images
directly into the renderer's image store. The dependency-aware variant additionally accepts the
resource-scoped callback described above. Both use the same compiler and runtime.
Compilation validates selections, compiles sources, resolves
bindings and returns a `CompiledModel`. Prepare and spawn it with the ordinary
renderer APIs. The compiled result owns its geometry and animation data, so the
source model registry can be released after compilation. Failed setup returns no
partial assembly; images already inserted remain available in the asset store.

Failures return `ModelError` with inspectable `PreparationContext` frames and an
original typed error source. Frames run from innermost to outermost and identify
the operation, root resource or zero-based part position, image dependency or
input, and resolved image where applicable. Part positions always refer to the
original ordered plan, even after compatible skinned parts coalesce. Validation
uses `assembly::SelectionError` to retain the offending look, draw, instance,
image or clip identity. Atlas failures retain `AtlasError`; dependency and image
callbacks retain errors supplied through `ModelError::from_source(error)`.
Follow `std::error::Error::source` to inspect those causes. Display text includes
context and the immediate cause for reporting and does not define a parsing API.

For preparation on a worker, supply an `ImageSink` implementation or an
`FnMut(Image) -> Handle<Image>` callback. It receives the same interpreted pixels,
color format and sampler as immediate insertion. Reserve handles through a clone
of the destination store's `Assets<Image>::get_handle_provider()`, and retain
each strong handle together with its image until the main thread publishes it
with `Assets::insert`. All reservations and publication must use that same store;
independent `Assets<Image>` collections can allocate equal IDs. Publish the batch
before exposing the compiled model to rendering. Images may be emitted before a
later compilation error, so discard the staged images and handles if preparation
fails or its result is no longer needed. The application owns this task lifetime
and publication policy; the compiler does not run tasks or mutate an ECS world.

Compatible skinned parts that refer to the same resource coalesce into one rig
with united geometry selections. Bindings, material selections, texture replacements
and clip maps must agree. Material selections compare the resolved look, so
`Default` and an explicit `Draw` of the graph's default look are compatible.
Different resolved looks fail explicitly for shared-skeleton requests. Repeated
rigid sources with equivalent selections reuse compilation; different looks
retain their own appearances. Preparation can still share equal meshes and
inverse-bind palettes across occurrences. Live rigs retain independent state.
Reuse compiled and prepared assemblies across instances to amortize setup work.

Native facial bone corrections are selected by the [WoW adapter](characters.md#facial-bone-corrections).
They preserve full affine matrices through live skinning and offline baking; generic
assembly does not infer which imported correction resources should be active.

`PreparedModel::mounts()` and `ModelRig::mounts()` expose corresponding component
definitions and live roots. `PoseOverrides` provides application pose control.
`PhysicsSimulation` enables supported authored rigid-body secondary motion, with
settings inherited by rigid components unless overridden. Native physics coverage
and approximations are described in the [secondary-motion guide](physics.md).
Assembly itself introduces no additional simulation or shading pipeline; all parts
continue through Bevy PBR, lighting, skinning and the renderer's existing limits.
