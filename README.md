# wc3

Pure Rust Warcraft III model codecs, with MDX reading and writing for Classic and Reforged models.

The [MDL support contract and coverage inventory](docs/MDL_SUPPORT.md) records
the agreed dialect behavior, preservation rules, and remaining codec work.
The crate keeps decoded chunks in file order. Unknown chunks retain their
exact payload bytes. Known chunks that cannot be decoded return an error.
Typed models are available for MDX versions 800, 900, 1000, 1100, 1200, 1300,
1400, 1600, and 1800.

Typed access covers the standard model, sequence, material, texture, geoset,
node, animation, emitter, light, camera, attachment, collision, face effect,
and bind pose chunks. Flag fields expose named bits while retaining unknown
bits. The package has no runtime dependencies.

The tests cover synthetic models across the supported versions. Optional
fixture tests check byte-for-byte round trips on local models. Full semantic
coverage across the entire game collection has not yet been verified.

## Rust API

Model types live under `wc3::model`. The `mdx` and `mdl` modules each expose
`Read` and `Write` traits and derives; use qualified names such as
`#[derive(mdx::Read, mdx::Write, mdl::Read, mdl::Write)]` to distinguish formats.

```rust
use wc3::model::{DynamicModel, Model, V800};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::scene::ModelInfo;

let mut model = Model::<V800>::new();
model.set_model_info(&ModelInfo::new("Example")?);
let encoded = model.encode_mdx()?;
let decoded = Model::<V800>::decode_mdx(&encoded)?;
assert_eq!(decoded.version(), 800);
let info = decoded.model_info().unwrap();
assert_eq!(info.name(), "Example");
assert!(matches!(DynamicModel::decode_mdx(&encoded, 800)?, DynamicModel::V800(_)));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Constructors and setters that can reject values return `ValueError`. Binary
decoding returns `mdx::ReadError`; encoding returns `mdx::WriteError`.

`Model<V>::chunks()` exposes `chunks::ModelChunk<V>` variants. Versioned records
in those chunks also carry `V`. An `Unknown` chunk can only use a tag that the
library does not recognize. Editing a typed chunk writes its new payload.

Run the full test suite with `cargo test --workspace`. Set `WC3_MDX_FIXTURES`
to a directory of local `.mdx` files for additional round-trip checks.

`Geoset<V>`, `Material<V>`, `Layer<V>`, `Camera<V>`, and `Light<V>` select their
version-specific fields through traits implemented beside those records.
Setters on `Model<V>` accept records with the same `V`.

Geosets and variable-length records such as materials, nodes, lights,
cameras, emitters, and bind poses store decoded sections. Track accessors borrow parsed
tracks, and `encode_mdx()` reconstructs records while preserving field bits,
fixed-width names, and optional section order. Geoset accessors such as
`vertices()` borrow decoded data, and `vertices_mut()` supports bulk edits.

## Version conversion

Conversions build a new typed model and leave the source intact:

```rust
use wc3::model::{ConversionOptions, Model, V800, V1100};

let source = Model::<V800>::new();
let converted = source.convert::<V1100>(&ConversionOptions::strict())?;
let target: Model<V1100> = converted.model;
let report = converted.report;
# Ok::<(), wc3::model::ConversionError>(())
```

`DynamicModel` and the versioned material, layer, geoset, light, and camera
records also expose `convert::<TargetVersion>()`. The result contains the
converted value in `model` and a report with paths identifying initialized
fields, omitted neutral defaults, discarded data, and opaque compatibility
caveats. Errors identify the source and target versions and the first field
or chunk that cannot be converted under the selected policies.

Strict conversion preserves shared storage, including raw names, flag bits,
track and optional-section order, duplicate chunks, and version extensions.
Target-only fields use their existing constructor defaults. Unsupported
fields may be omitted when equal to their neutral defaults (for example,
emissive gain 1.0, zero shadow intensity, or empty texture slots); other
values, unsupported animation tracks, and unsupported chunks cause an error.
A missing `VERS` chunk is inserted. Camera variants retain their explicit,
self-describing layout rather than adopting the target constructor's default.

`ConversionOptions::lossy()` explicitly permits discarding unsupported data.
Unknown chunks still require a separate `unknown_chunks` policy:
`UnknownChunkPolicy::Reject` (the default), `Preserve`, or `Drop`. Preserving
opaque bytes across versions does not guarantee compatibility with the game.
Unknown chunks are always retained when the source and target versions match.

Shader paths are preserved between V900 and V1000. Nonempty shader paths
cannot yet be translated to layer shader IDs, so conversion to other layouts
requires explicit loss permission. Weighted skinning is not translated into
Classic matrix groups. Skin indices above 255 cannot be represented before
V1400: strict conversion fails, while lossy conversion drops the whole skin
section rather than truncating indices. These operations convert supported
format data; they do not guarantee identical rendering across game versions.

## MDL primitives

`wc3::model::mdl` provides a borrowing `Lexer`, a copyable `Parser` with one token
of lookahead, source-span diagnostics, and an `mdl::MdlWriter<W: std::io::Write>`.
Parsing operates on resident UTF-8 text without an AST or a token buffer.
Strings retain literal backslashes and embedded line breaks. Only `//` comments
are supported. Numeric readers check ranges and accept the MDL non-finite float
literals; finite float output round-trips exactly, including negative zero.
NaN payload bits are not preserved by text output.

