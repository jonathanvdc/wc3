# Baked Popcorn effects

`model::formats::pkb` reads the native envelopes and compiled instruction streams
of version 2.5.1 PKB effects. Use it to inspect an external effect referenced by an
MDLX Popcorn feature. Reading a PKB does not simulate particles or make the effect
renderable in `bevy-wc3`.

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
object link. Unsupported shape, turbulence and future payload families remain raw
links. `curve_sampler()` and `event_stream_sampler()` expose borrowed native arrays
without applying simulation policy. Curve type, interpolation and quality omissions
remain `None`; the native view does not infer scalar width from sparse metadata.
Localized curve names and editor coordinates remain inspection data. Scheduling,
curve interpolation and unsupported-family diagnostics belong to checked lowering.

## Execute a checked programme

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

The evaluator supplies arithmetic and a host boundary, rather than implementations
of named native functions. Samplers, layer events, lifecycle scheduling, resource
resolution and rendering still require implementation. Compiling a script does
not establish that an entire effect can execute or render.
