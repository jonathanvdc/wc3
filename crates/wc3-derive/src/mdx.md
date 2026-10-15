# MDX derives

The binary derives describe individual record layouts. Import them through
`wc3::model::formats::mdlx::mdx` alongside the traits they implement. They encode fields and
record framing; whole-model codecs supply the `MDLX` header and top-level chunk
headers separately.

## Fixed fields

Struct fields are read and written in declaration order using their own
`mdx::Read` and `mdx::Write` implementations. Named, tuple, and unit structs are
supported. Numeric scalar codecs use little-endian storage. `PhantomData<T>`
fields occupy no bytes.

```rust
use wc3::model::formats::mdlx::mdx;
use wc3::model::formats::mdlx::mdx::{Read as _, Write as _};

#[derive(mdx::Read, mdx::Write)]
struct Pair {
    count: u32,
    weight: f32,
}

let bytes = Pair { count: 3, weight: 0.5 }.encode_mdx()?;
let decoded = Pair::decode_mdx(&bytes)?;
assert_eq!(decoded.count, 3);
assert_eq!(decoded.weight, 0.5);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`decode_mdx` rejects trailing bytes; `read_mdx` consumes one record from a
cursor. For lifetime-bearing structs, the derive uses the first declared
lifetime as the source lifetime. Types without declared lifetimes receive a
reader implementation generic over the input lifetime.

## Owned decoding

Derive `mdx::ReadOwned` when a record needs `decode_owned` or the standard I/O
reader adapter. It is independent of `mdx::Read`: a handwritten reader can use
the same owned-decoding derive. Buffer-bearing records also use the shared
`Ownership` derive to detach borrowed storage; ownership itself has no MDX
requirements. Records without borrowed storage can derive `ReadOwned` without
an ownership implementation.

```rust
use wc3::model::formats::mdlx::mdx;
use mdx::ReadOwned as _;

#[derive(mdx::Read, mdx::ReadOwned)]
struct Identifier(u32);

let value: Identifier = mdx::from_reader(&7u32.to_le_bytes()[..])?;
assert_eq!(value.0, 7);
assert_eq!(Identifier::decode_owned(&8u32.to_le_bytes())?.0, 8);
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Sized records and repeated entries

`#[mdx(sized(tag = *b"XXXX"))]` adds an inclusive `u32` byte length. The tag
identifies errors; it is not written into the record. Reading confines the body
to that length, and writing patches the length after the body is complete.

```rust
use wc3::model::formats::mdlx::mdx;
use wc3::model::formats::mdlx::mdx::{Read as _, Write as _};

#[derive(mdx::Read, mdx::Write)]
#[mdx(sized(tag = *b"TEST"))]
struct Values {
    kind: u32,
    entries: Vec<u32>,
}

let bytes = Values { kind: 1, entries: vec![2, 3] }.encode_mdx()?;
assert_eq!(bytes.len(), 16); // Inclusive length, kind, and two entries.
let decoded = Values::decode_mdx(&bytes)?;
assert_eq!(decoded.entries, [2, 3]);
# Ok::<(), Box<dyn std::error::Error>>(())
```

A `Vec<T>` is permitted only as the final field of a sized struct. Entries are
repeated directly until the bounded body ends, without a separate count. A reader
that consumes no bytes is rejected to prevent an endless loop. Because this field
consumes the remaining body, it leaves no space for a following animation tail.
Use explicit record codecs for other list framing.

## Animation tails and nested groups

A field attribute `#[mdx(tag = *b"KGTR")]` associates an `Animatable<T>` or
`Option<Track<T>>` with a four-byte animation tag. An animatable field writes its
base value among the fixed fields; an optional track has no fixed value and starts
as `None` on read. Track payloads follow the fixed portion of a sized record.

The derives also implement `mdx::ReadTracks` and `mdx::WriteTracks`. An unsized
struct reads and writes only its fixed portion through the ordinary codec traits;
its parent dispatches the animation tail. `#[mdx(flatten)]` on a nested field
forwards track dispatch and output to that group's track implementations while
keeping its fixed fields in their declared position. It cannot be combined with
`tag` on the same field.

Sized readers reject unrecognized tail tags. Repeated recognized tags replace
the previous track, and writers emit tracks in field declaration order. Declare
unique tags throughout nested groups; flattening does not add a namespace.

## Numeric enums

Numeric enums use `#[mdx(value = u32)]` and explicit wire values on unit variants.
Rust discriminants do not define this mapping. An optional `#[mdx(unknown)]`
variant contains exactly one `u32` and preserves unrecognized values:

```rust
use wc3::model::formats::mdlx::mdx;
use wc3::model::formats::mdlx::mdx::{Read as _, Write as _};

#[derive(Debug, PartialEq, mdx::Read, mdx::Write, mdx::Value)]
#[mdx(value = u32)]
enum Choice {
    #[mdx(value = 0)]
    Default,
    #[mdx(value = 2)]
    Alternate,
    #[mdx(unknown)]
    Unknown(u32),
}

assert_eq!(Choice::Alternate.raw(), 2);
assert_eq!(Choice::from_raw(7), Choice::Unknown(7));
let bytes = Choice::Unknown(7).encode_mdx()?;
assert_eq!(Choice::decode_mdx(&bytes)?, Choice::Unknown(7));
# Ok::<(), Box<dyn std::error::Error>>(())
```

Known values must be unique. Without an unknown variant, reading an unrecognized
value returns a codec error. `mdx::Value` generates const `raw()` and `from_raw()`
methods; `from_raw()` returns `Self` with an unknown variant and `Option<Self>`
without one. Preserving an unknown binary value does not give it an MDL spelling.
