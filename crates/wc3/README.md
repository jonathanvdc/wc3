# wc3

`wc3` is a Rust library for working with Blizzard model assets, BLP and Diablo
TEX textures, and MPQ archives. It provides types and codecs for
reading assets, changing their contents, and writing them back, whether you are
building an asset editor, a converter, or an application that loads game data. Binary MDX and text MDL models
share the same typed representation, allowing model editing code to work with
both formats and with supported Classic and Reforged layouts. World of Warcraft
M2 documents use a separate typed format API, including their supporting skin,
geometry, animation, skeleton, bone, and physics files. WMO roots and groups
have their own typed layouts and bundle API for static world geometry. M3 models
and external animations retain their indexed section graphs.
Diablo III and IV resources use independently typed SNO document layouts, with
external payloads supplied by the caller.

## Capabilities

The library covers three kinds of assets. Models can be edited through typed
records, while textures and archives can be inspected and rewritten without
first decoding their image or file contents. Optional features add pixel
operations and archive compression when your application needs them.

| Assets | Operations |
| --- | --- |
| Warcraft III MDX and MDL models | Read, edit, write, and convert between supported model versions. |
| World of Warcraft M2 models | Read, edit, and write typed core layouts and supporting files; resolve and rebuild model bundles. |
| World of Warcraft WMO models | Read, edit, and write root and group documents; resolve native geometry bundles. |
| StarCraft II and Heroes of the Storm M3 models | Read, edit, and write indexed model and external animation documents. |
| Diablo III and IV model resources | Read, edit, and rebuild supported typed appearance, actor, animation, and material documents. |
| BLP0, BLP1, and BLP2 textures | Inspect and edit encoded containers; optionally decode pixels and encode images. |
| Diablo III and IV TEX textures | Read, edit, and write typed containers; optionally decode and encode supported pixels. |
| MPQ v1–v4 archives | Index, extract, create, and edit archives; optionally decompress and compress entries. |

Native model APIs describe encoded records and supporting documents. For
independent authoring values, use [semantic model graphs](docs/semantic.md):
typed vocabularies, graph relationships, shared resource ownership, and checked
edits. The [conversion guide](docs/conversion.md) connects those graphs to native
imports and export plans, with explicit representability policies.

For Warcraft III model loading, animation, and rendering in Bevy, see the
companion [`bevy-wc3`](../bevy-wc3/README.md) crate.

## Getting started

Add the library to your project with Cargo. The default configuration includes
model codecs and the texture and archive container APIs; you can enable image
and compression features separately as described below.

```sh
cargo add wc3
```

### Warcraft III models

MDX and MDL model types live under `wc3::model::mdlx`, where the `mdx` and `mdl`
modules provide the codecs for each format. The example below reads a small MDL
model, changes its name, and writes the result in both formats. Importing the codec traits as
`_` makes their methods available without introducing conflicting trait names.

