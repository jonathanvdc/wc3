# wc3-mdx

Work in progress: a pure Rust Warcraft III MDX reader and writer. The current
crate parses the MDX container into ordered raw chunks and writes it back
without changing chunk data. Typed coverage is growing one chunk at a time.
The target range is MDX versions 800 through 1800.

Typed access is available for model information, animation sequences, texture
references, global sequences, pivot points, basic geoset mesh data, material
and layer headers, geoset animations, bones, helpers, node transform tracks,
bind-pose matrices, event objects, collision shapes, cameras, lights,
attachments, face animation references, popcorn emitters, and texture
animations. Unknown fields and chunks remain available as bytes. Full semantic
support across the target version range has not yet been verified.

## Rust API

```rust
use wc3_mdx::{Chunk, Model, ModelInfo};

let mut model = Model::new(800);
model.set_model_info(&ModelInfo::new("Example")?);
model.push(Chunk::new(*b"TEST", vec![1, 2, 3]));
let encoded = model.to_bytes()?;
let decoded = Model::from_bytes(&encoded)?;
assert_eq!(decoded.version(), Some(800));
let info = decoded.model_info()?.unwrap();
assert_eq!(info.name(), "Example");
# Ok::<(), wc3_mdx::Error>(())
```

Run unit and integration tests with `cargo test --all-targets`.
Set `WC3_MDX_FIXTURES` to a directory of local `.mdx` files to include
recursive byte-for-byte round-trip checks.
