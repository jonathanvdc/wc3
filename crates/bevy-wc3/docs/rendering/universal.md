# Direct semantic rendering

The direct renderer compiles `Model<'_, Universal>` without exporting it to a
native model format. Use this path for models whose geometry, binding, animation,
or material vocabulary cannot be represented by the Warcraft III runtime.
`UniversalModelInstance` loads these assets onto an application entity. The
separate `Wc3ModelInstance` runtime remains available; its feature support is
described by the other rendering guides.

## Compilation and ownership

Compilation borrows the graph, builds owned CPU meshes, and detaches animation
buffers from their source. `CompiledUniversalModel::prepare` inserts shared
meshes and inverse bind palettes into Bevy assets. `spawn_universal_model`
creates an independently animated rig beneath an application-owned transform.
The encoded source and semantic graph can be released after compilation.

The image callback receives a `TextureRequest` with the texture identity and
complete semantic texture definition. Resource identity, sampler, color space,
and producer controls remain available to the application. Two textures can
name the same file while requiring different sampling or color interpretation.
The native loader retains independently labeled image variants for those cases.
An unspecified transfer function follows the material role: color samples use
sRGB, while normal and numeric data maps use linear images. A texture used in
both roles receives distinct handles. Explicit transfer-function declarations
remain authoritative.

Meshes are cached by geometry, subset, selected texture coordinates, and skin.
Coincident vertices are merged only when their complete compiled attributes
agree, preserving UV seams and face attributes. Prepared instances share mesh,
animation, and bind-palette definitions. Constant material handles are shared;
animated surfaces have private material assets. An unchanged sampled surface
does not mark its material asset modified.

## Geometry, coordinates, and playback

Draw subsets select their authored connectivity and primitive ranges. Triangle
strips retain winding, polygons are triangulated along their boundaries, and
mixed connectivity becomes consecutive compatible draws. Vertex, face, corner,
selected-connectivity and whole-mesh attributes retain their original
correspondence. Compilation traverses only the selected primitive range.
Unsupported attribute-pool bindings fail compilation rather than
selecting arbitrary values. Explicit inverse bind matrices and identity bindings
retain their distinct interpretations. GPU skinning supports up to eight active
influences per vertex and rejects larger distributions without truncation.

The model coordinate frame maps authored up and forward directions into Bevy's
Y-up, negative-Z-forward frame. Declared meters-per-unit scales the entire model
frame. Application transforms remain outside that conversion. The driving camera
for billboards and camera anchoring is explicitly selected on `UniversalPlayback`.

Playback uses seconds. `wc3` evaluates typed curves, while `AnimationProgram`
indexes compiled channels by property destination. Clip aliases resolve their
programmes without losing the selected clip's loop or clamp policy. Independent
shared clocks wrap by their authored periods. Warcraft III and M3 clip metadata
select the corresponding authoring timeline interval, preventing interpolation
through unrelated sequence keys. Applications explicitly choose controllers;
priority orders their contributions, with caller order breaking ties.

`UniversalPoseOverrides` applies application-supplied local poses after authored
animation. Removing an override resumes normal sampling. Physics, gameplay
controllers, and solvers can use this boundary without replacing the renderer.

## Surfaces and scene integration

Surfaces extend Bevy's standard material and lighting pipeline. Authored blend
factors and independent depth-test/write controls specialize the GPU pipeline.
Color, opacity, texture choice, and coordinate transforms animate on the same
semantic clocks as node poses. Supported legacy M2 multiplication combinations
sample independent primary and secondary texture coordinates before lighting.
Unsupported shader selectors and composition rules produce material diagnostics.

Portable and ambient/diffuse model lights expose ordinary Bevy scene lights and
`UniversalLight` metadata. The application can suppress individual lights and
configure conversion scales with root `Wc3LightSettings`. Authored ambient terms
remain available as metadata; the renderer does not overwrite scene ambient
lighting. Attachment points preserve their identity namespace, resource identity,
and activation. Authored cameras are sampled on request, leaving ownership of
active views with the application.

`UniversalNode`, `UniversalDraw`, and feature metadata preserve semantic source
identities. `PreparedUniversalModel::features` also exposes integration points
whose visual interpreter is unavailable. Rendering diagnostics describe missing
behavior and approximations; successful decoding does not imply complete support
for a model's authored shader or effect programmes.

## Particle simulation

Staged atlas sprites use the shared GPU transport: immutable birth records carry
motion and appearance inputs, and shaders construct head/tail geometry. Scalar
emission controls sample at birth times. Keyed bursts track crossed keys and
clock cycles rather than repeating a burst on every frame. Gravity and fixed-plane
geometry follow the authored model axes after coordinate conversion.

M2 age-driven sprites retain independent normalized-age color, opacity, scale,
and atlas curves. Their current transport rebuilds one reusable CPU mesh per
emitter as its population or view changes. Motion evaluates gravity, wind, and
linear drag analytically. Local particles follow the current emitter transform;
world particles retain their birth frame. Paused, unchanged frames avoid mesh
uploads. Living-particle, per-update birth, and long-update budgets bound CPU work;
`UniversalParticleEmitter` exposes omitted work and current population.

Birth transforms currently interpolate frame endpoints. Arbitrary nonlinear
motion between those endpoints is an approximation. Model/child-model particle
instantiation, spline generators, packed-gravity interpretations, and several
programmed effect families remain unsupported and produce diagnostics.

## Native resources and captures

`UniversalModelLoader` imports native models and their required companions before
direct compilation. Applications implement `UniversalResourceResolver` to map
file-data IDs, namespaced resources, or paths into registered Bevy asset sources.
The default resolver handles relative paths. Required companion failures stop
loading; missing optional images remain inspectable on `UniversalModelAsset`.
WMO layout provenance must be supplied explicitly because a file version does
not always identify a dialect. Diablo IV layouts and external payload identities
likewise require provider knowledge.

The existing capture example accepts `--universal` and uses the same fixed-step
simulation and zero-delta readback protocol as the Warcraft III path. Specify
`--eye` and `--target` in Bevy coordinates. Universal capture currently requires
those explicit camera arguments and rejects unsupported capture options instead
of ignoring them.

## Current limits

Support differs by semantic family. Native loading, generic
geometry, and basic surface interpretation are separate capabilities from full
game-specific rendering. Complex M2 combiners, generated environment coordinates,
WMO regional illumination and visibility, programmed M3 surfaces, and external
Diablo shaders require additional interpreters. Direct ribbons, resource
particles, and external effect programmes also require additional interpreters. These gaps appear in model diagnostics.

Skin culling currently uses a conservative no-frustum-culling policy. Automatic
LOD selection, transition blending, event dispatch, and external attachment
instantiation have not yet moved to the direct runtime. Billboard conventions
and native shader fidelity require rendered validation beyond compilation.