```rust
use wc3::model::mdlx::{Model, V800};
use wc3::model::mdlx::mdl::{Read as _, Write as _};
use wc3::model::mdlx::mdx::Write as _;

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
[`mdlx` module documentation](https://docs.rs/wc3/latest/wc3/model/mdlx/) explains
how to edit those collections and convert between supported versions.

Continue with the [model editing guide](docs/models.md) for runtime version
selection, collection editing, borrowed buffers, conversion policies, and resource
rewriting. The API reference supplies the record types and codec contracts.

### World of Warcraft models

M2 types live under `wc3::model::m2`. Choose a numeric layout and either the flat
MD20 or chunked MD21 envelope, or use `DynamicModel` to detect both from bytes:

```rust
use wc3::model::m2::{Chunked, Model, V274};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::<Chunked<V274>>::new();
    let bytes = model.encode_m2()?;
    let decoded = Model::<Chunked<V274>>::decode_m2(&bytes)?;
    assert_eq!(decoded.version(), 274);
    Ok(())
}
```

The core exposes native records and borrowed encoded arrays. Standalone
`skin`, `geometry`, `animation`, `skeleton`, `bone`, and `physics` modules handle
supporting files; `Bundle` connects declared references through a caller-owned resolver and
coordinates external animation offsets when writing. The [M2 guide](docs/m2.md)
explains layout selection, ownership, complete bundle output, and preservation
boundaries. M2 rendering is not implemented.

World geometry uses `wc3::model::wmo::{Root, Group, Bundle}` with an explicit
client profile, since WMO's version field does not identify every layout change.
The [WMO guide](docs/wmo.md) explains borrowed chunk records, dependency
resolution, validation, and coordinated root/group output.

StarCraft II and Heroes of the Storm models use `wc3::model::m3`. The MD33 and
MD34 envelopes select reference widths, while each indexed section selects its
own record revision. The [M3 guide](docs/m3.md) explains borrowed section graphs,
typed packed vertices, shared references, external animations, and preservation
limits for unknown payloads.

### Diablo models

Diablo resources live under `wc3::model::diablo::{d3, d4}`. D3 document families
select their own numeric revision: an appearance revision does not select an
animation or material revision. D4 uses exact format hashes to select concrete
root layouts. Both families resolve borrowed native arrays and relocate their
references when writing. D4 documents can return a root file and a separate
payload body, leaving storage naming and lookup to the application. The
[Diablo guide](docs/diablo.md) describes the supported layouts and the boundary
between native records and semantic conversion. Diablo rendering is not implemented.

### Textures

For BLP textures, you can choose whether to work with the encoded container or
with decoded pixels. `Blp::read` borrows encoded content from your input buffer
without copying it. The same `Blp<'a>` container holds borrowed or owned bytes;
its payloads use copy-on-write storage so editing copies only borrowed data
that you change. `as_ref()` creates a borrowed view without copying buffers,
while `into_owned()` moves owned buffers and copies borrowed bytes into an
independent `Blp<'static>`. This is enough for operations that do not need to
inspect the image itself.

BLP1 and BLP2 contain their mipmaps in the root file. BLP0 uses sibling `.b00`,
`.b01`, and subsequent files; parsing its root leaves empty mipmap slots that
callers fill explicitly. `write_files()` returns the root and attached external
payloads, while single-file `write()` rejects BLP0.