`mdl::Read` / `mdl::Write` implementations currently cover `Texture` (`Bitmap`),
`Sequence` (`Anim`, including `SyncPoint`), `ModelInfo` (`Model`),
`GlobalSequence` (`Duration`), and `PivotPoint` (an anonymous vector entry). Readers accept fields
in any order, apply defaults, and reject unknown or duplicate fields. An `Anim`
requires `Interval`. `decode_mdl()` requires exactly one record; `Parser::read()`
consumes one record from a larger stream. `encode_mdl()` returns an owned UTF-8
`String`; `write_mdl()` writes to an existing `MdlWriter`. MDX uses the matching
`decode_mdx()` / `encode_mdx()` and `read_mdx()` / `write_mdx()` methods.

Counted lists yield records on demand, so individual records can go directly
into the binary encoder:

```rust
use wc3::model::animation::GlobalSequence;
use wc3::model::mdl::Parser;
use wc3::model::mdx::Encoder;

let mut parser = Parser::new("GlobalSequences 2 { Duration 1000, Duration 2500, }");
parser.expect_ident("GlobalSequences")?;
let mut bytes = Vec::with_capacity(8);
let mut encoder = Encoder::new(&mut bytes);
for record in parser.counted::<GlobalSequence>()? {
    encoder.write(&record?)?;
}
parser.finish()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

This example writes the collection payload, not an entire MDX model. Counted
readers validate the declared count and closing brace when exhausted; call
`finish()` to drain and validate a list after stopping early. Each record codec
owns its entry punctuation. Dropping a block or list does not validate unread
input. Parser copies are explicit checkpoints for speculative reads.

The writer uses tabs, LF, and deterministic field order. It rejects unknown flag
bits, model animation-file data, non-UTF-8 text, nonzero text padding, and unterminated fixed
text rather than silently losing binary data. Literal quotes and NUL cannot be
written inside strings. Errors may leave partial output. Use
`error.diagnostic(source)` to display line/column and the offending source span.

`#[derive(mdl::Read, mdl::Write)]` generates codecs for named-field structs with
`#[mdl(block = "Name")]`. Fields explicitly specify `header`,
`property = "Name"`, `flag = "Name"` (bool), or `skip`. Properties and flags are
required unless given `default` or a `default = "factory"`; skipped fields need
an explicit default. Optional bool flags use `default` to start false.
`skip_if = "predicate"` controls property omission separately from defaults.
Custom `read_with` / `write_with` value codecs and container `validate_read` /
`validate_write` hooks cover irregular data and binary preservation checks.
Generated codecs use stack locals and the existing parser/writer; ModelInfo
uses these derives, including validation of its animation-file field. Texture,
Sequence, GlobalSequence, and PivotPoint now use derives as well.

See the `mdl` module documentation for examples, hook signatures, and attribute
rules. Packed `flags(Name = 1, Other = 2)` mappings work with `u32` or an
`BitRange<u32>` type (with `Default` and `BitRangeMut<u32>` for parsing),
initialize to zero, and reject duplicate names and unknown bits.
`write_order(field_a, field_b, ...)` preserves MDL field order independently of
binary layout. Single-field tuple structs support `#[mdl(property = "Name")]`
and anonymous `#[mdl(entry)]` forms. The derives support at most 64 body names;
counted collections and enums remain handwritten. Whole-model MDL conversion is
not implemented yet.

Texture paths occupy 260 bytes (up to 259 UTF-8 bytes plus NUL). ModelInfo stores
an 80-byte name followed by a separate 260-byte animation-file path, exposed by
`animation_file_name()` and `set_animation_file_name()`. Editing the name preserves
the animation-file bytes. The supported MDL Model block has no animation-file
property, so nonzero animation-file data is rejected on MDL output.

## MDL model foundations

Event frame APIs use signed `i32` times, including negative lead-in frames.
Material priority planes also use `i32`; PRE2 priority remains unsigned.
These signed API changes preserve the existing four-byte binary layout.

`scene::CameraTrack` includes visibility and the three depth-of-field tracks.
Its scalar DOF helpers construct stepped keys at frame zero, and MDL output
uses the keyed spellings to avoid the client's scalar focal-length/f-stop swap.
`materials::ShaderType` is the layer's stored shader type, wrapping any `u32`
ID. Use `layer.set_shader_type(ShaderType::HD_DEFAULT_UNIT)` for named shaders
or `ShaderType::new(id)` for raw IDs. `shader_type()` returns the wrapper and
`id()` retrieves its exact binary value; `name()` is optional for unnamed IDs.
The checked accessors are `try_shader_type()` / `try_set_shader_type()`. These
replace the previous raw `shader_type_id` accessors, and the layout associated
type is now `MaterialLayout::ShaderType`.

`scene::Glider` and `chunks::GlidersChunk` represent the `DILG` world-picking
whitelist. `gliders()` / `set_gliders()` work on every typed version and through
`CommonModelAccess` on runtime models. Clearing the list removes its chunks.
Gliders have no version gate and are preserved during conversion.

New light constructors use white direct and ambient colors. New Popcorn
emitters use white color and unit lifespan, emission rate, speed, and alpha.
Binary decoding retains the values actually stored in the file.

## Delegated MDL properties

`#[mdl(property = "Name", delegate)]` delegates the property's payload, missing
value policy, and complete output to the field type's `mdl::ReadProperty` /
`mdl::WriteProperty` implementations. The derive retains name dispatch,
duplicate checks and output ordering, and adds no version checks. A missing
property is required by default; the field codec can instead supply a default
or absent storage. Unavailable field types reject presence with a source span
and omit the whole property on output. Delegated preflight validation runs
before the record writes any bytes.

`Option<T>` supports this interface directly: None omits the property and
Some(value) writes it. Delegated codecs own defaults and omission, including
in records with `#[mdl(default)]`; field-level default/required/skip_if and
value hooks are incompatible. The current form covers ordinary named
properties. Static/animated delegation remains a later extension.
