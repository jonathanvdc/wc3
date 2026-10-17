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
`Models` maps resource identities to `Model<Universal>` values. An
`Assembly` has a root `Source` and an ordered list of `Part`s.
Each source selects its model resource, a `DrawSelection`, and image replacements
by dependency or by structural input address. Each part supplies a `PartBinding` and optional host-to-part clip
mappings. Public assembly plans use concrete `PartBinding` values, so their
attachment and skeleton relationships are explicit.

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

## Select geometry and images

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
crops, destination rectangles and byte opacity. The generic compiler uses explicit
dependencies and input addresses; it does not infer native replacement categories.

`compose_atlas` applies checked source-over composition in the decoded sRGB byte
domain, including source and destination alpha, with nearest-neighbor scaling.
Invalid dimensions, pixel storage and rectangles are errors. Image callbacks
supply `RgbaImage` values; source decoding is cached within an assembly compilation.
After replacement resolution, equal sources, effective color spaces and samplers
share a Bevy image. Different sampling or color interpretation receives a separate
image; native metadata and input addresses do not enter the physical cache key. Atlas composition happens during setup, rather than animation updates.

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
rigid components follow the host's pause state and playback rate. Controller-driven
skinned parts require matching authored timeline origins. `PoseBaker` includes
composed components and attachment visibility; explicit mounted playback snapshots
can supply independent child clocks.

## Compile and reuse

`compile_assembly` accepts the resolved plan, semantic model map, Bevy image store
and decoded-image callback. It validates selections, compiles sources, resolves
bindings and returns a `CompiledModel`. Prepare and spawn it with the ordinary
renderer APIs. Failed setup returns no partial assembly; images already inserted
remain available in the asset store.

Compatible skinned parts that refer to the same resource coalesce into one rig
with united geometry selections. Bindings, texture replacements and clip maps must
agree. Conflicting policies fail explicitly. Repeated identical rigid sources
reuse compiled geometry, and preparation shares meshes and inverse-bind palettes
across occurrences. Live rigs retain independent state. Reuse compiled and
prepared assemblies across instances to amortize setup work.

`PreparedModel::mounts()` and `ModelRig::mounts()` expose corresponding component
definitions and live roots. `PoseOverrides` provides application pose control.
`PhysicsSimulation` enables supported authored rigid-body secondary motion, with
settings inherited by rigid components unless overridden. Native physics coverage
and approximations are described in the [WoW guide](characters.md#secondary-motion).
Assembly itself introduces no additional simulation or shading pipeline; all parts
continue through Bevy PBR, lighting, skinning and the renderer's existing limits.
