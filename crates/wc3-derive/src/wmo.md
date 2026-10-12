## WMO record derives

Import `wc3::model::wmo` to derive `wmo::Read`, `wmo::Write`,
`wmo::RecordLayout`, and `wmo::Validate` for ordinary records. Named, tuple,
unit, and generic structs are supported. Read and write have independent field
bounds, and a writer does not require a fixed record layout when it contains
variable payloads. `RecordLayout` sums fixed inline field widths without a
runtime version argument; associated field types express dialect differences.

```rust
use wc3::model::wmo;
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
