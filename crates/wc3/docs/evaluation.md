# Evaluating models without a renderer

A `Model<Universal>` can supply poses, editable draw geometry, and sampled surface
values without Bevy, image assets, or texture decoding. Native models can be widened
to `Universal` after import. Evaluation preserves semantic identities and native
surface descriptions; applications choose resource resolution, image libraries,
output encodings, and any deliberate material conversion.

## Preparing and sampling poses

Use `Model::prepare_pose` once to resolve parent indices, parent-before-child
traversal, native billboard conventions, and channel-presence masks. Its `PoseLayout`
is immutable and can be shared across independently sampled instances. Rebuild it
after changing node definitions or animation channel targets. Samples must belong
to the same model identities as the prepared layout. Animation compilation remains
separate: reuse
`AnimationProgram` when sampling many frames, and supply the resulting
`SampledAnimation` to the layout.

```rust
use wc3::model::{AnimationProgram, AnimationSample, ModelPose, PoseInputs};
# use wc3::model::{Model, Error};
# let model = Model::standalone("Example");
let layout = model.prepare_pose()?;
let animation = AnimationProgram::compile(&model);
let sample = animation.sample(AnimationSample::default())?;
let mut pose = ModelPose::default();
layout.evaluate_into(&sample, PoseInputs::default(), &mut pose)?;
# Ok::<(), Error>(())
```

`evaluate_into` retains output capacity and allocates nothing once its node and
visibility buffers are sized. `evaluate` is the allocating convenience method.
Both return full authored-space affine matrices and signed rotation/scale factors,
including mirrored or collapsed axes. Ordinary hierarchy retains shear. Native
billboards and selective inheritance use the same numerical evaluator as
`bevy-wc3`; existing approximations to native camera-facing behavior remain.
Missing camera input leaves camera-dependent corrections unapplied. Offline tools
requiring a camera-complete snapshot should require camera input themselves.

`PoseInputs` supplies an authored-space camera and complete local overrides keyed
by `NodeId`. Outputs follow `layout.nodes()` order; `layout.index(id)` resolves a
source identity. Visibility includes the ancestor chain. `PoseNode::sample` and
`PoseBehavior::evaluate` expose the same operations separately for renderers with
prepared traversal, local transitions, physics, or occurrence-time sampling.
`Transform::blend` interpolates local poses before hierarchy composition.
`cargo bench -p wc3 --bench evaluation` measures reusable pose evaluation and
draw projection without initializing a renderer.
`PropertySample` accepts both ordinary and retimed animation samples, so effect
and event times do not require selecting animation programmes again.

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
application's own storage. It changes only positions, normal covectors, and tangent
directions/handedness. UVs, colors, source mappings, and other application fields
remain untouched. Singular normal transforms produce zero normals. `bounds`
computes a finite box and conservative bounding sphere after deformation.
`Model::coordinate_transform` converts authored basis and units to an explicit
target; an identity matrix retains authored coordinates. Unspecified units use a
numerical scale of one meter per unit for conversion.

## Sampling surfaces and processing the result

Surface evaluation does not require texture pixels. `LayeredSurface::sampled`
freezes Warcraft III pass values, texture selections, opacity, emissive gain, and
Fresnel controls while retaining native render state and shared texture-transform
references. Sample those referenced transforms separately. `materials::m2::MaterialStep::sampled`
freezes model-shader color, opacity, input weights, frames, and inline transforms.
Both preserve shader selectors instead of approximating their meaning with PBR.
`MaterialSurface::sampled` applies these operations across a universal material program,
retaining its order and each step's rendering settings. WMO, M3, and external shader steps are cloned unchanged; their shader programmes are not
executed or flattened by this helper. Other native surface families remain available through their semantic definitions
and typed property sampling; these helpers do not interpret arbitrary shader
programmes or simulate procedural texture playback.

`MeshAppearance::compile` prepares geometry-owned tint and opacity controls. Rebuild
this preparation after changing geometry-animation definitions or channel targets.
Its sampled RGBA multiplier combines with surface color; `SurfaceColor` provides
the shared color/opacity/weight composition used by Bevy materials. Texture IDs
continue to resolve through `Model::texture`, retaining resource identities, color
interpretation, and samplers. `TextureTransform::sampled` freezes all components;
`affine` applies the established layered-center or model-shader-origin UV convention.

For a standing building, select a clip, evaluate its pose, choose the desired LOD,
project and deform its visible draws, and encode the sampled materials and geometry.
For vegetation, the same results can feed application-owned normalization, texture
sampling, crown shading, shadow projection, and simplification. Those operations
can use any suitable decoded image representation. Particle simulation, external
model resolution, world visibility selection, physics stepping, and application
baking recipes remain outside these evaluation methods.
