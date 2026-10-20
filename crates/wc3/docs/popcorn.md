# Baked Popcorn effects

`model::formats::pkb` reads the native envelopes and compiled instruction streams
of version 2.5.1 PKB effects. Use it to inspect an external effect referenced by an
MDLX Popcorn feature. Reading a PKB does not simulate particles or make the effect
renderable in `bevy-wc3`.

## Inspect a document

The envelope borrows strings and object field bytes from the source. Type entries,
object order, flags, and uninterpreted payloads remain available, and `as_bytes()`
returns the original source without reconstructing unknown fields.

```rust
use wc3::model::formats::pkb::Document;

# fn inspect(bytes: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
let document = Document::read(bytes)?;
for object in document.objects() {
    let native_type = &document.types()[object.type_index as usize];
    let name = document.strings()[native_type.name as usize];
    println!("{}: {} fields", String::from_utf8_lossy(name), object.field_count);
}
# Ok(())
# }
```

Native object links are one-based and nullable; `document.object(link)` checks
them independently of the zero-based type and string tables. Strings retain their
original bytes, including empty or non-UTF-8 values. Unknown field widths require
a schema, so the envelope does not guess boundaries within an object payload.

## Inspect compiled programmes

`compiler_blobs()` discovers typed compiler-cache views in native object order.
Each view retains its stage identifier, external and function declaration links,
entrypoint link, and borrowed script bytes. A typed view rejects unknown cache
fields while the original document remains intact.

`CompilerBlob::script()` checks declared constant and instruction regions and
decodes the supported IR instruction family. Instructions retain byte positions,
packed swizzles, argument controls, signed receiver slots, and raw register words.
Register bank and scope bytes remain separate from external declaration type IDs.
Constant slots occupy 32 bytes; `Script::constant()` reads the first four component
words without discarding the remaining uninterpreted native words.

Unsupported envelope versions and instruction encodings return errors. The module
does not yet provide a scalar evaluator, named functions, samplers, layer events,
resource resolution, or rendering. Decoding an instruction does not establish its
execution or native-game fidelity.
