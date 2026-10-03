# wc3

`wc3` is a Rust library for working with Warcraft III models, textures, and
archives. It provides types and codecs for reading assets, changing their
contents, and writing them back, whether you are building an asset editor, a
converter, or an application that loads game data. Binary MDX and text MDL models
share the same typed representation, allowing model editing code to work with
both formats and with supported Classic and Reforged layouts.

## Capabilities

The library covers three kinds of assets. Models can be edited through typed
records, while textures and archives can be inspected and rewritten without
first decoding their image or file contents. Optional features add pixel
operations and archive compression when your application needs them.

| Assets | Operations |
| --- | --- |
| MDX and MDL models | Read, edit, write, and convert between supported model versions. |
| BLP1 and BLP2 textures | Inspect and edit encoded containers; optionally decode pixels and encode images. |
| MPQ v1–v4 archives | Index, extract, create, and edit archives; optionally decompress and compress entries. |

For model loading, animation, and rendering in Bevy, see the companion
[`bevy-wc3`](../bevy-wc3/README.md) crate.

## Getting started

Add the library to your project with Cargo. The default configuration includes
model codecs and the texture and archive container APIs; you can enable image
and compression features separately as described below.

```sh
cargo add wc3
```

### Models

Model types live under `wc3::model`, where the `mdx` and `mdl` modules provide
the codecs for each format. The example below reads a small MDL model, changes
its name, and writes the result in both formats. Importing the codec traits as
`_` makes their methods available without introducing conflicting trait names.

```rust
use wc3::model::{Model, V800};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::Write as _;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"Version { FormatVersion 800, } Model "Example" {}"#;
    let mut model = Model::<V800>::decode_mdl(source)?;
    let mut info = model.model_info().unwrap();
    info.name.set_text("Renamed")?;
    model.set_model_info(&info);

    let mdx_bytes = model.encode_mdx()?;
    let mdl_text = model.encode_mdl()?;
    assert!(!mdx_bytes.is_empty());
    assert!(mdl_text.contains("Renamed"));
    Ok(())
}
```

The example uses `Model<V800>` because its input has a known format version.
When the version needs to be determined from the file, `DynamicModel` provides
the same model operations with runtime version selection. Once a model is
loaded, its records give you access to geometry, materials, scene nodes,
animation, and effects. The
[`model` module documentation](https://docs.rs/wc3/latest/wc3/model/) explains
how to edit those collections and convert between supported versions.

### Textures

For BLP textures, you can choose whether to work with the encoded container or
with decoded pixels. `BlpRef` borrows the encoded mipmaps from your input buffer;
converting it to an owned `Blp` lets you change the container and write it back.
This is enough for operations that do not need to inspect the image itself.

When you need pixels, the `blp-decode` feature lets you decode mipmaps to RGBA
images. The corresponding `blp-encode` feature creates BLP textures from images,
using either generated or authored mipmaps. Both integrate with the `image`
crate, which you should also add as a direct dependency if your code uses its
types. The [`blp` module documentation](https://docs.rs/wc3/latest/wc3/blp/)
provides examples and explains the available encoding options.

### Archives

The MPQ API lets you open an archive and read its entries on demand through
`Archive`, or create an archive through `ArchiveWriter`. You can also edit a
copy of an existing archive while preserving its original encoded entries,
which avoids decoding and recompressing files you have not changed. These
operations require sources and destinations that support seeking.

Compressed entries need the `mpq-decode` feature for extraction, and
`mpq-encode` enables compression when writing. The
[`mpq` module documentation](https://docs.rs/wc3/latest/wc3/mpq/) walks through
the API and describes format support, editing behavior, and archive recovery.

## Optional features

The crate has no default features, so applications that only work with models
or encoded containers do not pull in image or compression dependencies. Model
codecs, BLP container operations, and MPQ indexing, stored entries, encryption,
and encoded archive editing are always available. The following features add
the operations that require decoding or encoding asset contents:

| Feature | Adds |
| --- | --- |
| `blp-decode` | BLP mipmap decoding to RGBA pixels and an `image::ImageDecoder` adapter. |
| `blp-encode` | BLP image encoding and an `image::ImageEncoder` adapter. |
| `mpq-decode` | MPQ decompression. |
| `mpq-encode` | MPQ zlib and bzip2 compression. |

You can enable these features independently. For example, an application that
both reads and creates texture images can enable the two BLP features together:

```sh
cargo add wc3 --features blp-decode,blp-encode
```

## Format behavior and limitations

Reading and writing an asset does not always preserve its original bytes or
appearance. The codecs and conversion APIs make different guarantees depending
on the format and operation.

For models, MDX preserves chunk order and opaque data, while MDL produces
canonical text and reports data it cannot faithfully represent. MDL source
formatting is therefore discarded when writing a model back to text. Readers
accept both Warcraft III and Hive Workshop spellings, and writers use Warcraft
III syntax by default. The [MDX](https://docs.rs/wc3/latest/wc3/model/mdx/) and
[MDL](https://docs.rs/wc3/latest/wc3/model/mdl/) documentation explains these
behaviors in more detail.

Converting between model versions creates a new model and returns a report,
leaving the source intact. Strict conversion rejects unsupported nondefault
data; you can opt into lossy conversion to allow that data to be dropped.
Because some Classic and Reforged features have no equivalent in another layout,
conversion cannot guarantee identical rendering in the game.

Archive support also has limits: MPQ patch-file application and signature
verification are not implemented. The
[archive documentation](https://docs.rs/wc3/latest/wc3/mpq/) describes the
compatibility limits and integrity checks that apply to reading and editing.

## Documentation and examples

The API reference provides examples for each asset type, and the repository
includes command-line examples for inspecting, editing, and converting assets:

- [API reference](https://docs.rs/wc3/latest/wc3/).
- [`mpq` example](examples/mpq.rs): archive listing, extraction, creation, and editing.
- [`model_dependencies` example](examples/model_dependencies.rs): deduplicated direct file references with their locations, plus replaceable resource IDs.
- [`model_info` example](examples/model_info.rs): model metadata, geometry and emitter counts, and animation intervals.
- [`model_convert` example](examples/model_convert.rs): MDX/MDL and model-version conversion with loss reports.
- [`model_repath` example](examples/model_repath.rs): literal, case-sensitive prefix replacement in declared asset paths.
- [`blp_to_png` example](examples/blp_to_png.rs): texture metadata and PNG export of a selected mip level; requires `blp-decode`.
- [`mdlx_compare` example](examples/mdlx_compare.rs): model round-trip comparisons.

The [command-line guide](docs/tools.md) collects invocation examples and explains
format detection, dependency reporting, repathing, and conversion losses. Each
example also provides `--help` for its options.

From a workspace checkout, build the crate documentation with all optional APIs:

```sh
cargo doc -p wc3 --all-features --no-deps --open
```
