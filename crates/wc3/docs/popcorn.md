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
defaults and coordinate conversion belong to checked lowering. Turbulence and
future payload families remain raw links. `curve_sampler()` and
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

```rust
use wc3::model::formats::pkb::{Document, EffectCompileOptions};
use wc3::model::scene::popcorn::{EffectLimits, SceneInputs};

# fn simulate(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
let document = Document::read(bytes)?;
let (link, _) = document.effect_roots().next().ok_or("missing effect")??;
let program = document.compile_effect(link, &EffectCompileOptions::default())?;
let mut instance = program.instantiate(EffectLimits::default(), 42)?;
instance.start(&program, program.default_entry(), SceneInputs::default())?;
instance.advance(&program, 1.0 / 60.0, SceneInputs::default())?;
for renderer in program.renderers() {
    for particle in instance.particles(renderer.layer) {
        for input in &renderer.inputs {
            let value = particle.field(input.field);
            // Feed these typed canonical values into an application renderer.
        }
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
sampling, and primitive `samplePosition` callbacks. It
executes the complete acquired HD LichAttack graph, including invisible
controllers and its routed child layer. Scene collision, broadcasts, spatial queries, unsupported payload
semantics, other transform masks, unknown callbacks and unresolved sampler
families fail explicitly. When a native curve omits dimensionality, lowering
resolves it from the return registers of its bound `sample` calls. All uses across
owning layers and programme stages must agree, and values/tangents must fit that width. The borrowed native
view still reports `None`; unused sparse curves remain unresolved.

## Execution policies

`EffectCompileOptions` makes stage selection, random policy and event-stream
duration interpretation explicit. The original deterministic random stream,
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
root declarations. Target layouts may have unused extra channels. Lowering checks
that every incoming route supplies channels explicitly read by selected scripts;
missing reads, conflicting layouts, unknown semantics and stale capabilities fail
explicitly. Position and Orientation also feed transforms implicitly, so they are
retained even when no extraction callback reads them.

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
