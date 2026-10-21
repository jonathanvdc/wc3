# External particle programmes

MDLX Popcorn features reference baked effects rather than ordinary sprite-emitter
parameters. The model loader imports the feature into `Model<Universal>`, resolves
its `.pkb` companion, and compiles a shared `wc3::model::scene::popcorn::EffectProgram`.
Each spawned model owns an independent bounded simulation. Supported billboard and connected-ribbon
outputs use the same compact sampled records, per-view sorting, Bevy lighting,
fog and output processing as M2 sprites.

This is a partial interpreter. Supported graphs can initialize and evolve particles,
sample curves and event streams, generate timestamped children, and retire them.
Typed routed payloads preserve birth-time values and native world-space frames
independently across child and fanout particles.
Unknown callbacks or unsupported graph contracts produce feature diagnostics;
loading a model does not imply that every external programme can execute.

## Resolve companions and images

`ResourceResolver::particle_effect_candidates` resolves the feature's resource.
By default, an authored `.pkfx` path selects a `.pkb` companion beside the model,
then at the asset root. The provider retains the asset source and owns archive
case handling and content identities. The renderer does not compile source PKFX.

External images have their own `ParticleTextureRequest`, including renderer index,
binding (`Diffuse` or `AlphaRemap`), resource, image role, transfer function and
sampler. They do not receive invented indices in the model's image table. Override
`ResourceResolver::particle_effect_texture_candidates` to supply archive paths or
replacements. Its owner is the resolved baked companion. Literal image paths and
BLP/DDS/PNG/TGA alternatives use the same per-load image cache as model materials;
equal paths with equal interpretations share prepared images.

Only enabled, supported renderer inputs request images. A disabled distortion
fallback does not make an otherwise ordinary diffuse effect fail loading. Missing
diffuse images use a white fallback and retain a resource diagnostic. Alpha-remap
LUTs request linear data sampling. Missing LUTs preserve diffuse alpha and report
the unresolved resource. Within one programme, cache handles by
`(request.renderer, request.binding)`;
one renderer can request multiple distinct images.

## Compile an in-memory model

Applications can prepare effects themselves and associate them with owner-local
Universal feature IDs. Native reading belongs at this resource boundary; model
compilation and live rendering continue to consume one Universal graph.

```rust
use bevy::prelude::*;
use bevy_wc3::{
    compile_model_with_options, CompileOptions, ParticleEffect, ParticleTextureBinding,
};
use wc3::model::{FeatureId, Model};
use wc3::model::scene::popcorn::EffectProgram;

# fn prepare(model: &Model<'_>, feature: FeatureId, program: EffectProgram,
#            diffuse: Handle<Image>, alpha_remap: Option<Handle<Image>>)
#            -> Result<(), bevy_wc3::ModelError> {
let effect = ParticleEffect::new(program, |request| match request.binding {
    ParticleTextureBinding::Diffuse => Some(diffuse.clone()),
    ParticleTextureBinding::AlphaRemap => alpha_remap.clone(),
});
let mut options = CompileOptions::default();
options.particle_effects.insert(feature, effect);
let compiled = compile_model_with_options(model, options, |_request| None)?;
# let _ = compiled;
# Ok(())
# }
```

For asynchronous loading, enumerate `ParticleEffect::texture_requests(&program)`,
load those image interpretations, then supply their handles to `ParticleEffect::new`.
Preparation diagnostics remain available before attaching the effect to a model.
The model compiler assigns their feature identity.
`effect.with_attribute("Game.TeamColor", Vec4::new(r, g, b, 1.0))?`
selects a typed exposed control for every new instance. Unknown names and
incompatible layouts fail during preparation; explicit values take precedence
over model control bindings. Other attributes retain their declared defaults.

## Playback and controls

Authored lifespan, emission rate and speed feed the programme's exposed multiplier
attributes directly. RGB and opacity feed `Game.ColorMultiplier`; scale and target
position controls bind when their typed declarations exist. Defaults remain in
place for other exposed attributes. Controls resolve by name during preparation,
then use numeric bindings during simulation.

The input transform preserves the native emitter quarter turn and the baked-to-MDLX
numeric unit ratio of one to one hundred. Simulation uses the native world axes;
the model's fixed Bevy basis converts extracted positions once, independently of
authored transform-mask filters. Native coordinate metadata is retained; its
independent game meaning remains unverified. Popcorn scaling routes average
emitter scale through `Game.Scale` and removes it from the input basis; without
that flag the basis retains its authored scale.

