# Asset command-line tools

The `wc3` examples provide small tools for inspecting, converting, and editing
assets. The [crate README](../README.md) introduces the library APIs and lists
the available examples. This guide covers common commands and the guarantees
that matter when writing modified assets. Run commands from the repository root;
all examples support `--help`, including individual `mpq` subcommands.

## Inspect model data and dependencies

Use `model_info` for metadata, geometry and emitter counts, and animation
intervals. Use `model_dependencies` to list declared resource paths and their
locations in the model:

```sh
cargo run -p wc3 --example model_info -- model.mdl
cargo run -p wc3 --example model_dependencies -- model.mdx
```

Dependency reporting covers texture, attachment, particle-emitter, popcorn FX,
FaceFX, and external animation paths, including unused references. It does not
resolve files, follow references recursively, or infer resources selected by game
logic. Replaceable IDs identify game-provided resources rather than fixed paths.

These tools, along with `model_convert` and `model_repath`, recognize MDX by its
header and otherwise read UTF-8 MDL. The separate `mdlx_compare` example selects
input format by its `.mdl` extension and writes canonical MDL in the requested
dialect. For independent converter comparisons, see the
[MDLX comparison workflow](../../../tools/mdlx-compare/README.md).

## Convert models

Use `model_convert` to change file format or model version. An output filename
ending in `.mdx` or `.mdl` selects the format; without `--version`, the source
version is preserved:

```sh
cargo run -p wc3 --example model_convert -- model.mdx model.mdl
cargo run -p wc3 --example model_convert -- model.mdx classic.mdx --version 800 --lossy
```

Version conversion is strict by default. `--lossy` permits conversion losses
and prints the conversion report to stderr; it does not bypass MDL writer
limitations. Model writes canonicalize the selected format, and output files
are overwritten. Review reported losses before using a converted model.

## Change declared paths

Use `model_repath` for literal, case-sensitive prefix replacement in the same
path fields reported by the dependency tool. It preserves empty paths and
replaceable IDs and reports each change to stderr:

```sh
cargo run -p wc3 --example model_repath -- model.mdx repathed.mdx 'Textures\' 'Custom\'
```

As with conversion, the output extension selects MDX or MDL, writing canonicalizes
the selected format, and an existing output file is overwritten. Repath changes
model references; it does not move the referenced files.

## Export textures and work with archives

The `blp_to_png` example inspects texture metadata and exports a selected mip
level. Enable `blp-decode` to access decoded pixels:

```sh
cargo run -p wc3 --features blp-decode --example blp_to_png -- texture.blp texture.png --mip 0
```

For archive listing, extraction, creation, and editing, use the
[`mpq` example](../examples/mpq.rs). Its subcommand help describes arguments
and options. Compressed entry extraction requires `mpq-decode`, and compression
when writing requires `mpq-encode`; the [crate feature guide](../README.md#optional-features)
explains how to enable them.
