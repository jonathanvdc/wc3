
## WEM record derives

Import `wc3::model::wem` when defining WEM section records. Ordinary structs
can derive `wem::Read`, `wem::Write`, and `wem::RecordLayout` when their fields
already implement those traits. Field order is the encoded order, and read and
write bounds remain independent. Native section revisions are selected by
record dialects independently of the file envelope generation.

```rust
use wc3::model::wem::{self, Array, Decoder, Encoder, InlineList};

#[derive(wem::Read, wem::Write, wem::RecordLayout)]
struct Example {
    identifier: u32,
    weights: InlineList<u16>,
}

let record = Example { identifier: 7, weights: InlineList(vec![10, 20]) };
let mut output = Encoder::new(1);
output.write(&record)?;
let bytes = output.finish();
let rows = Array::<Example>::read(&bytes, 1, 1)?;
assert_eq!(rows.get(0).unwrap()?.weights.0, [10, 20]);
assert_eq!(<Example as wem::RecordLayout>::minimum_size(), 8);
# Ok::<(), Box<dyn std::error::Error>>(())
```

WEM records may contain variable inline payloads. `RecordLayout::minimum_size`
is a lower bound for safely checking counts, rather than an array stride.
When every field has a constant width, `fixed_size` supplies an exact stride
and arrays require no per-row boundary metadata. Variable records use a checked
boundary index instead.
`InlineList<T>` writes a four-byte count followed by its rows; an empty list
still contributes the count word. `InlineOption<T>` writes a presence byte
and, when that byte is nonzero, the payload. Ordinary referenced arrays and
strings use twelve-byte `Ref<T>` fields instead of inline lists.

`wem::Read` also generates `skip` by delegating to each field's `skip` method.
`Array::read` uses these methods to index row boundaries while borrowing the
encoded payload. It does not materialize inline lists during the scan. Accessing
a row decodes that record and owns its inline-list values. Handwritten readers
can override `skip` to retain this behavior; the default calls `read` and drops
the result.

Use `Bool` and `InlineOption` when preserving native discriminator bytes.
`Bool` interprets every nonzero byte as true while retaining the actual byte;
`InlineOption` retains the presence byte alongside its optional payload. Its
writer rejects a presence/value mismatch. The standard Rust `bool` and
`Option<T>` codecs instead accept canonical zero/one discriminators. Record
layouts containing native machine words require an explicit ABI choice rather
than deriving word width from the file generation.

Derive `wem::Validate` and `wem::VisitReferences` to delegate checks and link
traversal through struct fields or an active enum payload. These traits have no
codec bounds. Reference visitors receive the target tag when the field's target
marker identifies it, without loading the target section or choosing its
revision. Write handwritten validation for relationships between fields and
handwritten codecs for runtime-discriminated payloads. The codec derives
support structs only and have no format attributes.