Camera-dependent programme simulation uses the camera explicitly selected by
`Playback::driving_camera`. Camera index zero refers to that choice, independently
of how many views render the model. The adapter converts its current world
position into the programme's native axes and one-to-one-hundred numeric units
once; emitter rotation, scale and translation do not transform the camera again.
Owned newborn programmes use the same selection at startup. Missing cameras and
unsupported indices yield unavailable positions rather than an invented world
origin. Their geometry is omitted while the instance remains alive. Later authored camera
queries can observe a new binding without restarting; initialization is not
replayed. Camera-dependent
simulation still has one result per model instance; use separate instances for
independent simulation cameras. Exact native camera behavior remains unverified.

`SceneContext` is a checked host capability, not collision support. Scene
intersection and unknown world-to-world operations still require additional
interpreters; no literal-zero collision response is supplied.

Procedural turbulence uses the checked core's original coherent curl-field
approximation. Prepared samplers share immutable octave controls; each model
instance owns its bounded effect-global oscillator cache. The sampler shares an
original seeded scalar/quadrature basis with eight discrete temporal frequency bands.
Field interpolation uses no trigonometry; a temporal cache miss evaluates eight
sine/cosine pairs, and interleaving more than four moving samplers can evict cached
phases. Native axes and units are not
converted a second time inside the sampler. Global scale changes both spatial
wavelength and velocity amplitude. Field samples consume no particle RNG.
The independent coordinate/channel hash and analytic gradients do not reproduce
the native stochastic field; rendering diagnostics mark this as an approximation.
At most 24 octaves execute per sample, with checked spatial/temporal precision
bounds. External PKVF and unsupported flow/divergence controls remain diagnosed.

The native PKB loader explicitly selects `MissingPayloadPolicy::Zero`: extracting
a valid declared root channel omitted by that birth returns typed zero words.
Supplied values and channel presence remain intact, and no event targets are
filtered or grouped from payload initialization controls. Unknown fields, wrong
layouts and stale or forged capabilities still fail. Rendering diagnostics report
the dedicated unverified missing-payload policy; these defaults are not established
native-client behavior. Applications needing strict missing-channel errors can
compile with the core's default `EffectCompileOptions` and pass that programme to
`ParticleEffect::new`.

Routed births retain their occurrence timestamps and are processed in chronological
order. A newborn evolves only for the remaining portion of its birth step. The
loader uses the last authored event timestamp as an event stream's duration, or
zero for an empty stream; applications compiling programmes themselves can choose
an explicit duration through `EffectCompileOptions`. Lifetime uses the canonical
`self.lifeRatio` and `self.invLife` fields: elapsed time advances the ratio, and a
ratio of at least one retires the particle, including a value written by its
programme. Authored death-step controls select terminal evolution. These timing,
duration and lifetime policies are implemented; exact native-game fidelity
remains unverified.

Playback pause retains both simulation and immutable GPU records. Seeking skips
the crossed simulation interval; sequence changes and backwards traversal restart
the graph. Initial graph execution waits until the first active frame.
After initialization, visibility, inherited activation and `AnimVisibilityGuide`
gate future renderer-bearing births while structural controllers continue. Living particles
keep aging without an emission backlog. Guide names match sequence-name substrings
without case sensitivity; `always=on/off` selects the default. Exact client
visibility reset and per-cycle restart conventions remain unverified.

## Spatial snapshots

Supported baked spatial declarations share a bounded store by full resource name
and named Float3 payload schema. Layer-local aliases and read/write flags do not
split that identity. Each simulation tick reads the previous snapshot and rebuilds
the next, so layer execution order does not expose newly inserted points early.
Lookup coordinates remain distinct from payload positions; a keyed lookup does
not become a nearest-world-particle search. Rank zero, payload slot zero and zero
query options are supported, using an inclusive radius and first-insertion ties.
Other query controls require an additional interpreter.

A first-tick miss or missing rebuilt key yields unavailable geometry while the
programme remains alive. No world-origin substitute or historical lookup replay
is introduced. Entry and draft omissions are available in
`ParticleProgramEmitter::omitted_spatial_writes`; exhausting the shared candidate
row-check allowance increments `omitted_spatial_queries`. These counters remain
separate from omitted births and executions. Configure the entry and row-check
capacities through `ParticleEffect::with_limits`. Native snapshot and tie fidelity
remain unverified.

## Rendering and budgets

The billboard adapter supports scalar size, rotation,
enabled state, HDR color, diffuse textures, rectangular atlas crossfade,
alpha-remap LUTs and native transparent blend choices. Facing is evaluated for
each particle and view from shared records:

