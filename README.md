# wc3

`wc3` is a Rust library for working with Warcraft III assets. It reads, edits,
and writes models in binary **MDX** and text **MDL**, and textures in **BLP1**
and **BLP2** containers, and **MPQ v1–v4** archives.

Models share one typed representation across MDX and MDL, with layouts for
Classic and Reforged versions. Texture APIs can inspect and rewrite encoded
BLP data without decoding pixels, or decode and encode images with optional
features.

The workspace also contains an experimental [Bevy renderer](crates/bevy-wc3/README.md)
with an animated MDX viewer.

## Getting started

The library crate is `wc3`. Add the latest published version to your project:

```sh
cargo add wc3
```

Enable both BLP image operations with
`cargo add wc3 --features blp-decode,blp-encode`. You can enable either feature
on its own. Add a direct `image = "0.25"` dependency if you use `image` types in
your code, as the examples below do.

| API | Cargo feature |
| --- | --- |
| MDX and MDL models; BLP container reading, editing, and writing | None |
| BLP mipmap decoding to RGBA pixels; `image::ImageDecoder` adapter | `blp-decode` |
| BLP image encoding from RGBA pixels; `image::ImageEncoder` adapter | `blp-encode` |
| MPQ indexing, stored entries, encryption, creation, and encoded archive editing | None |
| MPQ zlib, bzip2, PKWARE DCL, sparse, Huffman, ADPCM, and LZMA decompression | `mpq-decode` |
| MPQ zlib and bzip2 compression | `mpq-encode` |

### Models: read, edit, and write

Model types live under `wc3::model`. Import the relevant `mdx::Read` /
`mdx::Write` and `mdl::Read` / `mdl::Write` traits as `_` to enable their
methods.

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

Use `Model<V>` when you know the version. `DynamicModel` selects the version
from an MDL file or an MDX `VERS` chunk; MDX reading takes a fallback version
for files without that chunk. Supported versions are **800, 900, 1000, 1100,
1200, 1300, 1400, 1600, and 1800**.

