# Baked Popcorn effects

`model::formats::pkb` reads the native envelopes and compiled instruction streams
of version 2.5.1 PKB effects. Use it to inspect an external effect referenced by an
MDLX Popcorn feature. Native inspection borrows its source; checked graph lowering
produces an owned, Bevy-independent programme with bounded reusable simulation.
Graphics integration resolves the compiled renderer descriptors and their resources.

## Inspect a document

The envelope borrows strings and object field bytes from the source. Type entries,
object order, flags, and uninterpreted payloads remain available, and `as_bytes()`
returns the original source without reconstructing unknown fields.

```rust
use wc3::model::formats::pkb::Document;

# fn inspect(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
let document = Document::read(bytes)?;
for object in document.objects() {
    let native_type = &document.types()[object.type_index as usize];
    let name = document.strings()[native_type.name as usize];
    println!("{}: {} fields", String::from_utf8_lossy(name), object.field_count);
}
# Ok(())
# }
```

Native object links are one-based and nullable; `document.object(link)` checks
them independently of the zero-based type and string tables. Strings retain their
original bytes, including empty or non-UTF-8 values. Unknown field widths require
a schema, so the envelope does not guess boundaries within an object payload.

## Inspect compiled programmes

`compiler_blobs()` discovers typed compiler-cache views in native object order.
Each view retains its stage identifier, external and function declaration links,
entrypoint link, and borrowed script bytes. A typed view rejects unknown cache
fields while the original document remains intact.

`CompilerBlob::script()` checks declared constant and instruction regions and
decodes the supported IR instruction family. Instructions retain byte positions,
packed swizzles, argument controls, signed receiver slots, and raw register words.
Register bank and scope bytes remain separate from external declaration type IDs.
Constant slots occupy 32 bytes; `Script::constant()` reads the first four component
words without discarding the remaining uninterpreted native words.

Unsupported envelope versions and instruction encodings return errors. The native
views also expose external declarations, function declarations,
argument layouts and entrypoint ranges. Required binding fields fail explicitly
when absent; supported sparse defaults remain separate from unknown native data.
Declaration type identifiers and register-bank bytes are different domains.

## Inspect graph routes

`Document::effects()` discovers effect roots; `layer_graph(effect.graph)` resolves
entry, layer and event objects into checked graph slots. `LayerIndex` and
`EventIndex` distinguish these positions from one-based native object links.
The reader checks route bounds and declared event ownership before returning a
graph. Anonymous entry zero remains distinct from a named `Root` entry, which
may select another event.

Event parent omission denotes layer zero; explicit minus one denotes no parent.
The authored coordinate tuple and broadcast declarations remain native data.
Their presence does not imply coordinate interpretation or broadcast execution.
Graph inspection does not start layers or emit particles.

`layer_cache()` resolves a layer's field and event declarations while its
constant/range arrays borrow unaligned native bytes through `PackedArray`.
Field storage indices remain separate from VM register addresses. Sparse zero
indices and explicit minus-one sentinels retain their distinct meanings, and
unknown field types remain data rather than guessed evaluator layouts. Event
payload elements retain native type, footprint and range controls; their total
footprint must match the declared event size.

## Inspect renderer and sampler bindings

`renderer()` resolves ordered `RendererInput` and `RendererProperty` declarations.
Input storage indices address the owning layer's fields, independently of script
registers. Custom input names and native types remain explicit. Renderer class,
draw order, input semantic, property type and numeric words have native sparse
zero defaults. Property numeric values preserve component bits, including boolean
all-ones and nonfinite float encodings. Missing properties are not synthesized:
an absent `EnableRendering` differs from a present toggle with zero numeric data.

