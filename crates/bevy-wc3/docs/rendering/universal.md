# Semantic compiler and native resources

The renderer compiles `Model<'_, Universal>` directly into Bevy assets and an
independently animated hierarchy. Native MDLX, M2, WMO, M3, and Diablo adapters
supply semantic models; none requires export to a Warcraft III runtime version.
`ModelInstance` loads file-backed models onto an application entity.

## Compilation and ownership

Compilation borrows the graph, builds owned CPU meshes, and detaches animation
buffers from their source. `CompiledModel::prepare` inserts shared meshes
and inverse bind palettes into Bevy assets. `spawn_model` creates an
independently animated rig beneath an application-owned transform. The encoded
source and semantic graph can be released after compilation.

The image callback receives a `TextureRequest` with the texture identity and
complete semantic texture definition. Resource identity, sampler, color space, and
producer controls remain available to the application. Two textures can name the
same file while requiring different sampling or color interpretation. The native
loader retains independently labeled image variants for those cases. An unspecified
transfer function follows the material role: color samples use sRGB, while normal
and numeric data maps use linear images. A texture used in both roles receives
distinct handles. Explicit transfer-function declarations remain authoritative.
WMO clamp flags are
surface-local: `TextureRequest::sampler` composes enabled clamp controls with the
texture's declared sampler. Requests and labeled images distinguish those uses
without changing the semantic texture or duplicating profile authority.

Meshes are cached by geometry, subset, selected texture coordinates, and skin.
After complete attribute projection, skinning, and indexing, byte-identical meshes
also share storage across those keys. Authored tangent or other attribute
differences retain distinct meshes; matching hashes are checked for exact equality.
Coincident vertices are merged only when their complete compiled attributes agree,
preserving UV seams and face attributes. Prepared instances share mesh, animation,
and bind-palette definitions. Constant material handles are shared; animated
surfaces have private material assets. An unchanged sampled surface does not mark
its material asset modified.

## Geometry, coordinates, and playback

Draw subsets select their authored connectivity and primitive ranges. Triangle
strips retain winding, polygons are triangulated along their boundaries, and mixed
connectivity becomes consecutive compatible draws. Vertex, face, corner,
selected-connectivity and whole-mesh attributes retain their original
correspondence. Compilation traverses only the selected primitive range. Unsupported
attribute-pool bindings fail compilation rather than selecting arbitrary values.
Explicit inverse bind matrices and identity bindings retain their distinct
interpretations. GPU skinning supports up to eight active influences per vertex and
rejects larger distributions without truncation.

The model coordinate frame maps authored up and forward directions into Bevy's Y-up,
negative-Z-forward frame. Declared meters-per-unit scales the entire model frame.
Application transforms remain outside that conversion. The driving camera for
billboards and camera anchoring is explicitly selected on `Playback`.

Playback uses seconds. `wc3` evaluates typed curves, while `AnimationProgram`
indexes compiled channels by property destination. Clip aliases resolve their
programmes without losing the selected clip's loop or clamp policy. Independent
repeating shared clocks wrap by their authored periods; frozen shared clocks
retain time zero without event traversal. Warcraft III and M3 clip metadata
select the corresponding authoring timeline interval, preventing interpolation
through unrelated sequence keys. Playback uses the selected M3 clip’s authored controller list by default. Applications
can replace it with an explicit list, including an empty list; priority orders
contributions, with list order breaking ties.

`PoseOverrides` applies application-supplied local poses after authored
animation. Removing an override resumes normal sampling. Physics, gameplay
controllers, and solvers can use this boundary without replacing the renderer.

## Surfaces and scene integration

Surfaces extend Bevy's standard material and lighting pipeline. Authored blend
factors and independent depth-test/write controls specialize the GPU pipeline. A
default `ModelMaterialExtension` leaves a standalone `StandardMaterial`'s blend
and depth behavior intact; compiled model passes supply explicit overrides. Color,
opacity, texture choice, and coordinate transforms animate on the same semantic
clocks as node poses. Supported [M2 shader programmes](m2-materials.md) combine independent texture
inputs and generated environment coordinates before Bevy lighting; additive
contributions enter emissive. Unsupported
shader selectors and composition rules produce material diagnostics. DefaultUnit HD
layers retain diffuse, RG normal, ORM, emissive, team-color, and reflection roles,
including animated bindings and Fresnel controls. The [Reforged material
guide](reforged-materials.md) describes the shared PBR interpretation and its
reflection approximation.