MDX preserves chunk order and opaque data. MDL writes canonical text and
reports data it cannot faithfully represent. MDL readers accept Warcraft III
and Hive Workshop spellings; writers use Warcraft III syntax by default.
See the [`model`](https://docs.rs/wc3/latest/wc3/model/) and
[`mdl`](https://docs.rs/wc3/latest/wc3/model/mdl/) module docs for editing
collections, dialects, and field-level coverage.

### Textures: inspect, decode, and encode

`BlpRef` borrows encoded mipmaps from the input buffer. Convert it to an owned
`Blp` to edit container fields, then write it back:

```rust
use wc3::blp::{Blp, BlpRef};

fn update(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut texture = BlpRef::read(bytes)?.to_owned();
    if let Blp::Blp1(blp1) = &mut texture {
        blp1.header.extra = 5;
    }
    Ok(texture.write()?)
}
```

Writing recalculates mipmap offsets and discards source padding. With
`blp-decode`, decode any BLP1 or BLP2 mipmap to an `image::RgbaImage`:

```rust
use wc3::blp::BlpRef;

fn decode(bytes: &[u8]) -> Result<image::RgbaImage, Box<dyn std::error::Error>> {
    Ok(BlpRef::read(bytes)?.decode_mip(0)?)
}
```

With `blp-encode`, encode an RGBA image to BLP JPEG, indexed colour with 0, 1,
4, or 8-bit alpha, DXT1/3/5, or uncompressed BGRA:

```rust
use wc3::blp::{Blp, BlpVersion, EncodeFormat, EncodeOptions, IndexedAlpha};

fn encode(image: &image::RgbaImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let options = EncodeOptions {
        format: EncodeFormat::Indexed {
            version: BlpVersion::Blp1,
            alpha: IndexedAlpha::Bit8,
        },
        ..Default::default()
    };
    Ok(Blp::encode_image(image, options)?.write()?)
}
```

`Blp::encode_image` generates mipmaps; `Blp::encode_mipmaps` accepts authored
mipmaps. See the [`blp`](https://docs.rs/wc3/latest/wc3/blp/) module docs for
the `image` crate adapters, options, and container behavior.

### Archives: stream, create, and edit

MPQ support lives under `wc3::mpq`. Enable compressed extraction and writing
with `cargo add wc3 --features mpq-decode,mpq-encode`. Compression uses optional
Rust libraries; ordinary model/texture users gain no compression dependencies.

```rust
use std::io::{Cursor, Read};
use wc3::mpq::{Archive, ArchiveWriter, FileOptions, WriteOptions};

fn example() -> Result<(), wc3::mpq::Error> {
    let mut writer = ArchiveWriter::new(Cursor::new(Vec::new()), WriteOptions::default())?;
    writer.add_file("war3map.j", 5, &mut b"hello".as_slice(), FileOptions::default())?;
    let bytes = writer.finish()?.into_inner();

    let mut archive = Archive::open(Cursor::new(bytes))?;
    let mut entry = archive.open_file("war3map.j")?;
    let mut text = String::new();
    entry.read_to_string(&mut text)?;
    assert_eq!(text, "hello");
    Ok(())
}
```

Sources require `Read + Seek`, sinks `Write + Seek`. Entry readers implement
`Read`; `start_file` returns a `Write` entry sink. Provide the decoded entry
size up front and call `finish` on both entry and archive writers. Sector files
use bounded payload memory; single-unit input files use a configurable buffer
limit. Index and sector tables remain in memory. Sector checksums are verified
by default, and `FileOptions::sector_checksums` enables writing them.

`ArchiveWriter::from_archive` creates an editable copy without decoding original
files. `replace_file` and `remove_file` change exact filename/locale/platform
matches. Original hash slots and relative block offsets remain intact, preserving
unnamed entries, unknown codecs, and adjusted encryption keys. Edits append data;
they do not compact or grow the original hash table. The original `(listfile)`
is preserved unless explicitly replaced, and map prefixes/user-data wrappers
outside the MPQ region are not copied. Use a separate output file for edits.

The reader discovers MPQ v1–v4 headers after map prefixes and user-data wrappers.
It supports 64-bit offsets, extended block tables, compressed index tables, and
HET/BET lookup, including archives without classic tables. Compressed tables
require `mpq-decode`. V4 header/table MD5s are verified when present; raw file
chunk MD5s are verified on opening an entry. HET/BET lookup uses Jenkins hashes
and supports neutral locale/platform; classic tables retain exact locale lookup.

Writers default to classic headers. Set `WriteOptions::header_version` to 1, 2,
or 3 for MPQ v2, v3, or v4. `extended_index` adds HET/BET tables for v3/v4;
`raw_chunk_size` enables v4 chunk MD5s (zero disables them). New index tables
are written without compression. Writers bound raw chunks and digest buffers
to 16 MiB each; payloads continue to stream. Encoded editing preserves the
source header version, extended filename hashes, raw chunk settings, and file
offsets, including files whose names are unknown. HET/BET edits compact block
records and remap IDs; payload space is retained.

Filename lookup follows MPQ byte hashing, ASCII case folding, and slash
normalization. Locale/platform matching is exact, neutral by default.
`known_names` returns listfile hints; it cannot enumerate every original name.
`index` and `encoded_file` work independently of payload codec support.

`ReadOptions { mode: ReadMode::Permissive, ..Default::default() }` opts into
bounded recovery of malformed classic MPQs. Strict mode remains the default.
Permissive mode handles misleading classic header fields, table-count and
block-index high bits, reserved hash bytes, truncated tables, invalid unrelated
records, fake headers and user-data wrappers, and wrapped 32-bit table/block/
sector addresses. It tolerates extra sector-table bytes and unusable checksum
metadata; actual checksum mismatches still fail. Allocation, decompression, and
physical source bounds remain enforced. Valid newer-format archives retain their
strict indexing and integrity checks; malformed older headers may fall back to
classic interpretation, while v4 integrity failures are never downgraded.

`open_file_by_index` extracts a block without its original filename. In
permissive mode, encrypted sector tables can supply a recoverable key; stored
RIFF, EXE, and XML signatures also permit limited key recovery. This does not
recover arbitrary encryption keys, missing filenames, or deleted editor data.
`diagnostics()` reports recoveries as they occur, including when opening a file;
it retains at most 1024 records plus an omission notice. Recovered archives
cannot be passed to `ArchiveWriter::from_archive`; clean permissive archives can.
Synthetic tests cover these recovery cases. Compatibility with a corpus of
protected game maps has not been verified.

Patch-file application and signature verification are unsupported.
Archive edits do not regenerate existing attributes or signatures. Game loading
has not been verified. Automated tests cover the public API, binary format
contracts, encryption, compression, corruption handling, and streaming limits;
newer-format tests also use tiny synthetic StormLib fixtures. Running the tests
does not require StormLib.

The `mpq` example supplies streaming `list`, `extract`, `create`, and `edit`
commands. Run it without arguments for usage:

```sh
cargo run -p wc3 --example mpq --features mpq-decode,mpq-encode -- \
  create /tmp/assets.mpq zlib adjusted \
  'Units\\Example.mdx=crates/wc3/tests/fixtures/mdl/quad_model.mdx'
```

## Version conversion

`Model::convert` builds a model for another version and returns a report,
leaving the source intact. Strict conversion rejects unsupported nondefault
data; `ConversionOptions::lossy()` permits discarding it. Unknown chunks have
a separate preserve or drop policy. Conversion cannot translate every feature
between Classic and Reforged layouts and does not guarantee identical game
rendering. See the [`model`](https://docs.rs/wc3/latest/wc3/model/) module
docs for the conversion example and editing guidance.

## API documentation

The [`mdx`](https://docs.rs/wc3/latest/wc3/model/mdx/) and
[`mdl`](https://docs.rs/wc3/latest/wc3/model/mdl/) modules document their
`Read` and `Write` traits, format behavior, and lower-level codecs. Build all
workspace API docs locally with:

```sh
cargo doc --workspace --no-deps --open
```