`named_sampler()` preserves each binding's symbol, native type, usage and checked
object link. `shape_sampler()` exposes primitive parameters and preserves sparse
omissions, raw authoring coordinates, localized names and editor metadata. Shape
defaults and coordinate conversion belong to checked lowering.
`turbulence_sampler()` retains procedural/external controls, numeric integer seeds,
localized metadata and sparse omissions. Defaults are applied only during checked
preparation; external vector-field metadata remains inspectable without execution.
`spatial_layer()` separates the full shared resource name from its local binding
alias and retains named payload types and read/write controls. `curve_sampler()` and
`event_stream_sampler()` expose borrowed native arrays
without applying simulation policy. Curve type, interpolation and quality omissions
remain `None`; the native view does not infer scalar width from sparse metadata.
Localized curve names and editor coordinates remain inspection data. Scheduling,
curve interpolation and unsupported-family diagnostics belong to checked lowering.

## Simulate a checked effect

`effect_roots()` supplies native effect links for `compile_effect()`. Lowering
resolves named fields, attributes, samplers, callbacks and event routes once.
The resulting `EffectProgram` owns immutable data and can be shared among many
instances after the source document is dropped. Initialization and the selected
physics, fixed or varying evolution stage remain separate; alternative stages do
not all execute for the same particle.
An extra native stage with an unrecognized identifier is accepted only when its
complete script and resolved external, function and entry-point metadata exactly
match a supported alternative. It contributes no additional invocation and is
reported as `EffectLimitation::RedundantProgramme`. This admits redundant native
data without assigning the unknown identifier a lifecycle meaning; that meaning
and native selection fidelity remain unverified. Other unknown stages reject.

```rust
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{EffectLimits, SceneInputs, Value};

# fn simulate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
let document = Document::read(bytes)?;
let (link, _) = document.effect_roots().next().ok_or("missing effect")??;
let program = document.compile_effect(link, &EffectCompileOptions::default())?;
let mut instance = program.instantiate(EffectLimits::default(), 42)?;
instance.start(&program, program.default_entry(), SceneInputs::default())?;
instance.advance(&program, 1.0 / 60.0, SceneInputs::default())?;
for renderer in program.renderers() {
    let fields: Vec<_> = renderer.inputs.iter().map(|input| input.field).collect();
    let mut values = vec![Value::from(0.0f32); fields.len()];
    for particle in instance.particles(renderer.layer) {
        particle.sample_fields(&fields, &mut values)?;
        // Pair renderer.inputs with values to extract typed canonical streams.
    }
}
instance.stop();
# Ok(())
# }
```

A pool defaults to 1,024 slots, including controllers and generators. Configure
`EffectLimits` before instantiation to bound population, pending births, generated
occurrences, executions, steps and predicted storage. The instance preallocates
field component planes, persistent particle registers, event storage and shared
VM scratch. Starting, advancing, retiring and reusing particles allocate nothing.
Checked effects compact referenced VM registers across their selected stages, so
sparse native addresses do not determine per-particle storage. Native script views
retain authored addresses, and execution failures retain instruction byte offsets.
Borrowed `ParticleView` values expose canonical fields independently of temporary
VM registers, and particle IDs include a generation to distinguish recycled slots.
Instances require the same programme identity when starting or advancing.

Selected evolution programmes can use full 16-particle batches for a checked
arithmetic and read-only curve subset. Initialization, short populations and
other operations retain scalar execution. The immutable layer owns its prepared
batch plan; each instance preallocates optional batch scratch within its storage
allowance, falling back to scalar execution if that allowance cannot fit it.
Batches share scalar arithmetic and curve evaluation, preserve canonical commit
order and replay failed speculative work through the scalar error path.

Batching trades preparation, scratch storage and a small per-step dispatch cost
for dense throughput. Measure both sparse and dense populations with
`cargo bench -p wc3 --bench particle_runtime`; its original fixtures and untimed
field oracle make those workloads reproducible without acquired assets.

`SceneInputs` supplies the world transform and a typed `SceneContext` snapshot;
elapsed effect time belongs to the instance. `SceneContext::with_camera` accepts
a finite position in the same world axes and numeric units as that transform.
The checked `view.position` callback uses camera index zero. An absent camera or
unsupported index returns positive infinity in all three components, allowing
authored validity tests and renderer adapters to omit unavailable geometry.
Supported coordinate transforms preserve that exact unavailable value after
checking their frame and mask. NaN, mixed infinities and transform overflow still
fail explicitly. Native scene capabilities remain opaque binding identities,
independent of numeric time, pointers and particle/random contexts.

