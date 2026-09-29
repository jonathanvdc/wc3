# wc3

A Rust library for reading, editing, and writing Warcraft III models in binary
**MDX** and text **MDL** formats, covering Classic and Reforged layouts.

Models share one typed representation across both formats. Use a compile-time
version when you know the layout, or let `DynamicModel` select it from the file.
MDX preserves chunk order and opaque data; MDL produces canonical text and
reports data it cannot faithfully represent.

## Getting started

The library crate is `wc3`.
To use a local checkout, add this dependency to your project's `Cargo.toml`:

```toml
[dependencies]
wc3 = { path = "../wc3/crates/wc3" }
```

Adjust the path to your checkout. Model types live under `wc3::model`. Import
`mdx::Read` / `mdx::Write` and `mdl::Read` / `mdl::Write` as `_` to enable their
methods without conflicting trait names.

### Create a model and encode both formats

```rust
use wc3::model::{Model, V800};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::mdl::Write as _;
use wc3::model::scene::ModelInfo;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut model = Model::<V800>::new();
    model.set_model_info(&ModelInfo::new("Example")?);

    let bytes = model.encode_mdx()?;
    let text = model.encode_mdl()?;
    let decoded = Model::<V800>::decode_mdx(&bytes)?;

    assert_eq!(decoded.model_info().unwrap().name.text(), "Example");
    assert!(text.contains("FormatVersion 800"));
    Ok(())
}
```

This creates a minimal model container. Add geometry, materials, and other
records to build a renderable model.

### Read a model with a runtime version

```rust
use wc3::model::{mdl, DynamicModel};
use wc3::model::mdl::{Read as _, Write as _};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"Version { FormatVersion 800, } Model "Example" {}"#;
    let model = DynamicModel::decode_mdl(source)?;
    let bytes = model.encode_mdx()?;

    // The fallback applies only when the MDX file has no VERS chunk.
    let decoded = DynamicModel::decode_mdx(&bytes, 800)?;
    assert_eq!(decoded.version(), 800);

    let text = decoded.encode_mdl_with_dialect(mdl::Dialect::HiveWorkshop)?;
    assert!(text.contains("FormatVersion 800"));
    Ok(())
}
```

MDL readers accept Warcraft III and Hive Workshop dialects, including mixed
input. Writers use Warcraft III syntax by default; select
`mdl::Dialect::HiveWorkshop` explicitly for its alternative spellings.

## Supported model data

Supported versions are **800, 900, 1000, 1100, 1200, 1300, 1400, 1600, and 1800**,
with corresponding marker types `V800` through `V1800`. `Model<V>` ties chunks
and versioned records to the selected layout; typed reads check the file version.

Both formats support whole models, including:

- Model information, sequences, global sequences, and animation tracks.
- Materials, layers, textures, and texture animations.
- Geosets, geoset animations, pivot points, and bind poses.
- Bones, helpers, attachments, lights, cameras, events, and collision shapes.
- Particle, Particle2, Popcorn, and ribbon emitters, plus face effects and gliders.

Availability and representation depend on the version and output dialect.
The `mdl` module documentation describes field-level coverage and restrictions.

## Editing models

Ordinary records expose public fields for scalars, flags, embedded nodes, and
vectors. Names and paths use `FixedText<N>`: `text()` reads the text,
`set_text()` validates a replacement and clears padding, and `as_bytes()` /
`from_bytes()` provide exact byte access. Geoset geometry and animation-track
internals use methods to preserve structural invariants.

Model collection getters return owned records collected across chunks. Setters
replace the corresponding chunks, so changing a getter's result requires
setting it back. For edits in place, match variants in the public ordered
`model.chunks` vector or use `chunk_mut()`.

`DynamicModel` exposes shared operations through `CommonModelAccess` and
`TryModelAccess`. Use `visit_model!` to run the same expression against each
possible typed model when you need direct access to its records.

## Preservation and errors