When you need pixels, the `blp-decode` feature lets you decode mipmaps to RGBA
images. The corresponding `blp-encode` feature creates BLP textures from images,
using either generated or authored mipmaps. Both integrate with the `image`
crate, which you should also add as a direct dependency if your code uses its
types. The [`blp` module documentation](https://docs.rs/wc3/latest/wc3/blp/)
provides examples and explains the available encoding options. `EncodeOptions`
lets you choose the filter for generated mipmaps and the DXT fitting algorithm,
with Triangle resampling and ClusterFit compression as defaults. Applications
that author their own mipmaps can supply those levels directly.

Diablo TEX textures use `wc3::tex::{d3, d4}` with typed native layouts.
Encoded operations require no image features. Enable `tex-decode` or `tex-encode`
for supported pixel conversions, which reuse existing block-compression libraries.
D3 embeds mip payloads; D4 metadata addresses separately supplied payloads.
The [TEX guide](docs/tex.md) explains payload selection, supported formats, and
preservation limits.

### Archives

Use `Archive` to read entries from a seekable source, or `SharedArchive` when
several entries need to be read concurrently. Both index the archive when it is
opened and decode payloads on demand. `SharedArchive` accepts files, shared
memory, and custom random-access sources. [`bevy-mpq`](../bevy-mpq/README.md)
integrates archives with Bevy asset sources.

`ArchiveWriter` creates archives and edits copies of existing archives. It can
preserve encoded entries, avoiding decompression and recompression of files you
have not changed. Writing requires a seekable destination.

To assemble a compact new archive, `copy_file_from` imports an unencrypted
entry's encoded payload into an `ArchiveWriter`. The lower-level
`open_encoded_file` and `add_encoded_file` APIs also support renamed entries
and standalone encoded caches. Encoded copying does not require compression
features; it preserves the payload without validating its decoded contents.

Compressed entries need the `mpq-decode` feature for extraction, and
`mpq-encode` enables compression when writing. The
[`mpq` module documentation](https://docs.rs/wc3/latest/wc3/mpq/) walks through
the API and describes format support, editing behavior, and archive recovery.

## Optional features

The crate has no default features, so applications that only work with models
or encoded containers do not pull in image or compression dependencies. Model
codecs, BLP/TEX container operations, and MPQ indexing, stored entries, encryption,
and encoded archive editing are always available. The following features add
the operations that require decoding or encoding asset contents:

| Feature | Adds |
| --- | --- |
| `blp-decode` | BLP mipmap decoding to RGBA pixels and an `image::ImageDecoder` adapter. |
| `blp-encode` | BLP image encoding and an `image::ImageEncoder` adapter. |
| `tex-decode` | Supported D3/D4 TEX mipmap decoding to RGBA pixels. |
| `tex-encode` | Supported TEX mip encoding through existing pixel codecs. |
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

For Warcraft III models, MDX preserves chunk order and opaque data, while MDL
produces canonical text and reports data it cannot faithfully represent. MDL source
formatting is therefore discarded when writing a model back to text. Readers
accept both Warcraft III and Hive Workshop spellings, and writers use Warcraft
III syntax by default. The [MDX](https://docs.rs/wc3/latest/wc3/model/mdlx/mdx/) and
[MDL](https://docs.rs/wc3/latest/wc3/model/mdlx/mdl/) documentation explains these
behaviors in more detail.

Converting between MDLX model versions returns a model and a report. `convert`
leaves its source intact; `into_convert` consumes it and moves unchanged chunks.
Strict conversion rejects unsupported nondefault data and opaque chunks when
changing versions. Lossy conversion permits unsupported fields to be dropped;
opaque chunks still require a separate policy. Inspect the report even after a
successful conversion, since it also describes normalization and defaults.
Because some Classic and Reforged features have no equivalent in another layout,
conversion cannot guarantee identical rendering in the game.

M2 writing relocates understood offset-addressed data and retains unknown chunk
payloads. `Bundle::encode()` updates external animation streams and their owner
records together. AFRA, classic lookup arrays, and the version-257 extra array
remain uninterpreted; format coverage and broad game compatibility are separate
concerns. See the
[M2 guide](docs/m2.md#coverage-and-preservation-boundaries) for the supported
layouts, chunk catalogue, and limits.

BLP container writing canonicalizes layout rather than preserving arbitrary
padding. BLP0 indexed and JPEG parsing, encoding, and external mipmap assembly
are implemented, but compatibility with real beta-game textures remains
unverified. The generic `image` decoder hook cannot resolve BLP0 sidecars, and
its single-writer encoder cannot output them; use the explicit container APIs.

Archive support also has limits: MPQ patch-file application and signature
verification are not implemented. The
[archive documentation](https://docs.rs/wc3/latest/wc3/mpq/) describes the
compatibility limits and integrity checks that apply to reading and editing.

## Documentation and examples

The API reference provides examples for each asset type, and the repository
includes command-line examples for inspecting, editing, and converting assets.
The model command-line examples below operate on Warcraft III MDX and MDL:

- [Documentation index](docs/README.md): MDLX, M2, WMO, M3, Diablo, and TEX workflows, command-line tools,
  and format-specific API references.
- [API reference](https://docs.rs/wc3/latest/wc3/).
- [`mpq` example](examples/mpq.rs): archive listing, extraction, creation, and editing.
- [`mpq_concurrent` example](examples/mpq_concurrent.rs): a synthetic benchmark comparing serialized and concurrent extraction; requires `mpq-decode` and `mpq-encode`.
- [`diablo_info` example](examples/diablo_info.rs): typed D3/D4 SNO metadata and direct dependency identities.
- [`tex_to_png` example](examples/tex_to_png.rs): selected Diablo TEX pixel export; requires `tex-decode`.
- [`model_dependencies` example](examples/model_dependencies.rs): deduplicated direct file references with their locations, plus replaceable resource IDs.
- [`model_info` example](examples/model_info.rs): model metadata, geometry and emitter counts, and animation intervals.
- [`model_convert` example](examples/model_convert.rs): MDX/MDL and model-version conversion with loss reports.
- [`model_repath` example](examples/model_repath.rs): literal, case-sensitive prefix replacement in declared asset paths.
- [`blp_to_png` example](examples/blp_to_png.rs): texture metadata and PNG export of a selected mip level; requires `blp-decode`.
- [`mdlx_compare` example](examples/mdlx_compare.rs): model round-trip comparisons.

The [command-line guide](docs/tools.md) collects invocation examples and explains
format detection, dependency reporting, repathing, and conversion losses. Each
command-line tool also provides `--help` for its options.

From a workspace checkout, build the crate documentation with all optional APIs:

```sh
cargo doc -p wc3 --all-features --no-deps --open
```