Authored attributes have typed defaults and resolved IDs; set overrides
before `start()` when initialization must observe them. Float defaults use native
F4 storage, while boolean defaults use separate I4 storage. An omitted boolean
default is false; supported explicit true encodings normalize to all-ones. `set_emitting(false)`
reversibly suppresses renderer-bearing births while structural controllers and
accumulators advance, so hidden intervals do not accumulate a later burst.
`stop()` disables further generation and retires pure spawners while existing
visible particles and authored controllers finish. An authored positive-infinite
inverse lifetime denotes an instantaneous particle: initialization and its routes
run normally, zero-delta warmup leaves it pending, and its first positive interval
retires it after any requested terminal evolution. This is an explicit scheduling
policy rather than a `0 * infinity` NaN dependency. NaN, negative-infinite and
negative inverse lifetimes remain errors. Restarting clears the live
population and simulation counters while retaining allocated storage.

A condition trigger emits zero or one occurrence on each invocation, without an
implicit edge latch. Finite interval fractions clamp to 0–1; nonfinite fractions
fail explicitly. False triggers still return valid empty batch capabilities.

Events retain subframe birth times, fan-out ownership and invocation-specific
batch/draft capabilities. Initialization runs once, and a newborn evolves only
for the remaining portion of its birth interval. Budget omissions increment
`EffectStats`; omitted work never enters a deferred catch-up queue. If step work
is omitted, elapsed time and lifetime still advance so particles can retire.
Runtime errors stop advancement until the instance is restarted and leave earlier
writes observable.

The implemented callback subset covers camera position, effect age/running state, event-stream
and rate generation, condition-triggered events, typed payload construction, routing and extraction, retirement, random intervals
and isotropic random vectors,
point transforms with masks 11/17/19, direction transforms with mask 18, checked
inverse point/direction transforms with masks 67/66, RGB/HSV
conversion, quaternion orientation and rotation, emitter axes, typed curve
sampling, primitive `samplePosition` callbacks, and checked procedural turbulence sampling. It
executes the complete acquired HD LichAttack graph, including invisible
controllers and its routed child layer. Shared spatial resources support named
Float3 insertion and rank-zero nearest lookup. Scene collision, broadcasts,
other spatial query options, unsupported payload
semantics, other transform masks, unknown callbacks and unresolved sampler
families fail explicitly. When a native curve omits dimensionality, lowering
resolves it from the return registers of its bound `sample` calls. All uses across
owning layers and programme stages must agree, and values/tangents must fit that width. The borrowed native
view still reports `None`; unused sparse curves remain unresolved.

## Prepare native execution

Ordinary effect compilation retains the Rust evaluator, including its eligible
16-particle batches. Enable the optional `wc3/popcorn-jit` feature and call
`program.with_execution(ExecutionOptions::default())` before instantiation to
prepare scalar native code and eligible 16-particle SIMD batches, each lowered
as four groups of four lanes. Cloned programme handles reuse a live preparation
for exactly equal execution options. Returning the current options does no work;
a sibling handle shares the prepared plan without allocating. The graph-local
index owns weak references, so executable storage is released after its last
wrapper or instance disappears. Independently compiled graphs keep separate
identities and caches. Instances share executable allocations and retain their
own particle storage.
The crate requires Rust 1.97. The feature supports native execution on Unix and Windows
with a Cranelift-supported x86-64, AArch64, RISC-V64 or s390x host. Other builds
retain the Rust evaluator. SIMD lowering currently supports x86-64 and AArch64.

`ExecutionOptions` separates backend preference from `EffectLimits` and native
format interpretation. Choose `ExecutionPreference::Interpreter` to skip native
preparation. Successful `ExecutionPreference::ScalarJit` preparation selects
scalar native execution without packet dispatch, providing a direct comparison
route; unavailable native execution retains the Rust fallback.
`NumericPolicy::Strict` preserves evaluator arithmetic policies;
`Graphics` permits reassociation and equivalent analytic trajectories while
retaining ordered callbacks, lifecycle, budgets and checked failure boundaries.
A policy grants permission for transformations; it does not require them.

