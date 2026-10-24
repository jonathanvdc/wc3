# Compilation and native resources

All models pass through `Model<Universal>` before rendering. Native MDLX, M2, and
WMO adapters supply that graph, and applications can author it directly.
Compilation produces reusable geometry and runtime definitions without exporting
to an intermediate native format. For application code, start with
[custom model sources](../usage.md#prepare-custom-model-sources).

## Compile, prepare, spawn

`compile_model` borrows the graph and creates a `CompiledModel` with owned CPU
meshes, animation, materials, and feature definitions. The graph and encoded source
can then be released. `CompiledModel::prepare` inserts shared meshes and inverse
bind palettes into Bevy assets and returns a `PreparedModel`. `spawn_model` uses
that prepared value to create an independent rig beneath an application transform.

Reuse the prepared model to avoid repeating setup. Meshes, bind palettes, and
animation definitions are shared. Constant materials share handles; animated
surfaces keep private material assets and only mark them modified when their
sampled appearance changes. The [architecture guide](../architecture.md) describes
the implementation and scheduling boundaries.

Asset-backed instances populated by the plugin reuse complete equal mesh and
inverse-bind assets across different `ModelAsset` values in the same World.
Sharing checks exact attributes, indices and mesh metadata; it does not combine
draws, materials, source IDs or animation state. This saves repeated asset storage
and uploads after compilation; each source still performs its own geometry
projection. Standalone `CompiledModel::prepare` creates fresh assets per call,
sharing repeated geometry within that preparation.

For the same reuse in a manual pipeline, keep a `PreparedGeometryCache` and pass
it to each `CompiledModel::prepare_with_cache` call. The cache stores candidate
IDs rather than strong handles or mesh buffers; the returned prepared models
own their assets. Dropping the cache leaves those prepared models usable.
The model plugin initializes this cache as a resource; manual systems can borrow
the same resource with `ResMut<PreparedGeometryCache>`.

```rust
use bevy_wc3::PreparedGeometryCache;

let mut geometry = PreparedGeometryCache::default();
let prepared = compiled.prepare_with_cache(&mut meshes, &mut bindposes, &mut geometry);
```

Treat renderer-generated meshes as shared data. Editing a shared mesh through
`Assets<Mesh>` affects every instance using its handle. Clone the mesh into an
application-owned asset before changing its vertex data independently.

## Geometry and coordinates

The compiler projects selected draw ranges and vertex, face, corner, and whole-mesh
attributes into Bevy meshes. Triangle strips preserve winding; polygons are
triangulated along their boundaries. Mixed connectivity becomes consecutive
compatible draws. Missing coordinate streams or unsupported attribute bindings
fail compilation instead of selecting arbitrary values.

Vertices merge only when their complete compiled attributes agree, preserving
seams. Complete equal meshes share storage, including across otherwise distinct
source selections. Authored tangent differences remain distinct. Each skinned draw
uses a compact palette of its contributing joints and matching inverse binds;
the authored skeleton remains available for animation and attachments. Explicit
inverse binds and identity bindings retain different meanings. More than eight
active influences per vertex is an error rather than a truncation.

Model axes and declared units are converted beneath the application root into
Bevy's Y-up, negative-Z-forward frame. Native model metadata does not establish a
physical length scale; apply an application root scale where needed. A single
`Playback::driving_camera` supplies camera-sensitive poses for each instance.

Skinned bounds conservatively enclose positive-weight joint contributions. Changed
transforms and source assets refresh them; hidden geometry defers this work until
revealed. Singular or unavailable bounds disable frustum culling temporarily.

## Image requests

The image callback receives a `TextureRequest`: a sampled-image value, its
structural input address where applicable, the resolved image declaration, role,
effective sampler, and color space. Resource declarations own native filenames;
material and effect inputs own sampling and coordinate settings.

One source image can therefore need several interpretations. Color roles default
to sRGB; normal and numeric roles default to linear unless explicitly declared
otherwise. Native loading and assembly share images when the resolved resource,
effective sampler, and transfer function agree. They do not change shared source
pixels or use native metadata as part of that physical cache key.

`TextureRequestKey` identifies a logical request. Assembly replacements target image
dependencies or structural input addresses, with input choices taking precedence.
See [assembly image selection](assembly.md#select-appearance-geometry-and-images) and
[per-instance replacements](../usage.md#resolve-resources-and-textures).
When a selected material contains a texture-valued animation slot, discovery
conservatively retains all sampled images in the model, including images that may
only be selected by a curve.

## Native loading

`ModelLoader` drives the shared
[`wc3::model::io::asset::Reader`](../../../wc3/docs/loading.md) to load a checked,
owned Universal graph and its conversion report, then compiles the graph. Native
format dispatch, companion traversal, and bundle validation belong to that
renderer-independent reader. `ModelLoadError` preserves typed native causes and
root context. Missing companions carry their request and candidate paths; failed
companion reads carry the request, failing path, and original source error.
`ResourceResolver` maps paths, namespaced resources, and file-data IDs into
registered Bevy asset sources. The default resolver tries the literal filename,
then BLP, DDS, PNG, and TGA candidates beside the model and under its asset root.
Named sources and authored filename case remain intact.

Use `texture_resource` for replacement selection, `texture_candidates` for lookup
policy, and `compile_options` for material appearances and other compilation
choices. Missing required companions fail loading. Optional missing images remain
inspectable on `ModelAsset`.

M2 particle geometry (`GPID`) and emitter-template (`RPID`) chunks import as
separate typed child-model resources, including file-data IDs. Zero table entries
retain a legacy filename fallback. The loader resolves both roles through the
same model-dependency graph; [model-particle runtime coverage](prem.md) remains
separate from successful resource discovery.

M2 bundles resolve animations and attachment tables through their external skeleton
chain. Nonempty deferred tracks need that context before validation or writing.
Empty equipment sequences do not invent sibling animation dependencies; explicit
animation-file declarations remain required. WMO dialect provenance must be supplied
by the provider because the file version alone may not identify its layout.

`ResourceResolver::import_options` defaults to strict preservation. Accepting
conversion loss is an explicit provider choice; diagnostics retain the dropped or
approximated interpretation. A permissive policy cannot repair malformed references
or supply missing companions.

## Diagnostics and fidelity

Successful decoding establishes neither shader support nor native rendering
fidelity. Compilation reports unsupported material programmes, effects, controls,
and approximations against their source identities. Inspect these alongside asset
load errors when investigating a model.

`ModelNode`, `ModelDraw`, and feature metadata preserve source identities.
`PreparedModel::features()` exposes IDs and kinds even when a visual interpreter
is unavailable. Generic hooks do not retain complete unsupported payloads; inspect
the source graph during compilation if an application needs them.

The [renderer overview](renderer.md) summarizes support. Material and effect guides
carry the detailed inventories so those limits have one primary home.

## Static worlds

World groups, placement sets, portal visibility, liquid grids, and environment
sampling are covered in [static-world integration](worlds.md). These APIs expose
scene data while leaving global lighting, fog, camera choice, and containment to
the application.
