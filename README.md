# wc3

Rust libraries for reading, editing, and rendering Blizzard assets: Warcraft III
MDX/MDL models, World of Warcraft M2/WMO models, BLP textures, and MPQ archives.
Use them to build asset tools, process models without a renderer, or display
animated models in Bevy.

## Choose a crate

[`wc3`](crates/wc3/README.md) provides the asset APIs. You can edit models through
a checked graph, work directly with native file records, decode textures, and
read or write archives. It has no Bevy dependency. Start here for converters,
editors, and offline processing.

[`bevy-wc3`](crates/bevy-wc3/README.md) loads MDX/MDL, M2, and WMO assets into
Bevy. It provides animation, materials, skinning, attachments, particles, and
model assembly, with a viewer and an offscreen capture tool. Rendering support
varies by feature and format; the [renderer overview](crates/bevy-wc3/docs/rendering/renderer.md)
explains the current boundaries.

[`bevy-mpq`](crates/bevy-mpq/README.md) mounts MPQ archives as Bevy asset sources.
Combine archives and loose files in an ordered overlay to load models and their
textures from the same source. It can be used independently of `bevy-wc3`.

[`wc3-derive`](crates/wc3-derive/README.md) supplies codec derives for contributors
adding native records. Applications normally use the existing types and the
traits re-exported by `wc3`.

## Try the tools

From a workspace checkout, inspect a Warcraft III model or open it in the viewer:

```sh
cargo run -p wc3 --example model_info -- path/to/model.mdx
cargo run -p bevy-wc3 --example viewer -- path/to/model.mdx
```

The viewer needs a GPU and access to the model's textures. The
[asset tool guide](crates/wc3/docs/tools.md) covers conversion, dependency listing,
path replacement, texture export, and archives. The
[viewer and capture guide](crates/bevy-wc3/docs/tools.md) explains resource lookup
and rendering options. Game assets are not included.

For library integration, start with the crate README, then follow the
[asset guides](crates/wc3/docs/README.md) or
[Bevy guides](crates/bevy-wc3/docs/README.md). Detailed API references can be built
locally with `cargo doc --workspace --all-features --no-deps --open`.

## Development

The crates are developed together in this Cargo workspace. Use the Rust toolchain
selected in [CI](.github/workflows/ci.yml) and run these checks from the root:

```sh
cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

Tests include rustdoc examples. The [benchmark guide](benchmarks/README.md)
covers synthetic workloads, targeted measurements, and GPU benchmarks;
`bash benchmarks/run-quick.sh` runs the representative optimized CPU suite used
in CI.
