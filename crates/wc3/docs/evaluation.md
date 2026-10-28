# Evaluating models without a renderer

A `Model<Universal>` can supply poses, editable draw geometry, and sampled surface
values without Bevy, image assets, or texture decoding. Native models can be widened
to `Universal` after import. Evaluation preserves semantic identities and native
surface descriptions; applications choose resource resolution, image libraries,
output encodings, and any deliberate material conversion.

## Preparing and sampling poses

Use `Model::prepare_pose` once to resolve parent indices, parent-before-child
traversal, explicit transform and facing behavior, and channel-presence masks.
`PoseLayout::index` uses a prepared identity map, so joint-palette evaluation does
not scan the hierarchy for every joint. Prepared node identities must be unique.
The resulting `PoseLayout`
is immutable and can be shared across independently sampled instances. Rebuild it
after changing node definitions or animation channel targets. Samples must belong
to the same model identities as the prepared layout. Animation compilation remains
separate: reuse
`AnimationProgram` when sampling many frames, and supply the resulting
`SampledAnimation` to the layout.

```rust
use wc3::model::{AnimationProgram, AnimationSample, ModelPose, PoseInputs};
# use wc3::model::{Model, ModelMetadata, Error};
# use wc3::model::profiles::Universal;
# let model = Model::<Universal>::empty(ModelMetadata::Mdlx(Default::default())).unwrap();
let layout = model.prepare_pose()?;
let animation = AnimationProgram::compile(&model);
let sample = animation.sample(AnimationSample::default())?;
let mut pose = ModelPose::default();
layout.evaluate_into(&sample, PoseInputs::default(), &mut pose)?;
# Ok::<(), Error>(())
```

`AnimationSample` supplies the selected clip, elapsed `clip_time`, and independent
`shared_time`. Sampling applies the clip's source interval and looping rules.
The [animation guide](animation.md) explains value slots, authoring, and selection
precedence.

`evaluate_into` retains output capacity and allocates nothing once its node and
visibility buffers are sized. `evaluate` is the allocating convenience method.
Both return full authored-space affine matrices and signed rotation/scale factors,
including mirrored or collapsed axes. Ordinary hierarchy retains shear. Native
billboards and selective inheritance use the same numerical evaluator as
`bevy-wc3`; the node guide describes camera-frame reconstruction and finite
fallbacks for singular facing inputs.
Missing camera input leaves camera-dependent corrections unapplied. Offline tools
requiring a camera-complete snapshot should require camera input themselves.

`PoseInputs` supplies an authored-space camera and complete local overrides keyed
by `NodeId`. Outputs follow `layout.nodes()` order; `layout.index(id)` resolves a
source identity. Visibility includes the ancestor chain. `PoseNode::sample` and
`PoseBehavior::evaluate` expose the same operations separately for renderers with
prepared traversal, local transitions, physics, or occurrence-time sampling.
`Transform::blend` interpolates local poses before hierarchy composition.

Node evaluation reads transform semantics, inheritance, billboard, and
camera anchoring directly from the nodes. The renderer's
[node guide](../../bevy-wc3/docs/rendering/node-flags.md) describes the native
mappings and remaining fidelity limits. Overrides replace a complete local pose;
blend local transforms with `Transform::blend` before hierarchy composition.

`cargo bench -p wc3 --bench evaluation` measures reusable pose evaluation and
draw projection without initializing a renderer. `cargo bench -p wc3 --bench
packed_storage` compares borrowed packed glam access with native `ZeroVec`
component access.
`ValueSample` accepts both ordinary and retimed animation samples, so effect
and event times do not require selecting animations again.

## Projecting and deforming geometry

Read placements through `Model::instances`. Each instance retains its `InstanceId`,
placement node, mesh, optional skin, LOD, and identified draws. Resolve each draw's
material with `material_for_draw`, using an explicit look when required. Resolve
the binding with `instance.skin.or(mesh.skin())`: a placement can select its own
skin, while some geometry vocabularies share one binding across all placements. This keeps
application selection and appearance policy separate from geometry evaluation.

