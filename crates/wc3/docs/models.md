# Reading and editing Warcraft III documents

MDX and MDL share the document types in `wc3::model::formats::mdlx`. Decode a
document, edit its native records, and encode it in either format. The
[crate README](../README.md) provides a first example; this guide explains
the choices that matter for editors, converters, and asset pipelines. For checked
authoring relationships, use the default [model representation](semantic.md) and
its [native import and export adapters](conversion.md).

## Choose a model version

Use `Document<V800>` when the input must use a particular layout. Its decoder checks
the file's version against that marker. Supported markers are `V800`, `V900`,
`V1000`, `V1100`, `V1200`, `V1300`, `V1400`, `V1600`, and `V1800`.
`FormatVersion` describes a numeric revision and its capabilities. `Dialect`
combines that revision with application extensions; ordinary version markers
implement both traits. These capabilities determine which fields and collections
are available.

Use `DynamicDocument` when the input version is discovered at runtime. MDX decoding
requires a fallback version for files without a `VERS` chunk; it does not override
an explicit version. MDL requires a `Version` block followed by a `Model` block:

```rust
use wc3::model::formats::mdlx::{DynamicDocument, NoExtensions};
use wc3::model::formats::mdlx::mdl::{Read as _, Write as _};

let source = r#"Version { FormatVersion 800, } Model "Example" {}"#;
let model = DynamicDocument::<NoExtensions>::decode_mdl(source)?;
assert_eq!(model.version(), 800);
let binary = model.encode_mdx()?;
let decoded = DynamicDocument::<NoExtensions>::decode_mdx(&binary, 800)?;
assert_eq!(decoded.version(), 800);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`NoExtensions` retains unknown MDX chunks as opaque bytes. Applications defining
additional chunks can use `Extended<V, E>` with `Document`, or select `E` on
`DynamicDocument`. The [model API](https://docs.rs/wc3/latest/wc3/model/formats/mdlx/) explains
how `mdx::Extension` and `mdl::Extension` dispatch those payloads.

## Edit collections and fields

Document getters for borrowed record collections return iterators without temporary
vectors. Named mutable iterators such as `geosets_mut()` and `materials_mut()`
edit records in place while preserving chunk boundaries and file order. Named
record setters accept arrays, vectors, or iterators of records, moving their
buffers into one replacement chunk. Use `.iter().cloned()` when replacing a
collection from records you want to retain.

Ordinary record fields are public. Names and paths use `FixedText`; its
`set_text()` method validates length and embedded NULs. Geometry setters coordinate
arrays that must agree in size. Track constructors pair interpolation with the
appropriate keyframe representation.

## Work with borrowed and owned buffers

MDX decoding also borrows geometry, animation keys, and opaque payloads from the
input slice. Numeric arrays use ZeroVec to read little-endian, unaligned data
without allocating a typed buffer; `get()` and `iter()` return ordinary Rust
values. `Geoset::new` and the track constructors accept native arrays, vectors,
and slices through `IntoZeroVec`. For example,
`Track::linear([first, second], None)` encodes those keys into owned storage.
Passing an existing ZeroVec retains its buffer during conversion. Track and event
records preserve their stored order and duplicate times, including unordered
borrowed keys. Semantic import canonicalizes curves for evaluation; native codecs
do not sample animation or resolve playback clocks. Collect generated values into
`ZeroVec` explicitly.

The arrays expose ZeroVec directly for inspection and editing. Edit encoded ULE
elements with `to_mut_slice()` or `with_mut()`. Only the array being edited becomes
owned. MDL decoding builds encoded arrays directly, and opaque payloads use
`Cow<[u8]>`.

Use `as_borrowed()` to create an editable view that borrows even buffers the
original model owns. `to_owned()` creates an independent deep copy.
`into_owned()` copies borrowed buffers and moves owned buffers into an
independent model. Cloning borrowed buffers is cheap; cloning owned buffers
copies them. Version conversion follows the same distinction:
`convert()` borrows its source, while `into_convert()` moves compatible buffers.

Keep the input byte buffer alive while using a borrowed MDX model. For standard
I/O, `mdx::from_reader` buffers the source and detaches the result; use
`mdx::from_reader_with_version` for runtime version selection. MDL parsing owns
its record data, so its model does not borrow the input text.

## Convert layouts and inspect the report

Conversion changes the typed layout, rather than merely replacing the numeric
version field. `convert::<Target>()` retains and borrows the source;
`into_convert::<Target>()` consumes it and moves compatible buffers. Both return
`Conversion { model, report }`:

```rust
use wc3::model::formats::mdlx::{ConversionOptions, Document, V800, V1800};

let source = Document::<V800>::new();
let converted = source.into_convert::<V1800>(&ConversionOptions::strict())?;
assert_eq!(converted.model.version(), 1800);
for issue in &converted.report.issues {
    println!("{:?} at {}: {}", issue.kind, issue.path, issue.description);
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

Strict conversion rejects unsupported nondefault fields. `ConversionOptions::lossy()`
permits those values to be dropped and reports each loss. Opaque chunks have their
own `UnknownChunkPolicy` when changing versions: reject, preserve exact bytes
without a target-compatibility
guarantee, or drop. Lossy field conversion leaves that policy at reject. Reports
also identify initialized fields, omitted defaults, and equivalent normalization;
success does not imply identical appearance in Warcraft III.

## Enumerate and rewrite resources

Resource tools can use `resources()` on either model type to enumerate direct
literal paths and replaceable IDs with typed source locations. `rewrite_resources()`
validates every proposed change before modifying the model and reports actual edits,
retaining chunk organization and unchanged bytes. The
[`resources` API](https://docs.rs/wc3/latest/wc3/model/formats/mdlx/resources/) includes an example.
File resolution, recursive discovery, path normalization, and packaging policy remain
application-owned, so enumeration includes PopcornFX and FaceFX even when a renderer
does not support them.

## Choose an output format

MDX retains chunk order and opaque payloads, while typed records are encoded
according to their layout. MDL writes canonical text, discarding comments and
source formatting. Its writer rejects binary data that the selected text dialect
cannot represent. A version conversion can succeed even when subsequent MDL
writing fails; field-loss policy does not relax text representability checks.

Use `encode_mdx()` or `encode_mdl()` to finish encoding in memory before writing
the result to a file. Standard I/O adapters do not flush their sinks, and transport
failures can leave partial output. The [MDX](https://docs.rs/wc3/latest/wc3/model/formats/mdlx/mdx/)
and [MDL](https://docs.rs/wc3/latest/wc3/model/formats/mdlx/mdl/) API references explain error
locations, dialect differences, and record-level streaming. The
[command-line guide](tools.md) covers the equivalent conversion and repath tools.