Inspect `execution_reports()` for each layer's initialization and selected
evolution route, including preparation fallback reasons. An instance captures
its plan at allocation, so preparing another handle for the same graph does not
switch existing instances. Programme and attribute identities remain shared.
`execution_stats()` exposes scalar invocation attempts, SIMD batch entries and
committed native/Rust packet lanes separately from semantic simulation counters.
These execution counters remain cumulative across `start()` calls.
Automatic execution retains the checked Rust packet path when a native tile is
unavailable. Reports establish preparation; counters establish dispatch,
and neither alone establishes a performance improvement.

`specialization_report(layer)` and `specialization_reports()` explain semantic
eligibility without allocating or repeating analysis. The pure-packet verdict
belongs to the shared graph; the immutable-expression verdict belongs to the
wrapper's execution policy. A rejection includes a typed reason and, where
applicable, the original byte offset and canonical field. External-binding
rejections retain both the programme-local binding ordinal and a
`SpecializationExternal` semantic kind, such as SceneContext, Attribute or Sampler;
the ordinal alone is not a global capability identifier. The first rejection
follows deterministic analysis order. `NotExamined` means that this policy did
not request the analysis; `NotApplicable` means there is no selected evolution.
Eligibility does not promise a compiled SIMD tile, enough living particles to
batch, successful runtime authorization, or graphics-adapter support. Use
`execution_reports()` for platform and compilation fallbacks and execution
counters for actual dispatch; runtime faults retain their ordinary error path.

The [performance guide](popcorn-performance.md) provides paired comparisons and
recorded measurements against the packet-enabled Rust evaluator.

Generated scalar arithmetic uses the checked typed instruction representation.
Eligible immutable curves embed their interpolation directly within bounded
preparation work; other curve calls retain checked services.
Operations that need checked host services retain their original ordered call
boundary. A scalar invocation failure leaves earlier writes visible and does not
replay the invocation. Failed speculative SIMD batches discard staged output and
replay their invocations through the scalar path. Native compilation is bounded;
unsupported platforms, disabled features and preparation failures select Rust execution before simulation.

## Prepare immutable appearance

`ExecutionPreference::ImmutableAge` with `NumericPolicy::Graphics` prepares
closed appearance expressions for eligible renderer-bearing leaf layers. It is
portable and does not require the optional native-code feature. Controllers,
initialization, randomness, event routing and lifetime remain CPU-owned; other
layers keep their ordinary evaluator. Strict policy retains ordinary execution
with a typed fallback reason.

CPU projections and graphics adapters consume the same bounded typed expression
DAG, with each backend admitting its supported inputs. Normalization reuses
identical typed expressions and folds finite literal arithmetic while retaining
checked curve calls, original preparation bounds and finite capture guards. It does not remove source authorization obligations.

The supported subset closes initialized fields, arithmetic, immutable curves driven
by appearance life, and certified affine phase/clock recurrences. A scalar or
vector accumulator can add or subtract an immutable rate times the difference
between the current time and its paired stored clock. Rates may be constants or
pure expressions of initialized invariant fields. Birth admission checks finite
capture-only intermediates; changing rates, cross-feedback and observers of an
old accumulator retain ordinary execution.
Unproven live inputs or potentially failing curve coordinates retain ordinary
execution. This is an optimization boundary, not an additional native feature
limitation. `immutable_recipe()` exposes read-only typed nodes, capture bindings,
output fields and curve data for a graphics adapter without exposing native VM
registers or mutable programme state.

