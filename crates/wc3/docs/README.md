# wc3 guides

Start with the [crate README](../README.md) for a first model example and an
overview of textures and archives. Choose a guide below when you need to edit an asset, evaluate it, or
understand a format's preservation rules.

## Work with models

For tools that inspect or change model relationships, begin with the graph guide.
Import and export are separate from file decoding and encoding, so choose the
native-document guides below when you need direct access to encoded records.

| Task | Guide |
| --- | --- |
| Understand IDs, math values, packed storage, and checked edits | [Model graphs](semantic.md) |
| Import a native asset or plan an export | [Import and export](conversion.md) |
| Change streams, skinning, or draw subsets | [Geometry](geometry.md) |
| Edit shader steps and texture inputs | [Materials](materials.md) |
| Author curves, clips, clocks, and events | [Animation](animation.md) |
| Inspect lights, cameras, attachments, and effects | [Features](features.md) |
| Sample poses, deform geometry, or read animated surfaces | [CPU evaluation](evaluation.md) |

## Edit native documents

These guides explain version selection, buffer ownership, validation, and output.
They also identify data that is preserved without a complete interpretation.

- [Warcraft III MDX and MDL](models.md) covers record edits, layout conversion,
  resource rewriting, and binary versus text output.
- [World of Warcraft M2](m2.md) covers core layouts, skins, external animation,
  dependency resolution, and coordinated bundle output.
- [World of Warcraft WMO](wmo.md) covers roots and groups, client profiles,
  collision and liquids, resources, and bundle validation.
- [Baked Popcorn effects](popcorn.md) covers borrowed native envelopes, compiled
  instructions, and the boundary between inspection and effect execution.
- [Command-line tools](tools.md) provides runnable inspection, conversion,
  repathing, texture, and archive commands.

## API reference and integrations

The API reference holds individual type contracts and codec details:
[models](https://docs.rs/wc3/latest/wc3/model/),
[BLP textures](https://docs.rs/wc3/latest/wc3/blp/), and
[MPQ archives](https://docs.rs/wc3/latest/wc3/mpq/).
Build it locally with all optional features from the workspace root:

```sh
cargo doc -p wc3 --all-features --no-deps --open
```

For rendering, continue with [bevy-wc3](../../bevy-wc3/README.md).
For archive-backed Bevy assets, use [bevy-mpq](../../bevy-mpq/README.md).
Contributors adding records can follow the [derive guide](../../wc3-derive/README.md).
