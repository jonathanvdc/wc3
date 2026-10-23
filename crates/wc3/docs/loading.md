# Load a model and its supporting files

Use `wc3::model::io::asset` when an application needs a checked model from native
bytes, including M2 companions or WMO groups. The reader handles native layouts,
dependency traversal, and bundle validation. Your application supplies bytes for
typed requests, so the same workflow works with files, archives, network caches,
or an asynchronous asset system without making `wc3` depend on those systems.

Choose a `Format` and a root `FileKey`. `Format::Mdlx` accepts binary MDX when the
bytes begin with `MDLX` and otherwise reads UTF-8 MDL. `Format::M2` selects the
native dialect from the encoded document. `Format::Wmo(profile)` requires explicit
client provenance because the WMO version alone does not identify its layout.
The reader supports modern WMO profiles; the native legacy WMO APIs remain
available separately. M2 version 272 uses the native decoder's default split
blend-time layout. For its earlier `V272PreSplit` interpretation, use the direct
native `decode_m2_with_layout` and bundle APIs.

## Supply required companions

For synchronous sources, call `asset::load` with the root bytes, import policy,
and a resolver.
Each `Request` carries the owning document, required file identity, and its
`CompanionKind`. The resolver returns that companion's bytes or its own typed
source error. Successful loading returns an owned Universal model and its
conversion report:

```rust
use std::{error::Error, fs, io};
use wc3::model::common::FileKey;
use wc3::model::conversion::ImportOptions;
use wc3::model::io::asset::{self, Format};

fn inspect(path: &str) -> Result<(), Box<dyn Error>> {
    let imported = asset::load(
        Format::M2,
        FileKey::Path(path.to_owned()),
        fs::read(path)?,
        ImportOptions::default(),
        |request| {
            let FileKey::Path(path) = &request.key else {
                return Err(io::Error::new(io::ErrorKind::Unsupported,
                    format!("an archive provider is required for {:?}", request.key)));
            };
            fs::read(path)
        },
    )?;
    println!("{}: {} nodes", imported.value.name(), imported.value.nodes().len());
    Ok(())
}
```

This filesystem example accepts concrete path identities. A game archive may
instead map `FileKey::FileId` to entries, while `FileKey::Sibling` retains the
owner and suffix for a source whose concrete filename is unavailable. Preserve
those identities rather than inventing names for numeric IDs. The request kind
lets a provider distinguish skins, animations, skeletons, physics, child models,
and WMO groups even when storage uses a common namespace.

The reader requests files required to understand the native bundle. It does not
fetch image pixels or compile a renderer. Imported resource declarations retain
the image and external-resource identities needed by later processing.

For asynchronous loading or source retries, create a `Reader` with the same
format, identity, root bytes, and import policy. Call `poll` to obtain `Progress::Pending(request)`,
await your source read, then call `provide(&request, bytes)` and poll again.
`Progress::Ready(imported)` completes the session. Polling while a request remains
pending returns that same request, so a failed source read need not restart
decoding. The reader itself performs no I/O and depends on no executor.

## Import checked semantics

The completed `Conversion<Model<'static, Universal>>` owns its semantic storage.
It can outlive the reader, root bytes, companion buffers, and temporary native
documents. The reader imports from borrowed native records, then detaches the
checked graph before releasing those sources. It retains no native backing
bundle in the result. Applications can edit the graph, share it in an asset
cache, evaluate it, or pass it to a renderer.

`ImportOptions::default()` is strict. Explicit options can permit supported loss,
and the returned report records accepted drops and approximations. Those options
do not make malformed native data or absent required companions valid. The policy
is fixed when the session starts and applies to its complete import. See
[import and export](conversion.md) for policy selection and native output.

Reader failures distinguish native decoding, unsupported provenance, and invalid
request sequencing; import failures retain the conversion error and root identity. Match the typed
errors or follow their standard error sources instead of parsing display text.
`LoadError::Companion` retains the request and the resolver's original error;
`LoadError::Native` retains the decoding or import failure. Source policy remains
application-owned, where the resolver can retain its archive path, attempted
locations, and underlying I/O failure.

For native record editing or coordinated output, use the direct
[MDLX](models.md), [M2](m2.md), and [WMO](wmo.md) document APIs. The shared reader
provides a semantic ingestion boundary; applications that need original records
and layout controls should keep their native document workflow.