A second affine form updates an accumulator by an invariant rate times the
invocation's Delta. Its `IntegratedDelta` input is the sum of the actual
VM-visible f32 cursor intervals over authorized evolutions since birth capture,
accumulated in f64 and converted to f32 when evaluated. It is independent of
Time, Life and caller FPS: internal subdivisions and a newborn's partial first
interval contribute their own rounded intervals, while omitted work contributes
nothing. Multiple Delta reads count one interval; zero-delta warmup does not
invoke evolution. `ImmutableParticleView::sample().integrated_delta` exposes the
committed sum without changing it. Arbitrary Delta outputs and uncertified Delta
uses retain ordinary execution with `UncertifiedDelta`; this is not a general
constant-acceleration integrator.

Graphics policy permits reassociation of these repeated source updates into the
closed expression, so source f32 accumulation need not remain bit-identical.
Strict retains ordinary execution. Prospective nonfinite interval/sum admission
restores the previous authorized outputs before executing the current source
invocation, preserving its checked failure and write prefix. The sum shares
budgeted per-slot authorization storage within the previous 64-bit footprint
bound; storage prediction still uses the actual target layout. No sampling call
allocates or advances that sum.

Canonical Life and retirement still use the CPU's accumulated lifetime. The
Graphics appearance input instead uses initialized life plus initialized inverse
life times authorized elapsed time, with explicit f32 time anchors and an upper
clamp at one. Paired-clock expressions use their initialized accumulator and
clock; Delta expressions use their authorized interval sum. These analytic
expressions can differ slightly from repeated f32 accumulation.
`ParticleView::field()` returns their evaluated canonical outputs without heap
allocation; reading Life itself still returns the authoritative CPU value.
Each individual virtual-output query evaluates the complete bounded recipe.
Use `ParticleView::sample_fields(fields, outputs)` to read several typed fields
with one shared evaluation, preserving caller order and duplicate requests.
It accepts ordinary canonical fields and immutable outputs together, validates
the request before writing results, and allocates nothing while sampling.

When only part of the recipe is needed, prepare `recipe.project(fields)` once.
The resulting `ImmutableProjection` retains only those virtual outputs and their
shared dependency/capture closure. Allocate reusable storage with `new_scratch()`,
then call `evaluate(view, scratch, outputs)` for each authorized immutable view.
The projection owns its exact source-plan identity and rejects views from a
different prepared recipe, even when it belongs to the same graph. Equal-policy
wrappers sharing a live execution plan also share compatible recipe identity.
`node_count()`, `capture_count()` and `output_count()` expose the compact cost.
Requests preserve order and duplicates; an empty projection is valid. Only virtual
output fields can be projected. Use bulk sampling for mixed raw and virtual fields.
Output length, scratch capacity and recipe identity are checked before results
change; checked evaluation failures also leave the output slice unchanged.

A graphics adapter can use `projection.recipe(source_recipe)` to inspect that
same compact dependency/capture closure, and `projection.view(source_view)` to
read its remapped captures. These borrowed views retain the source's sampler
resources, particle identity, birth anchor and authorized observation. They do
not create a new simulation or authorize an earlier or later sample. `evaluate`
accepts either the original source view or that projection's own compact view;
other compact projections require explicit view conversion along their source
relationship. Source execution guards still cover the complete selected evolution,
including computations that a renderer's projection does not consume.

