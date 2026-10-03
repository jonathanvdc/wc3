# wc3

This project provides Rust libraries for working with Warcraft III assets, from
editing a model or converting a texture to displaying an animated character in
Bevy. It brings together support for MDX and MDL models, BLP textures, and MPQ
archives, so applications can work with these assets throughout the process of
loading, modifying, and rendering them.

## Crates

The workspace contains three crates. The asset library can be used on its own,
while the derive and Bevy crates support custom model codecs and rendering,
respectively.

[`wc3`](crates/wc3/README.md) is the starting point for applications that need to
read, edit, or write assets. It represents binary MDX and text MDL models using
the same Rust types, so model editing code can work with either format. Those
types cover geometry, materials, animation, and effects across Classic and
Reforged layouts, and the library can convert between supported model versions.
Alongside models, it provides BLP texture and MPQ archive APIs, with optional
features for working with decoded images and compressed archive entries. This
makes it useful for asset tools and converters as well as applications that
supply their own renderer.

[`wc3-derive`](crates/wc3-derive/README.md) helps define the model records that
`wc3` reads and writes. Its procedural macros describe how a Rust struct or enum
maps to MDX bytes or MDL text, including how fields, defaults, and animation
tracks are represented. Most applications use the existing record types and
never need to work with these macros directly. For contributors extending the
codecs or users defining additional records, the derives are available through
`wc3`'s MDX and MDL modules alongside the traits they implement.

[`bevy-wc3`](crates/bevy-wc3/README.md) builds on the asset library to load and
render Warcraft III models in Bevy. It turns models and their textures into
scene instances with skinned geometry, animated sequences, and Classic or
Reforged materials, and supports scene features such as attachments, lights,
particles, and ribbon trails. A model viewer lets you explore assets, while an
offscreen capture example helps check their appearance at particular animation
times. The renderer is experimental, and its guides explain the current
limitations and which aspects of its appearance have been visually verified.

Each crate's README explains how to get started and links to further examples.
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

You can also build and browse the API documentation locally. Enabling all
features includes the optional image and archive APIs in the generated docs:

```sh
cargo doc --workspace --all-features --no-deps --open
```