`CompileOptions` selects typed material appearances. Draw-based looks and
Diablo slot alternatives retain their own identities. An ambiguous Diablo default
requires an explicit selection; compilation does not choose the first material or
variant.

Portable, ambient/diffuse, WMO, and M3 lights expose ordinary Bevy scene lights and
`ModelLight` metadata. Root `ModelLightSettings` controls conversion scales
and suppression. Common diffuse color, intensity, range, and supported shadow flags
animate with the instance. Ambient terms authored on individual light features
remain metadata; the renderer does not overwrite scene ambient lighting. Native WMO cone and orientation conventions,
cookies, flicker, and regional light-set activation remain uninterpreted. M3's
separate specular response, decay law, and light LOD controls also remain available
without an exact lighting equivalent.

Attachment points preserve identity namespaces, resource identity, and activation.
`PlacementAppearance` retains complete WMO placement appearance. Its
interior direct illumination, linked-light and region controls remain application
hooks; they are not multiplied into child albedo. Child models start their first
clip and restart on activation, parent clip changes, or backwards parent traversal.

Authored cameras can be sampled on request or bound to an application-owned
perspective `Camera3d` through `CameraBinding`. Bindings select the model
root and camera `FeatureId`, and update lens and view before Bevy's camera update.
They sample the current playback and pose overrides and resolve ordinary application
transform hierarchies directly, avoiding a previous-frame rig pose or root
transform. Activation, viewport, and render target remain application-owned.
Animated rig nodes and other bound cameras in the root or camera-parent chain are
rejected, as are parents requiring a sheared local camera transform; invalid
bindings retain their previous view. Camera feature IDs change when a source is
decoded again, so bindings must be retargeted after asset replacement.

`ModelNode`, `ModelDraw`, and feature metadata preserve semantic source
identities. `PreparedModel::features` also exposes integration points whose
visual interpreter is unavailable. These generic hooks retain IDs and kinds rather
than complete owned unsupported payloads; inspect the graph during compilation when
those definitions are required. Rendering diagnostics describe missing behavior and
approximations; successful decoding does not imply complete support for a model's
authored shader or effect programmes.

## Particle simulation

Staged atlas sprites use the shared GPU transport: immutable birth records carry
motion and appearance inputs, and shaders construct head/tail geometry. Scalar
emission controls sample at birth times. Keyed bursts track crossed keys and clock
cycles rather than repeating a burst on every frame. Gravity and fixed-plane
geometry follow the authored model axes after coordinate conversion.

M2 age-driven sprites retain independent normalized-age color, opacity, scale, and
atlas curves, plus authored twinkle and per-particle secondary texture motion.
Native packed gravity is decoded to animated vectors before rendering. Modern
head/tail flags and legacy geometry bytes resolve to an explicit semantic choice. Their current transport rebuilds one reusable CPU mesh per emitter as
its population or view changes. Motion evaluates gravity, wind, and linear drag
analytically. Local particles follow the current emitter transform; world particles
retain their birth frame with optional authored emitter-follow translation.
M2 attached particles use their current emitter frame; detached gravity does not
rotate or scale with the animated bone. Declared model axes and units still apply. Paused, unchanged frames avoid mesh uploads.
Living-particle, per-update birth, and long-update budgets bound CPU work;
`SpriteEmitter` exposes omitted work and current population.

