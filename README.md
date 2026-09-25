# wc3-mdx

Pure Rust Warcraft III MDX reader and writer for Classic and Reforged models.
The crate keeps chunks in file order and preserves their exact payload bytes,
including unknown fields and chunks. The target range is MDX versions 800
through 1800.

Typed access covers the standard model, sequence, material, texture, geoset,
node, animation, emitter, light, camera, attachment, collision, face effect,
and bind pose chunks. Flag fields expose named bits while retaining unknown
bits. `Model::validate()` checks known record layouts and tracks. The package
has no runtime dependencies.

Byte-for-byte round-trip has been tested on a small local set of version 1800
models and on synthetic models spanning 800 through 1800. Full semantic
coverage across the entire game collection has not yet been verified.

## Rust API

```rust
use wc3_mdx::{Record, RawChunk, Model, ModelInfo};

let mut model = Model::new(800);
model.set_model_info(&ModelInfo::new("Example")?);
model.push(RawChunk::new(*b"TEST", vec![1, 2, 3]));
let encoded = model.encode()?;
let decoded = Model::decode_latest(&encoded)?;
decoded.validate()?;
assert_eq!(decoded.version(), 800);
let info = decoded.model_info()?.unwrap();
assert_eq!(info.name(), "Example");
# Ok::<(), wc3_mdx::Error>(())
```

Run unit and integration tests with `cargo test --all-targets`.
Set `WC3_MDX_FIXTURES` to a directory of local `.mdx` files to include
recursive byte-for-byte round-trip checks.

`Geoset`, `Material`, and `Layer` retain the MDX version supplied when they
are created or decoded. Their field accessors use that version automatically.
For example, `model.materials()?` returns materials whose `layers()` and
`shader()` methods need no version argument. Model setters reject records
built for a different version.

Geosets and variable-length records such as materials, nodes, lights,
cameras, emitters, and bind poses store decoded sections. Track accessors borrow parsed
tracks, and `encode()` reconstructs records while preserving field bits,
fixed-width names, and optional section order. Geoset accessors such as
`vertices()` borrow decoded data, and `vertices_mut()` supports bulk edits.
