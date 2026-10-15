## WMO record derives

Import `wc3::model::formats::wmo` to derive `wmo::Read`, `wmo::Write`,
`wmo::RecordLayout`, and `wmo::Validate` for ordinary records. Named, tuple,
unit, and generic structs are supported. Read and write have independent field
bounds, and a writer does not require a fixed record layout when it contains
variable payloads. `RecordLayout` sums fixed inline field widths without a
runtime version argument; associated field types express dialect differences.

```rust
use wc3::model::formats::wmo;
use wmo::codec::{Decoder, Encoder};
use wmo::{Read as _, RecordLayout as _, Validate as _, Write as _};

#[derive(wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
struct Example {
    position: [f32; 3],
    material: u16,
}

let original = Example { position: [1.0, 2.0, 3.0], material: 7 };
assert_eq!(Example::size(), 14);
let mut output = Encoder::new();
original.write(&mut output)?;
let bytes = output.finish();
let decoded = Example::read(&mut Decoder::new(&bytes))?;
decoded.validate()?;
assert_eq!(decoded.material, 7);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Validate` also supports enums, delegating only to fields in the active variant.
Validation stops at the first field error. Relations between fields, file
framing, liquid metadata, and batch-dependent tangent maps belong in handwritten
implementations at the level that owns their context. Derives do not select a
client profile from WMO's shared version number.

### Scalar enums and flags

Use a scalar enum for a field whose wire code selects a named choice. The
`wmo::Value` derive supplies `from_raw`, `raw`, and `From` conversions;
codec and layout derives use the declared integer width independently of that
derive. Every enum requires exactly one `Unknown` payload with the same wire
type, so unrecognized codes round-trip without being interpreted as a known choice.

```rust
use wc3::model::formats::wmo;

#[derive(Clone, Copy, Debug, PartialEq, Eq,
         wmo::Value, wmo::Read, wmo::Write, wmo::RecordLayout)]
#[wmo(value = u16)]
enum Mode {
    #[wmo(value = 0)]
    Disabled,
    #[wmo(value = 1)]
    Enabled,
    #[wmo(unknown)]
    Unknown(u16),
}

assert_eq!(Mode::from_raw(1), Mode::Enabled);
assert_eq!(Mode::from_raw(65535), Mode::Unknown(65535));
assert_eq!(Mode::Unknown(65535).raw(), 65535);
```

Known variants must be unit variants with distinct integer literal codes.
Signed wire types support negative codes. Revision numbers, indices, counts,
and offsets remain numerical fields. Use `bitfield!` tuple structs for flag
words, with named getters and setters for established bits and a whole-word
`bits` accessor for preserving unrecognized bits. Their codec derives delegate
to the underlying integer field; they do not reinterpret the flags as enum choices.
