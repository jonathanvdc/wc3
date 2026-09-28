# wc3-mdx

Pure Rust Warcraft III MDX reader and writer for Classic and Reforged models.
The crate keeps decoded chunks in file order. Unknown chunks retain their
exact payload bytes. Known chunks that cannot be decoded return an error.
Typed models are available for MDX versions 800, 900, 1000, 1100, 1200, and 1800.

Typed access covers the standard model, sequence, material, texture, geoset,
node, animation, emitter, light, camera, attachment, collision, face effect,
and bind pose chunks. Flag fields expose named bits while retaining unknown
bits. The package has no runtime dependencies.

The tests cover synthetic models across the supported versions. Optional
fixture tests check byte-for-byte round trips on local models. Full semantic
coverage across the entire game collection has not yet been verified.

## Rust API

```rust
use wc3_mdx::{DynamicModel, Model, V800};
use wc3_mdx::io::{Readable, Writable};
use wc3_mdx::scene::ModelInfo;

let mut model = Model::<V800>::new();
model.set_model_info(&ModelInfo::new("Example")?);
let encoded = model.encode()?;
let decoded = Model::<V800>::decode(&encoded)?;
assert_eq!(decoded.version(), 800);
let info = decoded.model_info().unwrap();
assert_eq!(info.name(), "Example");
assert!(matches!(DynamicModel::decode(&encoded, 800)?, DynamicModel::V800(_)));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Constructors and setters that can reject values return `ValueError`. Binary
decoding returns `DecodeError`; encoding returns `EncodeError`.

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
tracks, and `encode()` reconstructs records while preserving field bits,
fixed-width names, and optional section order. Geoset accessors such as
`vertices()` borrow decoded data, and `vertices_mut()` supports bulk edits.

## Version conversion

Conversions build a new typed model and leave the source intact:

```rust
use wc3_mdx::{ConversionOptions, Model, V800, V1100};

let source = Model::<V800>::new();
let converted = source.convert::<V1100>(&ConversionOptions::strict())?;
let target: Model<V1100> = converted.model;
let report = converted.report;
# Ok::<(), wc3_mdx::ConversionError>(())
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

`wc3_mdx::mdl` provides a borrowing `Lexer`, a copyable `Parser` with one token
of lookahead, source-span diagnostics, and an `MdlWriter<W: std::io::Write>`.
Parsing operates on resident UTF-8 text without an AST or a token buffer.
Strings retain literal backslashes and embedded line breaks. Only `//` comments
are supported. Numeric readers check ranges and accept the MDL non-finite float
literals; finite float output round-trips exactly, including negative zero.
NaN payload bits are not preserved by text output.

`MdlRead` / `MdlWrite` implementations currently cover `Texture` (`Bitmap`),
`Sequence` (`Anim`, including `SyncPoint`), `ModelInfo` (`Model`),
`GlobalSequence` (`Duration`), and `PivotPoint` (an anonymous vector entry). Readers accept fields
in any order, apply defaults, and reject unknown or duplicate fields. An `Anim`
requires `Interval`. `parse_mdl()` requires exactly one record; `Parser::read()`
consumes one record from a larger stream.

Counted lists yield records on demand, so individual records can go directly
into the binary encoder:

```rust
use wc3_mdx::animation::GlobalSequence;
use wc3_mdx::mdl::Parser;
use wc3_mdx::Encoder;

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

`#[derive(MdlRead, MdlWrite)]` generates codecs for named-field structs with
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
`MdlFlags` type, initialize to zero, and reject duplicate names and unknown bits.
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
