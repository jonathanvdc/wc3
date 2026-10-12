# wc3

This project provides Rust libraries for working with Blizzard assets, from
editing a model or converting a texture to displaying an animated character in
Bevy. It brings together support for MDX and MDL models, BLP textures, and MPQ
archives, so applications can work with these assets throughout the process of
loading, modifying, and rendering them. World of Warcraft M2 support covers
file formats and supporting documents through the asset library. WMO support
adds native world-model roots, geometry groups, and bundle validation. M3 support
covers StarCraft II and Heroes of the Storm model files and external animations.
WEM support adds indexed multi-model documents with profiles, shared resources,
and independently revised native records.

## Crates

Choose `wc3` for asset tools, `bevy-wc3` for Warcraft III model rendering, and
`bevy-mpq` for archive-backed Bevy assets. The fourth library crate, `wc3-derive`, supports
contributors defining model codecs. These crates can be used separately or
combined according to your application's needs.

[`wc3`](crates/wc3/README.md) is the starting point for applications that need to
read, edit, or write assets. It represents binary MDX and text MDL models using
the same Rust types, so model editing code can work with either format. Those
types cover geometry, materials, animation, and effects across Classic and
Reforged layouts, and the library can convert between supported model versions.
World of Warcraft M2 models have their own typed layouts and native supporting
file codecs, with explicit dependency resolution and coordinated bundle writing.
M3 documents retain indexed section graphs, independently versioned records,
packed geometry, and external animations. WMO documents expose their own root
and group layouts, including geometry, materials, portals, liquids, and collision
data. WEM documents hold several models, texture identities, and animation clips
in a native indexed graph; the [WEM guide](crates/wc3/docs/wem.md) explains typed
records, profile metadata, and preservation of unsupported payloads. Alongside
models, it provides BLP texture and MPQ archive APIs, with optional
features for working with decoded images and compressed archive entries. This
makes it useful for asset tools and converters as well as applications that
supply their own renderer.

[`wc3-derive`](crates/wc3-derive/README.md) helps define the model records that
`wc3` reads and writes. Its procedural macros describe how a Rust struct or enum
maps to MDX bytes or MDL text, including how fields, defaults, and animation
tracks are represented. Most applications use the existing record types and
never need to work with these macros directly. For contributors extending the
codecs or users defining additional records, the derives are available through
`wc3`'s MDX, MDL, M2, WMO, M3, and WEM modules alongside the traits they implement.

[`bevy-wc3`](crates/bevy-wc3/README.md) builds on the asset library to load and
render Warcraft III models in Bevy. It turns models and their textures into
scene instances with skinned geometry, animated sequences, and Classic or
Reforged materials, and supports scene features such as attachments, lights,
particles, and ribbon trails. A model viewer lets you explore assets, while an
offscreen capture example helps check their appearance at particular animation
times. The renderer is experimental, and its guides explain the current
limitations and which aspects of its appearance have been visually verified.

[`bevy-mpq`](crates/bevy-mpq/README.md) exposes MPQ archives as Bevy asset
readers. Applications configure named sources and compose archives with loose
files using an ordered overlay. The adapter is independent of the renderer;
`bevy-wc3` preserves the source when resolving a model's dependencies.

Each crate's README explains how to get started and links to further examples.
The [asset documentation index](crates/wc3/docs/README.md) connects MDLX, M2, WMO, M3, and WEM
editing, command-line tools, and the format APIs.
API references are available for [`wc3`](https://docs.rs/wc3/latest/wc3/) and
[`wc3-derive`](https://docs.rs/wc3-derive/latest/wc3_derive/), and the
[Bevy documentation index](crates/bevy-wc3/docs/README.md) collects the application
and rendering guides.

## Development

The crates are developed together in this Cargo workspace. To run their tests
with the optional texture and archive codecs enabled, use:

```sh
cargo test --workspace --all-features
```

Use the Rust toolchain pinned in `rust-toolchain.toml` for local checks. Rustdoc
examples are included in `cargo test`; to check documentation links separately,
run `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps`.

Criterion suites measure the formats, animation, and Bevy integration with
self-contained synthetic inputs. Run `bash benchmarks/run-quick.sh` for the
representative optimized measurements used on every CI push and pull request.
The [benchmark guide](benchmarks/README.md) explains targeted runs, smoke checks,
scaling cases, baselines, and host GPU measurements.

You can also build and browse the API documentation locally. Enabling all
features includes the optional image and archive APIs in the generated docs:

```sh
cargo doc --workspace --all-features --no-deps --open
```
