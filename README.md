# wc3-mdx

Work in progress: a pure Rust Warcraft III MDX reader and writer. The current
crate parses the MDX container into ordered raw chunks and writes it back
without changing chunk data. Semantic decoding of model objects, geometry,
animations, and Reforged extensions is still to be implemented.
The target range is MDX versions 800 through 1800.

## Rust API

```rust
use wc3_mdx::{Chunk, Model};

let mut model = Model::new(800);
model.push(Chunk::new(*b"MODL", vec![0; 372]));
let encoded = model.to_bytes()?;
let decoded = Model::from_bytes(&encoded)?;
assert_eq!(decoded.version(), Some(800));
# Ok::<(), wc3_mdx::Error>(())
```

Run unit and integration tests with `cargo test --all-targets`.
