# wc3

Read, edit, and write Warcraft III and World of Warcraft models, BLP textures,
and MPQ archives in Rust. The library supports asset tools and offline processing
without a renderer. For displaying models in Bevy, use
[`bevy-wc3`](../bevy-wc3/README.md).

```sh
cargo add wc3
```

Model codecs and encoded texture/archive operations are always available.
[Optional features](#optional-features) add pixel decoding and archive compression.

## Read and edit a model

There are two ways to work with models. Use `wc3::model::Model` for geometry,
materials, animation, and relationships that are checked together when edited.
Use native documents under `wc3::model::formats` when your task depends on file
records, version-specific fields, or preservation of unknown data.

For a self-contained Warcraft III model, decode MDX or MDL directly into the
checked representation:

```rust
use wc3::model::conversion::ImportOptions;
use wc3::model::io;

let source = r#"Version { FormatVersion 800, } Model "Example" {}"#;
let imported = io::mdlx::decode_mdl(source, &ImportOptions::default())?;
assert_eq!(imported.value.name(), "Example");
# Ok::<(), wc3::model::io::mdlx::ReadError>(())
```

The returned model owns its values, so you can release the input. Import also
returns a report. Strict import rejects unsupported data; explicitly permitted
losses are recorded there. The [model guide](docs/semantic.md) introduces checked
editing, and the [import and export guide](docs/conversion.md) shows how to plan
native output and inspect conversion decisions.

For M2 models with companions, WMO roots with groups, or a common ingestion path
across formats, use `io::asset::Reader`. It requests supporting files by typed
identity and returns an owned, checked Universal model with its conversion report.
Your application controls source reads and import policies; the completed graph
can outlive every source buffer and native document. The
[model loading guide](docs/loading.md) shows the complete workflow.

For a native edit, decode a document, change its records, and encode it again.
MDX and MDL share the same document types; import their codec traits as `_` to
make the methods available:

```rust
use wc3::model::formats::mdlx::{Document, V800};
use wc3::model::formats::mdlx::mdl::{Read as _, Write as _};
use wc3::model::formats::mdlx::mdx::Write as _;

let source = r#"Version { FormatVersion 800, } Model "Example" {}"#;
let mut model = Document::<V800>::decode_mdl(source)?;
let mut info = model.model_info().unwrap();
info.name.set_text("Renamed")?;
model.set_model_info(&info);

let mdx_bytes = model.encode_mdx()?;
let mdl_text = model.encode_mdl()?;
assert!(!mdx_bytes.is_empty());
assert!(mdl_text.contains("Renamed"));
# Ok::<(), Box<dyn std::error::Error>>(())
```

`Document<V800>` selects a known layout. Use `DynamicDocument` when the file's
version is determined at runtime. The [native MDLX guide](docs/models.md) covers
version selection, collection edits, resource paths, and version conversion.

## Work with other assets

The same crate includes WoW models, textures, and archives. Each has its own
native API and output guarantees:

| Asset | Start with | Guide |
| --- | --- | --- |
| WoW M2 models | `model::formats::m2::{Document, DynamicDocument, Bundle}` | [M2 models and supporting files](docs/m2.md) |
| WoW WMO worlds | `model::formats::wmo::{Root, Group, Bundle}` | [WMO roots and groups](docs/wmo.md) |
| BLP0, BLP1, BLP2 textures | `blp::Blp` | [BLP API](https://docs.rs/wc3/latest/wc3/blp/) |
| MPQ v1–v4 archives | `mpq::{Archive, SharedArchive, ArchiveWriter}` | [MPQ API](https://docs.rs/wc3/latest/wc3/mpq/) |

M2 bundles connect models to skins, animation, skeleton, and other supporting
files through a caller-provided resolver. WMO bundles connect roots and groups;
an explicit client profile selects layout details that the file version alone
cannot identify. Their guides explain coordinated output and preservation limits.

BLP containers let you inspect or rewrite encoded mipmaps without decoding
pixels. BLP0 stores mipmaps in sibling files; attach those payloads explicitly
and use `write_files()` for output. Enable the pixel features below when you need
RGBA images or integration with the `image` crate.

Use `Archive` for seekable archive input or `SharedArchive` for concurrent reads.
`ArchiveWriter` creates archives and edits copies of existing archives, including
copying unchanged encoded entries without recompression. The
[`bevy-mpq`](../bevy-mpq/README.md) adapter makes archives available to Bevy.

## Optional features

There are no default features. Enable only the pixel and compression operations
your application needs:

| Feature | Adds |
| --- | --- |
| `blp-decode` | RGBA mipmap decoding and an `image::ImageDecoder` adapter. |
| `blp-encode` | Image encoding and an `image::ImageEncoder` adapter. |
| `mpq-decode` | Compressed archive entry extraction. |
| `mpq-encode` | Zlib and bzip2 compression when writing archives. |

For example, to read texture pixels and extract compressed archive entries:

```sh
cargo add wc3 --features blp-decode,mpq-decode
```

Add `image` as a direct dependency if your application names its types. Stored
MPQ entries, encryption, archive indexing, and encoded copying need no optional
features.

## Choose the right output guarantees

Native editing and checked model conversion serve different purposes. Native
MDX documents preserve chunk order and opaque data. MDL writing produces
canonical text and rejects data it cannot represent; it does not preserve source
formatting. Checked model imports retain supported meanings rather than a copy
of the source file. Keep the native document if you also need its unknown data.

Conversion is strict by default. When changing versions or exporting a checked
model, inspect the report and explicitly choose any permitted loss. Successful
conversion establishes what the selected format can represent; it does not
establish identical appearance in the game.

M2 and WMO support varies by layout and record family; consult their guides
before relying on preservation. BLP writing canonicalizes container layout, and
real beta-game compatibility of BLP0 remains unverified. MPQ patch-delta
application and signature verification are not implemented. Finish encoding or
archive finalization before replacing a source file, since transport errors can
leave partial output.

## Continue with a task

The [documentation index](docs/README.md) connects model editing, CPU evaluation,
native formats, and command-line workflows. To try the tools from a checkout:

```sh
cargo run -p wc3 --example model_info -- path/to/model.mdx
cargo run -p wc3 --features blp-decode --example blp_to_png -- texture.blp texture.png
```

The [tool guide](docs/tools.md) covers the other commands and their write policies.
For complete types and contracts, browse the [API reference](https://docs.rs/wc3/latest/wc3/)
or build it locally with `cargo doc -p wc3 --all-features --no-deps --open`.
