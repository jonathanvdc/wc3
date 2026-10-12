# wc3-derive

Codec derives for maintaining and extending Warcraft III and World of Warcraft
model records in [`wc3`](../wc3/README.md). Applications using the built-in model types only need
the codec traits in `wc3`; the derives are re-exported through its format modules.

## Derives

Import `wc3::model::mdlx::{mdx, mdl}` to distinguish the binary and text derives.
Import `wc3::model::m2` for M2. The format modules re-export the following macros
alongside their codec traits:

| Derive through `wc3` | Purpose |
| --- | --- |
| `mdx::Read`, `mdx::Write` | Read and write binary MDX records. |
| `mdx::ReadOwned` | Decode MDX values into independently owned storage. |
| `mdx::Value` | Generate numeric enum `raw()` and `from_raw()` conversions. |
| `mdl::Read`, `mdl::Write` | Read and write text MDL records. |
| `m2::Read`, `m2::Write`, `m2::RecordLayout` | Read and write offset-addressed M2 records and describe their inline metadata width. |
| `m2::Validate`, `m2::VisitArrays` | Compose field validation and animation-leaf traversal. |

For buffer-bearing records, `wc3::model::common::Ownership` generates
borrowed views and owned conversion independently of any codec. Records name
their buffer lifetime `'data`; the derive delegates fields containing that
lifetime to the shared ownership traits and clones other metadata for views.
Named and tuple structs and enums are supported, and type and const parameters
are preserved. `CommonPartialEq` compares structs across source-buffer
lifetimes through their borrowed views, including fields selected by layout
associated types. Ownership derives have no codec requirements.

Derive `mdx::ReadOwned` separately when a record needs owned MDX decoding, such
as loading through standard I/O. It uses the record's `mdx::Read` implementation
and, for buffer-bearing records, its shared `IntoOwned` contract. It works with
both derived and handwritten readers; records without borrowed storage do not
need an ownership derive.

The generated implementations target `wc3`'s codec APIs. Add `wc3` to use the
re-exported derives:

```sh
cargo add wc3
```

## Example

This record has a binary representation with fields in declaration order and a
text representation as an MDL block:

```rust
use wc3::model::mdlx::{mdx, mdl};
use wc3::model::mdlx::mdx::{Read as _, Write as _};
use wc3::model::mdlx::mdl::{Read as _, Write as _};

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

M2 records also read fields in declaration order, but their arrays are stored
outside the inline metadata. Derive `m2::Read`, `m2::Write`, and
`m2::RecordLayout` together for ordinary structs. `ZeroVec<T>` fields encode a
count/offset pair followed by borrowed fixed-width values; `Vec<T>` fields encode
a count/offset pair addressing record headers and their nested payloads. The
writer reserves the whole inline header before allocating those payloads.
Dedicated field types implement codecs for version-dependent or externally
stored payloads, so containing records can use the same derives. M2 record
methods are `read` and `write`, taking a `Decoder` or `Encoder`; whole model
operations remain `decode_m2` and `encode_m2`.

Ordinary M2 records can also derive `m2::Validate` and `m2::VisitArrays`.
They process struct fields or the active enum payload in declaration order and
return the first error. Each derive requires only its field-level trait;
validation and traversal do not depend on codecs or ownership. Keep handwritten
implementations for cross-field invariants and leaves whose traversal must
supply animation context.

## Documentation

Use the attribute reference when defining records, and the codec and model
guides when connecting those records to whole-model operations:

- [MDX attribute reference](src/mdx.md): fixed fields, sized records, repeated
  entries, animation tails, and numeric enums.
- [MDL attribute reference](src/mdl.md): record shapes, field mappings, defaults,
  animation, and validation hooks. Also included in the
  [crate API documentation](https://docs.rs/wc3-derive/latest/wc3_derive/).
- [M2 derive reference](src/m2.md): field-order records, array descriptors,
  inline metadata widths, and version-selected field types.
- [MDX codec documentation](https://docs.rs/wc3/latest/wc3/model/mdlx/mdx/): binary
  derives, numeric enums, and animation records.
- [`wc3` model documentation](https://docs.rs/wc3/latest/wc3/model/mdlx/): built-in
  records and whole-model operations.

From a workspace checkout, build the derive documentation with:

```sh
cargo doc -p wc3-derive --no-deps --open
```
