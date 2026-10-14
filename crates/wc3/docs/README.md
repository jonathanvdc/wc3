# wc3 documentation

Start with the [crate README](../README.md) to choose model, texture, or archive
APIs and enable the optional codecs you need. These guides connect that overview
to editing workflows and runnable tools.

- [Warcraft III model editing](models.md) explains version selection, collection
  edits, buffer ownership, conversion, and resource references.
- [World of Warcraft M2 models](m2.md) explains core layouts, supporting documents,
  borrowed records and geometry streams, dependency resolution, and coordinated
  bundle output.
- [World of Warcraft world models](wmo.md) explains root and group dialects,
  borrowed records, collision and liquids, resource identities, and bundle edits.
- [StarCraft II and Heroes of the Storm M3 models](m3.md) explains indexed
  section graphs, independent record revisions, packed vertices, and animations.
- [Diablo models](diablo.md) explains independently versioned D3 documents,
  D4 format hashes, external payload address spaces, and semantic conversion limits.
- [Diablo textures](tex.md) explains typed TEX containers, explicit payload
  resolution, encoded editing, and optional pixel conversion.
- [WEM documents](wem.md) explains native multi-model graphs, profiles, variable
  records, and indexed-section preservation.
- [Semantic model graphs](semantic.md) explains graph relationships, typed
  authoring profiles, dependency ownership, playback declarations, borrowed arrays,
  widening, and checked edits.
- [Model conversions](conversion.md) explains native imports and export plans,
  concrete profiles, selected appearance, and representability policies.
- [Asset command-line tools](tools.md) covers inspection, dependency reporting,
  repathing, conversion, texture export, and archive operations.

The API reference contains the detailed contracts and executable examples:

- [MDLX models](https://docs.rs/wc3/latest/wc3/model/mdlx/) describes record organization
  and application chunks; [MDX](https://docs.rs/wc3/latest/wc3/model/mdlx/mdx/) and
  [MDL](https://docs.rs/wc3/latest/wc3/model/mdlx/mdl/) describe binary and text codecs.
- [M2 models](https://docs.rs/wc3/latest/wc3/model/m2/) describes native core and
  companion records, typed envelopes, dependency identities, and bundle assembly.
- [WMO models](https://docs.rs/wc3/latest/wc3/model/wmo/) describes root and group
  documents, native chunk records, validation, and historical layouts.
- [M3 models](https://docs.rs/wc3/latest/wc3/model/m3/) describes native section
  records, reference envelopes, graph validation, and external animations.
- [WEM documents](https://docs.rs/wc3/latest/wc3/model/wem/) describes native
  profiles, typed records, graph references, and encoded section editing.
- [Diablo models](https://docs.rs/wc3/latest/wc3/model/diablo/) describes typed
  appearance, actor, animation, and material documents.
- [Diablo textures](https://docs.rs/wc3/latest/wc3/tex/) describes native TEX storage
  and optional pixel conversion.
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