| Mode | Geometry | Axis interpretation |
| --- | --- | --- |
| 0 | Screen-facing quad, or two authored axes | Both axes retain their lengths and may shear the quad. |
| 2 | Axial quad | Axis0 gives full longitudinal extent; size gives half width. |
| 3 | Spheroid quad | Axis0 gives the central extent; size adds view-facing padding at both ends. |
| 4 | Capsule, four triangles | Axis0 gives the constant-width core; view-facing pointed caps extend beyond its ends. |
| 5 | Planar quad | Axis0 and Axis1 define an orthogonal plane; Axis1 retains its length. |

`Axis0`/`Axis` and `Axis1`/`NormalAxis` custom streams bind the same typed vectors
as their native numeric selectors. Axial, spheroid and capsule facing use the particle's
direction from the eye and ignore scalar rotation. Planar rotation has the
opposite sign to screen rotation. Missing required axis streams use camera-facing quads;
degenerate axes have finite fallbacks. These edge policies and exact native
facing fidelity remain unverified. Capsules use shared immutable selectors with
separate core/cap normals and diagonal cap UVs; they are not stretched quads.
Their missing-axis fallback uses an ordinary quad while still ignoring rotation.

Size is a half extent for screen and planar quads. Nonpositive sizes omit geometry.
Atlas cursors use absolute
value and clamp to the final cell; nonfinite cursors select zero. Their fractional
part blends adjacent sampled RGBA frames before particle-color modulation.
`RotateTexture` rotates local UVs before effective U/V flips; `FlipUVs` enables
both axis flips. Cell placement follows these local operations. Authored atlas
blending metadata remains unverified; this adapter selects adjacent-frame crossfade.
Alpha remap replaces the modulated alpha with the LUT red channel while retaining
RGB. Billboard LUT V uses a deterministic hash of layer, pool slot and incarnation,
so sorting and swap removal do not change a living particle's lookup. This is an
explicit original random policy, with native fidelity unverified. Colors retain HDR
components and alpha above one. Authored soft intersections retain ordinary
billboard geometry with an unsupported-feature diagnostic. Unsupported facing
modes, two-dimensional size, general UV programmes and additional material
features produce renderer diagnostics. Distortion and mesh outputs have no adapter.

### Connected ribbons

Connected ribbons group enabled living particles by their exact parent slot and
incarnation, then connect neighbors in monotonic birth order. Pool reuse and
identical birth times do not reorder a strand. Disabled, expired and invalid
points are absent from connectivity; collapsed segments are omitted. Adjacent
segments share the same endpoint cross section rather than averaging a bend.
Ribbon mode 0 uses incoming segment direction and the current eye; mode 1 uses
`Axis0` crossed with that direction; mode 2 uses normalized `Axis0` as its side axis.
Axis modes require a typed `Axis0` stream. Degenerate cross sections use world +X.
These finite edge policies and native client fidelity remain unverified.

Ribbon size is half width, with endpoint color and width interpolated along each
segment. Default U runs from zero to one per segment. `CustomTextureU` instead
uses each endpoint's `TextureU`. The start point's `TextureID` selects the atlas
cell using absolute value, clamp and truncation; ribbons do not crossfade cells.
Canonical V is zero on the positive cross-section edge. `RotateTexture` and UV
flips precede atlas placement. With alpha remap, endpoint `Cursor` supplies the
interpolated LUT row, defaulting to zero when absent; it does not use the billboard
random row. General UV programmes and deformation corrections retain connected
geometry with explicit diagnostics. Ribbon segments use the shared immutable quad
and 112-byte records, including per-view geometry and sorting without changing
strand connectivity. They use ordinary effect lighting and fog, with no shadow or
ordinary mesh-prepass draw.

### Simulation budgets

`ParticleEffect::with_limits` selects `EffectLimits` for independent instances.
The default pool contains 1,024 particle slots, including controllers. Event queues,
generation handles, payload copying, simulation steps, execution work and predicted storage are
bounded separately. Omitted work advances time without a catch-up queue.
Programmes inside particle-owned models also consume the enclosing
[model-particle tree's shared emission-step allowance](prem.md#limits).
`ParticleProgramEmitter` exposes living particles (including controllers), billboard
`sprites` counts, `ribbon_segments`, `omitted_geometry`, simulation omissions,
suppressed births, skipped seconds and checked execution errors. Despawn removes
the simulation and its render entities.

The scalar evaluator uses shared local scratch and persistent particle storage;
steady-state execution performs no allocation within its configured capacities.
This establishes a storage contract, not native random-sequence fidelity or a
full-renderer performance comparison. The [core programme guide](../../../wc3/docs/popcorn.md)
describes supported execution policies and their remaining limits.
