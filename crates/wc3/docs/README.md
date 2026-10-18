# wc3 documentation

Start with the [crate README](../README.md) to choose model, texture, or archive
APIs and enable the optional codecs you need. These guides connect that overview
to editing workflows and runnable tools.

- [Semantic model graphs](semantic.md) explains graph relationships, typed
  authoring profiles, dependency ownership, playback declarations, borrowed arrays,
  typed material programs and shader dialects, widening, and checked edits.
- [Model import and export](conversion.md) explains native imports and export plans,
  concrete profiles, selected appearance, and representability policies.
- [Warcraft III native document editing](models.md) explains version selection, collection
  edits, buffer ownership, conversion, and resource references.
- [World of Warcraft M2 models](m2.md) explains core layouts, supporting documents,
  borrowed records and geometry streams, dependency resolution, and coordinated
  bundle output.
- [World of Warcraft world models](wmo.md) explains root and group dialects,
  borrowed records, collision and liquids, resource identities, and bundle edits.
- [Asset command-line tools](tools.md) covers inspection, dependency reporting,
  repathing, conversion, texture export, and archive operations.

The API reference contains the detailed contracts and executable examples:

- [Model graphs](https://docs.rs/wc3/latest/wc3/model/) describes the default model
  representation, profiles, checked edits, and native adapters.
- [MDLX models](https://docs.rs/wc3/latest/wc3/model/formats/mdlx/) describes record organization
  and application chunks; [MDX](https://docs.rs/wc3/latest/wc3/model/formats/mdlx/mdx/) and
  [MDL](https://docs.rs/wc3/latest/wc3/model/formats/mdlx/mdl/) describe binary and text codecs.
- [M2 models](https://docs.rs/wc3/latest/wc3/model/formats/m2/) describes native core and
  companion records, typed envelopes, dependency identities, and bundle assembly.
- [WMO models](https://docs.rs/wc3/latest/wc3/model/formats/wmo/) describes root and group
  documents, native chunk records, validation, and historical layouts.
- [BLP textures](https://docs.rs/wc3/latest/wc3/blp/) covers encoded containers,
  external BLP0 mipmaps, and optional image adapters.
- [MPQ archives](https://docs.rs/wc3/latest/wc3/mpq/) covers streaming and
  concurrent extraction, encoded copying, editing, and recovery policy.

For record contributors, the [derive guide](../../wc3-derive/README.md) links
the format-specific derive references. For Warcraft III rendering applications, continue with
[bevy-wc3](../../bevy-wc3/README.md); archive-backed Bevy sources use
[bevy-mpq](../../bevy-mpq/README.md).

Build the complete API reference locally from the workspace root:

```sh
cargo doc -p wc3 --all-features --no-deps --open
```
