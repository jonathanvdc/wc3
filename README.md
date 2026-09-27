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
use wc3_mdx::{AnyVersionModel, Model, V800};
use wc3_mdx::io::{Readable, Encodable};
use wc3_mdx::scene::ModelInfo;

let mut model = Model::<V800>::new();
model.set_model_info(&ModelInfo::new("Example")?);
let encoded = model.encode()?;
let decoded = Model::<V800>::decode(&encoded)?;
assert_eq!(decoded.version(), 800);
let info = decoded.model_info().unwrap();
assert_eq!(info.name(), "Example");
assert!(matches!(AnyVersionModel::decode(&encoded, 800)?, AnyVersionModel::V800(_)));
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