**MDX:** decoding retains chunk order, duplicate chunks, unknown chunk payloads,
fixed-width text bytes, and unknown flag bits. Known chunks with malformed
payloads return errors instead of becoming opaque chunks. Encoding reconstructs
typed records from their current values.

**MDL:** decoding preserves represented values, IDs, and references. Encoding
writes deterministic field and block order, merges known collection chunks,
and omits empty optional collections. Comments, whitespace, and original chunk
organization do not survive a text round trip.

MDL writers reject opaque chunks and binary values without a faithful text
representation, including unknown flag bits, invalid fixed text, and certain
hidden animation bases or noncanonical record layouts. NaN payload bits cannot
survive text output. A successful format conversion does not imply byte equality
with the original MDX file.

Value validation uses `ValueError`; codec errors are `mdx::ReadError`,
`mdx::WriteError`, `mdl::ReadError`, and `mdl::WriteError`. MDL read errors provide
`diagnostic(source)` for source-span and line/column details. Streaming writers
may leave partial output on error; use `encode_mdx()` or `encode_mdl()` to obtain
an owned result before writing it to a file.

## Converting versions

Version conversion builds a new model and returns a report, leaving the source
intact:

```rust
use wc3::model::{ConversionOptions, Model, V800, V1100};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = Model::<V800>::new();
    let converted = source.convert::<V1100>(&ConversionOptions::strict())?;
    let target: Model<V1100> = converted.model;

    assert_eq!(target.version(), 1100);
    Ok(())
}
```

Strict conversion preserves shared data, initializes target-only fields with
constructor defaults, and rejects unsupported nondefault data. The report
identifies initialized fields, omitted neutral defaults, discarded data, and
opaque compatibility caveats.

`ConversionOptions::lossy()` permits discarding unsupported data. Unknown chunks
have a separate `UnknownChunkPolicy`: `Reject` by default, `Preserve`, or `Drop`.
Preserving opaque bytes across versions does not guarantee game compatibility.
Conversions do not translate weighted skinning into Classic matrix groups or
shader paths into layer shader IDs, and do not guarantee identical rendering.

## Codec APIs and documentation

The `mdx` and `mdl` modules expose `Read` and `Write` traits for records and
whole models. `mdx::Cursor` / `mdx::Encoder` handle binary streams;
`mdl::Parser` / `mdl::Writer` handle text. MDL parsing borrows
resident UTF-8 input without building an AST or token buffer.

The module docs cover reading, writing, and dialect handling. Internal codec
derive documentation lives in `wc3-derive`. Build the API documentation locally
with:

```sh
cargo doc --workspace --no-deps --open
```

Standard I/O adapters are available as `mdx::from_reader` / `mdx::to_writer`
and `mdl::from_reader` / `mdl::to_writer`. Readers buffer through EOF and
validate complete input. MDX output is buffered; MDL output streams. Neither
writer adapter flushes its sink. Use `mdx::from_reader_with_version` for dynamic
version detection with a fallback and `mdl::to_writer_with_dialect` to select
text syntax.

## Development

Run the workspace checks from the repository root:

```sh
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

Integration tests are grouped into `mdx`, `mdl`, `model`, `derive`, and
`interoperability` targets with nested modules. Allocation checks run in their
own `allocations` target. Run a target with `cargo test -p wc3 --test mdl`, or
filter a module with `cargo test -p wc3 --test mdl core::parser`.

Tests include synthetic models across all supported versions and independent
binary/text fixtures. To enable additional byte-for-byte MDX round-trip checks
against your own model collection:

```sh
WC3_MDX_FIXTURES=/path/to/models cargo test -p wc3 --test corpus -- --ignored
```

The corpus tests are ignored by default and require a nonempty fixture directory.
These checks supplement the synthetic suite; full semantic coverage of the
entire game model collection has not been verified.

## License

MIT OR Apache-2.0, as declared in the crate manifests.
