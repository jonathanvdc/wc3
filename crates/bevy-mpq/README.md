# bevy-mpq

`bevy-mpq` supplies MPQ readers for Bevy 0.19 asset sources. It has no renderer
or plugin dependency. Applications choose source names, archive precedence,
locale/platform selection, and archive parsing limits.

Open an archive with `wc3::mpq::Archive`, then wrap it in `MpqAssetReader`.
Register its source before adding Bevy's `AssetPlugin` (including through
`DefaultPlugins`). Use `OverlayAssetReader` to search any combination of MPQ
and loose-file readers in the order you supply:

```rust,ignore
use bevy::asset::{io::{AssetSourceBuilder, file::FileAssetReader}, AssetApp};
use bevy_mpq::{MpqAssetReader, OverlayAssetReader};
use std::fs::File;
use wc3::mpq::Archive;

let map = MpqAssetReader::new(Archive::open(File::open("map.w3x")?)?, "map.w3x");
let base = MpqAssetReader::new(Archive::open(File::open("base.mpq")?)?, "base.mpq");
app.register_asset_source("warcraft", AssetSourceBuilder::new(move || {
    Box::new(OverlayAssetReader::new(vec![
        Box::new(FileAssetReader::new("overrides")),
        Box::new(map.clone()),
        Box::new(base.clone()),
    ]))
}));
// Add DefaultPlugins after registering the source.
// Load with asset_server.load("warcraft://units/human/footman/footman.mdx").
```

The first mount containing an entry wins. Only missing entries permit fallback;
I/O, checksum, and decoding failures stop the search and identify the archive
and entry. Metadata comes from the selected asset mount, so a lower-priority
mount cannot supply settings for an overridden file. MPQs have no Bevy metadata
by default; configure Bevy to skip metadata checks if desired.

`with_locale(locale, platform)` tries the requested locale and then neutral
locale on the same platform within that archive. The next overlay mount is
consulted only if neither exists. MPQ name lookup is ASCII case insensitive and
accepts either slash direction. Known paths work without a listfile. Parent
traversal and absolute asset paths are rejected by the MPQ reader.

Archive indexing happens when the caller opens the archive. Reads and
extraction run on blocking workers, with a lock per archive. The returned Bevy
reader owns its decoded bytes, so consumers never hold that lock. Cloned MPQ
readers share the archive. Use `Archive::with_options` to bound file sizes and
other allocations and to select strict or permissive parsing.

This implementation supports direct reads of immutable archive snapshots.
Directory enumeration, asset processing, filesystem watching, and MPQ patch
delta application are unsupported. Directory queries return false and listing
returns an unsupported-operation error. Use separate source names for distinct
map stacks to keep Bevy handles isolated. There is no decoded-byte cache:
existence probes, metadata selection, and the eventual load may extract the
same entry repeatedly. Bevy still caches the loaded asset handles normally.

The runnable `bevy-wc3` example demonstrates caller-owned archive composition:

```sh
cargo run -p bevy-wc3 --example mpq -- \
  units/human/footman/footman.mdx map.w3x base.mpq
```

Archives are searched in argument order. This overlays complete entries; it
does not apply binary patch entries. The example uses a fixed camera; adjust
it for models with different bounds.
