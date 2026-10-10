# bevy-mpq

`bevy-mpq` mounts MPQ archives as Bevy 0.19 asset sources and combines them
with other readers in ordered overlays. Applications choose source names,
archive precedence, locale/platform selection, and archive parsing limits.

## Register an asset source

Open an archive with `wc3::mpq::SharedArchive` and pass it to
`MpqAssetReader::new`. Register its source before adding Bevy's `AssetPlugin`
(including through `DefaultPlugins`). Use `OverlayAssetReader` to search any combination of MPQ
and loose-file readers in the order you supply:

```rust,ignore
use bevy::asset::{io::{AssetSourceBuilder, file::FileAssetReader}, AssetApp};
use bevy_mpq::{MpqAssetReader, OverlayAssetReader, OverlayMount};
use std::fs::File;
use wc3::mpq::SharedArchive;

let map = MpqAssetReader::new(SharedArchive::open(File::open("map.w3x")?)?, "map.w3x");
let base = MpqAssetReader::new(SharedArchive::open(File::open("base.mpq")?)?, "base.mpq");
app.register_asset_source("warcraft", AssetSourceBuilder::new(move || {
    Box::new(OverlayAssetReader::new(vec![
        OverlayMount::reader(Box::new(FileAssetReader::new("overrides"))),
        OverlayMount::mpq(map.clone()),
        OverlayMount::mpq(base.clone()),
    ]))
}));
// Add DefaultPlugins after registering the source.
// Load with asset_server.load("warcraft://units/human/footman/footman.mdx").
```

The source name is part of each asset path, such as `warcraft://units/model.mdx`.
Use distinct source names for separate map stacks so their Bevy handles remain
independent. The label passed to `MpqAssetReader::new` identifies the archive in
error messages.

## Lookup and overlay precedence

The first mount containing an entry wins. Only missing entries permit fallback;
I/O, checksum, and decoding failures stop the search and identify the archive
and entry. Metadata comes from the selected asset mount, so a lower-priority
mount cannot supply settings for an overridden file.

MPQ mounts do not supply Bevy metadata. `OverlayMount::mpq` uses the archive
index to select the mount during metadata lookup, without extracting the asset.
Generic mounts created with `OverlayMount::reader` select metadata by reading
the asset, which can repeat work during the subsequent load.

`with_locale(locale, platform)` tries the requested locale and then neutral
locale on the same platform within that archive. The next overlay mount is
consulted only if neither exists. MPQ name lookup is ASCII case insensitive and
accepts either slash direction. Known paths work without a listfile. Parent
traversal and absolute asset paths are rejected by the MPQ reader.

## Runtime behavior and limits

Reads extract complete entries on blocking workers and return readers that own
the decoded bytes. Clones share the archive and allow up to four extractions at
once. Use `with_max_concurrent_reads` with a `NonZeroUsize` value before cloning
to choose a different limit. Requests wait asynchronously for capacity, which
is released when extraction finishes.

The extraction limit controls work in progress. Decoded bytes can remain in use
after extraction finishes, and their memory is not covered by that limit.
Configure `SharedArchive::with_options` to bound individual file sizes and other
allocations, and to choose strict or permissive parsing. Keep the archive source
unchanged while it is mounted. See the [MPQ API documentation](https://docs.rs/wc3/latest/wc3/mpq/)
for supported sources and recovery diagnostics.

The readers have no decoded-byte cache; Bevy caches loaded assets normally.
Directory enumeration, asset processing, filesystem watching, and MPQ patch
delta application are unsupported. Directory queries return false, and directory
listing returns an unsupported-operation error.

## Try archive-backed models

The runnable `bevy-wc3` example demonstrates caller-owned archive composition
and rendering through the [model plugin](../bevy-wc3/README.md):

```sh
cargo run -p bevy-wc3 --example mpq -- \
  units/human/footman/footman.mdx map.w3x base.mpq
```

Archives are searched in argument order. This overlays complete entries; it
does not apply binary patch entries. The example uses a fixed camera; adjust
it for models with different bounds.
