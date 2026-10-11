# wc3 documentation

Start with the [crate README](../README.md) to choose model, texture, or archive
APIs and enable the optional codecs you need. These guides connect that overview
to editing workflows and runnable tools.

- [Model editing](models.md) explains version selection, collection edits,
  buffer ownership, conversion, and resource references.
- [Asset command-line tools](tools.md) covers inspection, dependency reporting,
  repathing, conversion, texture export, and archive operations.

The API reference contains the detailed contracts and executable examples:

- [Models](https://docs.rs/wc3/latest/wc3/model/) describes record organization
  and application chunks; [MDX](https://docs.rs/wc3/latest/wc3/model/mdx/) and
  [MDL](https://docs.rs/wc3/latest/wc3/model/mdl/) describe binary and text codecs.
- [BLP textures](https://docs.rs/wc3/latest/wc3/blp/) covers encoded containers,
  external BLP0 mipmaps, and optional image adapters.
- [MPQ archives](https://docs.rs/wc3/latest/wc3/mpq/) covers streaming and
  concurrent extraction, encoded copying, editing, and recovery policy.

For record contributors, the [derive guide](../../wc3-derive/README.md) links
both attribute references. For rendering applications, continue with
[bevy-wc3](../../bevy-wc3/README.md); archive-backed Bevy sources use
[bevy-mpq](../../bevy-mpq/README.md).

Build the complete API reference locally from the workspace root:

```sh
cargo doc -p wc3 --all-features --no-deps --open
```