Choose this policy using both advance and extraction workloads. The
[performance guide](popcorn-performance.md#immutable-cpu-appearance-2026-10-22)
records query costs with their measured method; a new bulk or projected query
needs its own comparison rather than inheriting individual-query results.
`immutable_sample()` borrows raw initialized captures and the authorized sample
without evaluating appearance fields, allowing a renderer to upload births
rather than resample every output each tick.

A newborn's initializer remains visible until its first authorized evolution.
`immutable_time()` returns a common clock only when all eligible living particles
have an observation at that time. An omitted evolution or simulation step, an
unsupported birth capture or a checked-guard failure restores old virtual outputs and latches the instance
to ordinary execution until restart. Denied work never animates appearance ahead
of its authorized observation. A checked failure then comes from the original
source invocation, retaining its exact location and earlier writes without
replaying previous invocations. `immutable_active()` and execution counters make
actual admission observable; preparation alone is not GPU-route or performance
proof.

## Execution policies

`EffectCompileOptions` makes stage selection, random policy, missing-payload
behavior and event-stream duration interpretation explicit. The original deterministic random stream,
remaining-subframe birth scheduling, transform-mask interpretation and curve
interpolation are implemented policies whose exact native-game fidelity remains
unverified. Constant-capable fields initialize from native constants before the
authored initializer; other fields begin at zero. Native range metadata does not
trigger invented randomization. Authored death-step controls decide whether
terminal-interval evolution runs. `limitations()` exposes these policies and
uncertainties. Native coordinate metadata remains inspectable in the document;
the compiler does not infer a global coordinate conversion from that tuple.
Shape authoring properties have their separate explicit lowering boundary. Renderer
descriptors preserve authored properties and stream bindings; a graphics adapter
must diagnose unsupported enabled features before drawing.

The caller chooses the coordinate frame carried by `SceneInputs`. One `advance()`
uses that input frame throughout its substeps; callers needing intermediate
moving frames can supply smaller separately sampled advances. Long moving-frame
update fidelity remains unverified. Point mask 17
adds translation only, point mask 19 applies the caller's basis and translation,
and direction mask 18 applies its basis without translation. Inverse point mask67
and inverse direction mask 66 use the same selected frame and a checked affine
inverse, including the caller's nonuniform scale. Singular, nonaffine or nonfinite
frames fail explicitly. Exact native scale filtering remains unverified. Mask 11 always uses
the ordinary caller frame. Masks 17/19/18 select the routed payload frame when a
Position or Orientation value was supplied; without either they use the caller
frame. A supplied Position is already world-space. In the payload frame, absent
Position means the origin, absent Orientation means identity, and scale is one;
the ordinary emitter transform is replaced as a whole. The basis includes
any embedded scale; exact native scale filtering remains unverified. A graphics adapter using a
fixed axis conversion should keep that conversion separate from authored T/Q
filters and apply it once when extracting positions.

Payload event declarations map by exact name, native type, footprint and role into
root declarations. Target layouts may have unused extra channels. The default
`EffectCompileOptions::missing_payload` policy is `MissingPayloadPolicy::Error`:
lowering checks that incoming event declarations cover selected script reads,
and extraction fails if a particular birth packet omitted a read channel.

Applications may explicitly select `MissingPayloadPolicy::Zero`. Extraction then
returns zero component words with the valid root declaration's type when that
birth omitted its value, including channels absent from its incoming event schema.
Unknown slots, conflicting layouts, unknown semantics and stale or forged
capabilities still fail. Supplied values remain unchanged. The fallback does not
mark a channel present, alter queued packets or select different event targets.
Position and Orientation therefore feed implicit transforms only when actually
supplied. `EffectProgram::missing_payload_policy()` retains the selection;
`EffectLimitation::MissingPayloadZero` records its unverified native defaults.
A zero quaternion is zero words, not an invented identity orientation.

Each invocation owns its batch, element and draft handles. Appending validates
both event and batch ownership; element values can be reused by independent drafts.
Controls 0–2 create independent drafts, retaining their raw control while its
additional native routing meaning remains unverified. A kick creates one immutable
bounded endpoint snapshot shared by queued fanout births. Each birth samples its
own occurrence fraction and copies canonical values before initialization; queued
ownership is released before that initializer may emit more children. Retiring,
restarting and omission paths recycle storage without allocating.

Constant endpoints preserve exact bits. Varying scalar/vector/color values use
linear interpolation; varying orientations use normalized shortest-arc spherical
interpolation. Emitted-count channels use zero-based cumulative occurrence indices.
These interpolation and index policies remain unverified against the client and
appear in `limitations()` only when the compiled graph can use them. Unknown
semantic words are errors, rather than guesses about unobserved derivative data.
`EffectLimits::payload_words_per_update` separately bounds endpoint-row preparation,
snapshot copying, root-row clearing and interpolation. Exhaustion omits entire
births through `EffectStats::omitted_births`, with no partial payload or backlog.

Spatial declarations retain full resource names separately from layer-local
aliases and preserve payload names, types and read/write controls. Checked
lowering shares references with the same full resource and payload schema within
one effect; separate instances never share mutable rows. The current subset uses
one Float3 payload, payload slot zero, nearest rank zero and zero query options.
Unknown layouts or controls fail rather than acquiring an inferred meaning.

Each actual simulation step reads one immutable previous-step snapshot and builds
the next write snapshot. Inserts never affect another layer's reads in that step.
Key coordinates and returned payload positions remain independent: a scalar key
replicated into XYZ can locate a world-position payload without indexing that
world position. Radius tests include the boundary; exact-distance ties select the
first original insertion. No self exclusion is applied. Missing reads return
unavailable positions and do not stop the effect. Snapshot and tie policies remain
unverified against the native client.

`EffectLimits::spatial_entries` bounds each resource's two preallocated snapshots.
`spatial_checks_per_update` bounds candidate rows examined across all queries,
layers and internal steps of one advance. An incomplete search returns unavailable
rather than a partial nearest result. Entry/draft or unavailable-data omissions
increment `EffectStats::omitted_spatial_writes`; query-work omissions increment
`omitted_spatial_queries`, independently of birth counts. Queries allocate no
storage. Zero-delta calls do not swap snapshots, reset clears them, and omitted
simulation tails discard stale rows without replaying writes later.

Primitive sampling supports boxes, spheres, complex ellipsoids, cylinders,
capsules and cones in surface or volume mode. Lowering applies explicit sparse
defaults and converts shape authoring properties once into the native simulation
frame: position maps `(x, y, z)` to `(x, -z, y)`, box dimensions and ellipsoid scale
swap Y/Z, and XYZ-degree rotations use the corresponding basis conjugation. The
borrowed native view preserves the original properties. This authoring-frame
interpretation remains unverified against the game client.

Sphere volume uses shell-volume weighting; surface shells use area weighting.
Box faces use their areas, with volume sampling for line/point degeneracies.
Cylinder surfaces include side walls only; volume sampling uses a
radial approximation for nonzero inner radius. Capsules combine sphere and
cylinder portions by their surface/volume weights. Cones omit the base cap in
surface mode and ignore inner radius. Only complex ellipsoids apply nonuniform
scale and the hemisphere fold; other primitives ignore that hemisphere toggle.
Fixed six-draw random consumption and sampling distributions remain explicit
fidelity limitations. Zero-radius
and zero-height shapes are legal; mesh shapes and other domain selectors fail.

Procedural turbulence is an original coherent scalar-potential curl-field
approximation. Global scale multiplies both wavelength and velocity strength;
each octave multiplies wavelength by lacunarity and amplitude by gain times its
additional multiplier. The analytic gradients are in lattice coordinates, so
velocity does not acquire an extra inverse-wavelength factor. Linear, cubic and
quintic interpolation are supported. Sparse controls select explicit prepared
defaults, including numeric integer seed `1114229502`, while native omissions stay
`None`. A zero octave count selects one; more than 24 octaves fail preparation.
External PKVF resources, unknown data sources and authored nonzero flow/divergence
or fake-flow controls remain unsupported.

The immutable sampler shares octave data and an original 256-entry seeded
quadrature basis through `EffectProgram`. Eight discrete temporal frequency bands
are an explicit stochastic approximation. A bounded per-instance oscillator cache
uses effect-global execution time, independently of particle age or invocation
order; it evaluates eight sine/cosine pairs on a time-cache miss. The four-entry
cache can evict phases when more moving samplers are interleaved. Field interpolation
uses hashed table lookups and analytic derivatives, with no trigonometry. Frozen
samplers share fully prepared scalar values. Time scale zero freezes the field, negative
scale reverses time, and time base shifts its temporal phase. Sampling consumes
no particle RNG and allocates nothing. Seeded coordinate/channel hashing and
oscillating scalar corners define an independent stochastic basis; equal seeds
do not imply native samples. `TurbulenceField` explicitly records this approximation.

Each callback evaluates at most 576 corner contributions. Frequencies, amplitudes,
queries and results are checked. Scaled lattice coordinates must have magnitude
at most `2^40`; each varied frequency band's angular phase must also
fit `2^40` radians. These f64 bounds preserve resolved fractions/phases rather than
silently saturating coordinates or losing all time variation. The exact all-positive-
infinity unavailable query propagates after resource/context/time validation;
NaN, mixed infinities and overflow fail. These policies are explicit in
`TurbulenceDomain`, and native client stochastic/numeric fidelity remains unverified.

Color conversion treats hue as turns, wraps hue, clamps HSV saturation to 0–1,
and clamps HSV value below zero while retaining HDR values above one. RGB-to-HSV
uses dominant-channel hue, range/value saturation, and zero hue/saturation for a
range of at most `1e-6`. Alpha passes through unchanged and no gamma conversion
occurs. Negative RGB follows the same formula; nonfinite inputs or results fail
explicitly. Exceptional native-game color behavior remains unverified.

Orientation aligns +Z with the supplied normalized forward direction using the
shortest arc; it does not normalize that input implicitly. Opposite directions
use a half-turn about +X and a zero forward vector produces identity. Quaternion
rotation applies the unit-quaternion vector kernel to the authored components
without normalization; native-game behavior for nonunit inputs remains unverified.
Axis-angle rotation normalizes its axis and rejects a zero axis. Emitter side/up
axes use normalized world-transform columns +X/+Z, with those positive unit axes
as degenerate-basis fallbacks. Nonfinite inputs or results fail explicitly.

`vrand` draws unit directions uniformly over a sphere, using the native +Y polar
axis. Its shell form clamps finite radii below zero, accepts reversed endpoints,
and samples the cubic radial interval uniformly. Angle draws precede the optional
radius draw, so the unit form consumes two random values and the shell consumes
three. Cubic interpolation uses normalized radii to avoid overflow for large
finite shells. Nonfinite radii fail explicitly. These distributions and bounded
edge policies do not establish native random-sequence fidelity.

`Value` accepts `f32`, `i32`, `bool`, and glam `Vec2`, `Vec3`, `Vec4` and `Quat`
through `From`, preserving their distinct layouts. Boolean values normalize to
zero/all-ones. For raw encoded fields, `Value::from_words` retains direct control
over component bits.

## Execute a scalar programme

`Script::compile()` takes ordered external value layouts and resolved function
signatures. It validates register scopes, constant writes, component layouts,
receivers and call arguments before producing an owned
`scene::popcorn::Program`. Compilation has configurable storage and instruction
limits; unsupported operations fail with their instruction byte location.

The checked programme uses typed numeric operations, with logical and bitwise
operations kept distinct. Native encodings are resolved during compilation;
execution retains the checked value layouts and original instruction locations.
This lets an application reuse a compiled programme without interpreting native
operation selectors or rebuilding its callback bindings.

Allocate a particle with `Program::new_state()`, or use `State::for_programs()`
to share storage across a layer's initialization and alternative evolution
programmes. Execution clears local scratch and preserves input/stream storage.
`State::clear()` resets a reused particle. The evaluator performs no allocation
after compatible state creation; the host controls allocations in its callbacks.

Implement `Host` to bind external fields and dispatch resolved numeric function
slots. Values preserve typed component bits, and opaque capabilities use binding
identities. Function calls retain native controls without inferring unverified
generic argument writeback. A retirement request belongs to the host's lifecycle
and does not interrupt later instructions. Host return types are checked at
runtime, and errors leave preceding writes observable.

Arithmetic uses host `f32` operations. Signed fraction retains its sign, booleans
use zero/all-ones, integer arithmetic wraps, and integer division by zero fails
explicitly. Float-to-integer conversion uses saturating Rust casts, including
NaN-to-zero. These are defined evaluator policies; their exact native-game edge
fidelity remains unverified.

The scalar evaluator also remains useful as an independent oracle or with an
application-provided host. Compiling an isolated script establishes its checked
instruction and call layout; compiling an effect additionally checks the native
graph bindings and supported callback subset. Rendering still belongs to a
graphics adapter.
