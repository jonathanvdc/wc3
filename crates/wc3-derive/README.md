# wc3-derive

Procedural macros for defining native model records in [`wc3`](../wc3/README.md).
If you use the built-in model types, start with that crate's codecs. These
derives are for contributors adding records or maintaining their layouts.

## Define a record

Depend on `wc3` and import the derives through its format modules. There is no
need to depend on `wc3-derive` directly:

```sh
cargo add wc3
```

This record stores its fields in declaration order in binary MDX and writes an
`Example` block in text MDL:

```rust
use wc3::model::formats::mdlx::{mdx, mdl};
use wc3::model::formats::mdlx::mdx::{Read as _, Write as _};
use wc3::model::formats::mdlx::mdl::{Read as _, Write as _};

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

The result is one record, not a complete model file. Whole-model codecs add
file headers, chunks, and other framing.

## Choose a format reference

The references explain supported record shapes, attributes, and the cases that
need handwritten implementations. Each format module re-exports its traits and
derives under the same names.

| Format | Main derives | Reference |
| --- | --- | --- |
| MDX | `mdx::Read`, `mdx::Write`, `mdx::ReadOwned`, `mdx::Value` | [Binary fields, sized records, animation tails, and enums](src/mdx.md) |
| MDL | `mdl::Read`, `mdl::Write` | [Text blocks, properties, defaults, and hooks](src/mdl.md) |
| M2 | `m2::Read`, `m2::Write`, `m2::RecordLayout`, `m2::Value`, `m2::Validate`, `m2::VisitArrays` | [Offset-addressed records and animation traversal](src/m2.md) |
| WMO | `wmo::Read`, `wmo::Write`, `wmo::RecordLayout`, `wmo::Value`, `wmo::Validate` | [Inline records, validation, and enums](src/wmo.md) |

Import `mdx` and `mdl` from `wc3::model::formats::mdlx`, and `m2` and `wmo`
from `wc3::model::formats`. Import a codec trait as `_` when you need its methods
alongside a derive of the same name.

## Borrowed and owned storage

Buffer-bearing records can derive `wc3::model::common::Ownership` to generate
borrowed views and conversions to owned storage. Use `'data` for the buffer
lifetime. Fields containing that lifetime delegate to the shared ownership
traits; other metadata is cloned for a view. Structs and enums are supported,
including type and const parameters. Ownership does not require a codec.

Derive `mdx::ReadOwned` separately to decode through standard I/O or detach a
borrowed MDX record. It uses the record's `mdx::Read` implementation and its
`IntoOwned` contract, whether the reader is derived or handwritten. Records
without borrowed storage need no ownership derive.

`CommonPartialEq`, also in `wc3::model::common`, compares records across buffer
lifetimes through their borrowed views, including version-selected field types.

## API documentation

The [crate API documentation](https://docs.rs/wc3-derive/latest/wc3_derive/)
includes the format references. To build it from this checkout, run:

```sh
cargo doc -p wc3-derive --no-deps --open
```
