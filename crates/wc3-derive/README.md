# wc3-derive

Codec derives for maintaining and extending Warcraft III model records in
[`wc3`](../wc3/README.md). Applications using the built-in model types only need
the codec traits in `wc3`; the derives are re-exported through its format modules.

## Derives

| Derive through `wc3` | Purpose |
| --- | --- |
| `mdx::Read`, `mdx::Write` | Read and write binary MDX records. |
| `mdx::Value` | Generate numeric enum `raw()` and `from_raw()` conversions. |
| `mdl::Read`, `mdl::Write` | Read and write text MDL records. |

The generated implementations target `wc3`'s codec APIs. Add `wc3` to use the
re-exported derives:

```sh
cargo add wc3
```

## Example

This record has a binary representation with fields in declaration order and a
text representation as an MDL block:

```rust
use wc3::model::{mdx, mdl};
use wc3::model::mdx::{Read as _, Write as _};
use wc3::model::mdl::{Read as _, Write as _};

#[derive(mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(block = "Example")]
struct Example {
    #[mdl(property = "Size")]
    size: f32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let record = Example::decode_mdl("Example { Size 2.0, }")?;
    let bytes = record.encode_mdx()?;
    let decoded = Example::decode_mdx(&bytes)?;
    assert_eq!(decoded.size, 2.0);
    assert!(decoded.encode_mdl()?.contains("Size 2.0,"));
    Ok(())
}
```

A record codec encodes that record, rather than a complete model file.

## Documentation

- [MDL attribute reference](src/mdl.md): record shapes, field mappings, defaults,
  animation, and validation hooks. Also included in the
  [crate API documentation](https://docs.rs/wc3-derive/latest/wc3_derive/).
- [MDX codec documentation](https://docs.rs/wc3/latest/wc3/model/mdx/): binary
  derives, numeric enums, and animation records.
- [`wc3` model documentation](https://docs.rs/wc3/latest/wc3/model/): built-in
  records and whole-model operations.

From a workspace checkout, build the derive documentation with:

```sh
cargo doc -p wc3-derive --no-deps --open
```