`Mesh::project` resolves a mesh-owned subset into ordered point, line, or triangle
batches. Concave polygons are triangulated, strip parity is retained, and face and
corner attributes take precedence over vertex streams. Batches contain editable
positions, normals, authored tangents, selected UV sets, colors, and a `sources`
mapping back to original vertices, faces, and corners. They are expanded primitive
streams; applications can index or optimize them afterward. Missing triangle
normals are generated, while missing tangents remain empty. Bevy's adapter generates
its renderer tangents during mesh construction.

Projection stages source corners in contiguous storage with primitive boundaries.
Triangles, lines, points, and expanded strips append directly into batch buffers,
without a heap buffer per primitive. Polygon triangulation retains its scratch
storage and appends results into the same batch buffers. All selected vertex
indices are checked before triangulation; all triangulation finishes before
attribute reads or color callbacks begin.

Use `ProjectedGeometry::attribute` to gather an additional typed stream through
that source mapping. Named application fields and exact integer labels therefore
survive projection without being converted into floats or copied for renderers
that do not use them. Keep the originating mesh and subset with the batch when
requesting additional attributes. Independent attribute pools require explicit
bindings; unsupported pool interpretations return an error.

For skinned geometry, `ProjectedGeometry::skin_weights` resolves source influences
into normalized variable-length rows and a compact palette of source joint slots.
It imposes no eight-influence GPU limit. `PoseLayout::joint_matrices_into` combines
the corresponding pose matrices with the skin's inverse binds and an explicit
output frame. Pass those compact joint matrices to `SkinWeights::matrices_into`,
then apply the resulting vertex matrices with `ProjectedGeometry::deform`.
Rigid geometry instead uses its placement node matrix for every vertex.

`DeformableGeometry` offers the same deformation for borrowed mutable slices in an
application's own `Vec3` position/normal and `Vec4` tangent storage. It changes only positions, normal covectors, and tangent
directions/handedness. UVs, colors, source mappings, and other application fields
remain untouched. Singular normal transforms produce zero normals. `bounds`
computes a finite box and conservative bounding sphere after deformation.
`Model::coordinate_transform` converts authored basis and units to an explicit
target; an identity matrix retains authored coordinates. Unspecified units use a
numerical scale of one meter per unit for conversion.

## Sampling surfaces and processing the result

Surface evaluation does not require texture pixels. Each potentially animated
field references a typed slot in the model's authored `PropertyValues` table. Read a field
through `sample.animated(&field)` for lazy evaluation, or materialize a complete
snapshot with `sample.evaluate()`. The resulting `PropertyValues` has the same identities
as the authored table, so the same node, material, and effect definitions can be
used with independently sampled instances.

`sample.evaluate_into(&mut values)` replaces the output contents and resets all entries
from the authored constants before sampling. A value without an active writer
therefore returns to its authored value. This evaluates slot values while
preserving native shader descriptions and render state; it does not execute
shader programs or procedural textures. `PropertyValues` itself implements
`ValueSample`, allowing an evaluated snapshot to supply pose and material reads.

`MeshAppearance::compile` prepares geometry-owned tint and opacity controls. Rebuild
this preparation after changing geometry-animation definitions or channel targets.
Its sampled RGBA multiplier combines with surface color; `SurfaceColor` provides
the shared color/opacity/weight composition used by Bevy materials. Sampled image
values retain their shared image dependency handles, color interpretation, and
samplers after evaluation. `TextureTransform::affine` reads its slots from a sample and applies the established layered-center or model-shader-origin UV convention.

For a standing building, select a clip, evaluate its pose, choose the desired LOD,
project and deform its visible draws, and encode the sampled materials and geometry.
For vegetation, the same results can feed application-owned normalization, texture
sampling, crown shading, shadow projection, and simplification. Those operations
can use any suitable decoded image representation. Particle simulation, external
model resolution, world visibility selection, physics stepping, and application
baking recipes remain outside these evaluation methods.