M3 standard billboards interpret common distributions, velocity selectors,
normalized three-stage appearance, keyed squirt counts and flipbook timing through
the CPU transport.
The [sprite guide](pre2.md#m3-standard-billboard-subset) distinguishes this subset
from unsupported programmed forces, collisions, secondary effects, and geometry.

Ribbon sections use the shared GPU transport and independent authored material
passes. Section appearance samples its birth clock; retired or omitted sections
break connectivity instead of joining unrelated trail segments.

Authored emitter hierarchy and visibility evaluate at each occurrence time,
including nonlinear parent motion. Application root motion interpolates the two
observed frame transforms. Continuous emission uses bounded 60 Hz midpoint
integration; this is a numerical approximation of animated rates. Keyed bursts
retain their discrete clock crossings. Spline/bone generators, advanced M2 texture programmes and unresolved modulation
gain, and several programmed effect families remain unsupported and produce
diagnostics.

## Static worlds

Prepared models expose world groups, portals, placement sets, regions, liquids, and
authored fog and ambient definitions. `WorldSelection` can hide groups or
select placement sets. The default is conservative; an explicit `ReachableFrom`
policy follows portal connectivity. `WorldPortalPolicy::Clipped(PortalView)` adds
opt-in portal-window traversal. Applications supply the containing group, eye and
inward-facing frustum planes in semantic model coordinates, and update that view
when their chosen camera moves. The renderer clips convex portal polygons against
the current window and narrows each neighboring view through the visible edges.
Different paths into the same room retain their separate windows; cycles are
bounded without discarding other entrance paths.

Exterior and unclassified groups remain visible. Unknown or exterior starting
groups, unresolved portal controls, malformed polygons and traversal work limits
fall back conservatively. Native portal-side conventions, supplemental plane
adjustments and occluder planes are not interpreted; this is geometric window
culling rather than a complete native visibility implementation. Applications own
camera choice and containment rather than relying on guessed room bounds. Visibility
gates intersect these selections with authored animation, material opacity and LOD.

Liquid grids preserve holes, native spacing, and absolute vertex heights. An
authored material is interpreted through the standard material compiler. A liquid
without a resolved material exposes its mesh and `WorldLiquidSurface`
identity for an application material provider; the renderer does not infer a shader
from an external liquid-type database. Environment sampling exposes regional fog and
ambient weights without changing global Bevy lighting or fog. [WMO surfaces](wmo-materials.md)
interpret baked vertex lighting, draw lighting classes and root/region ambient
contributions independently of those application-owned scene resources. Native regional
lighting fidelity remains unverified.

## Native resources

`ModelLoader` imports native models and their required companions before
direct compilation. Applications implement `ResourceResolver` to map
file-data IDs, namespaced resources, or paths into registered Bevy asset sources.
The default resolver handles relative paths and tries literal, BLP, DDS, PNG, and
TGA candidates beside the model before the asset root. Named asset sources and
authored filename case remain intact. `texture_resource` selects replacements by
semantic identity, source-list index, and sample role; `texture_candidates`
customizes lookup. `compile_options` selects authored material appearances. Required
companion failures stop loading; missing optional images remain inspectable on
`ModelAsset`.

M2 bundles resolve bone animations and attachment tables through their external
skeleton chain. Attachment activation uses the owning skeleton's animation stream;
ordinary root tracks use the inherited sequence-storage flags. Nonempty deferred
tracks require that context before validation or writing. Empty equipment
sequences do not manufacture sibling animation-file dependencies, while explicit
animation-file declarations remain required.

`ResourceResolver::import_options` selects native conversion policy.
Its default is strict preservation; accepting loss is an explicit provider choice,
and retained conversion diagnostics identify dropped or approximated records.
A permissive policy does not invent required resources or repair malformed data.

WMO layout provenance must be supplied explicitly because a file version does not always identify a dialect. Diablo IV layouts and external
payload identities likewise require provider knowledge.

The shared [viewer and capture tool](../tools.md) uses the same Universal compiler
and runtime as application loading.

## Current limits

Native loading preserves native-family vocabulary rather than promising the same
shader coverage for every family. MDLX common layers and effects, M2 common surfaces
and age-driven sprites, WMO groups/layered surfaces/vertex lighting/liquid grids, M3 common
Standard layers and composite passes, lights and selected programmes, and known Diablo roles each have separate
interpretation limits. Native loading, generic geometry, and basic surface
interpretation are separate capabilities from full game-specific rendering. M2 illumination/constant programmes and the modern six-binding guild layout, specialized WMO terrain/window/LOD and extended
combiners, specialized M3 families and advanced layer controls, external
Diablo shaders, and external effect programmes require additional interpreters.
These gaps appear in model diagnostics.

Skinned mesh bounds conservatively enclose positive-weight joint contributions,
including eight influences. Changed contributing transforms and source assets
refresh bounds; hidden geometry defers work until reveal. Singular or unavailable
bounds temporarily disable frustum culling rather than hiding uncertain geometry. Automatic authored LOD
selection and local-pose transitions are implemented; simplification, per-view
skeletal poses, and exact native lighting remain outside that scope. Implemented
native billboard and shader interpretations can still differ from the originating
game.
