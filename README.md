# wc3

`wc3` is a Rust library for working with Warcraft III assets. It reads, edits,
and writes models in binary **MDX** and text **MDL**, and textures in **BLP1**
and **BLP2** containers.

Models share one typed representation across MDX and MDL, with layouts for
Classic and Reforged versions. Texture APIs can inspect and rewrite encoded
BLP data without decoding pixels, or decode and encode images with optional
features.

## Getting started

The library crate is `wc3`. To use a local checkout, add it to your project's
`Cargo.toml` (adjust the path to your checkout):

```toml
[dependencies]
wc3 = { path = "../wc3/crates/wc3" }
```

Add `features = ["blp-decode", "blp-encode"]` to that dependency when you need
both BLP image operations. You can enable either feature on its own. Add a
direct `image = "0.25"` dependency if you use `image` types in your code, as
the examples below do.

| API | Cargo feature |
| --- | --- |
| MDX and MDL models; BLP container reading, editing, and writing | None |
| BLP mipmap decoding to RGBA pixels; `image::ImageDecoder` adapter | `blp-decode` |
| BLP image encoding from RGBA pixels; `image::ImageEncoder` adapter | `blp-encode` |

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
