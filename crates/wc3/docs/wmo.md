# World of Warcraft world models

The `wc3::model::formats::wmo` API reads, edits, and writes WMO root documents and their
geometry groups. It preserves the native layout of each selected client profile,
borrows encoded arrays, and keeps unknown chunks in their original order. The
format layer exposes materials, portals, doodads, lighting, collision, liquids,
and resource references. Rendering and conversion between client profiles are
outside this API.

## Choosing a native layout

The ordinary root and group formats store `17` in MVER across several client
families. That number does not determine how batch prefixes or split-group links
are interpreted. Choose `Classic`, `Legion`, `Bfa`, `ShadowlandsPre92`,
`Shadowlands`, `Dragonflight`, or `WarWithin` using the input's client provenance.
`Classic` selects the pre-Legion record family, rather than every file ever
called Classic. `Shadowlands` selects the family with 9.2 split-group links.

`Root<D>` and `Group<D>` are independent documents; `Bundle<D>` connects them.
`File<D>::decode` determines the root/group role from MOHD or MOGP without
inventing a client profile. `DynamicFile::decode(bytes, profile)` provides the
same operations when the profile is chosen at runtime. `as_typed::<D>()` borrows
a matching typed document, and `visit_wmo_file!` dispatches operations over its
variants. `DynamicFile::candidates` reports structurally compatible profiles;
multiple successes require caller provenance, not an arbitrary first match.
`decode_detected()` and the dynamic `Read` trait accept a uniquely identified
profile and return an ambiguity error otherwise.

```rust
use wc3::model::formats::wmo::{Classic, Root};

fn rename_identity(bytes: &[u8], identity: u32) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut root = Root::<Classic>::decode(bytes)?;
    root.header_mut().wmo_id = identity;
    Ok(root.encode()?)
}
```

## Alpha and early-beta documents

The historical MVER 14 and 16 formats have separate document representations.
`legacy::Root14` owns a MOMO root wrapper and embedded `Group14` documents. Its
native records include 44-byte materials, 40-byte group-info records, 128-byte
group headers, and four-byte triangle metadata. Group-info offsets address the
whole file. Encoding relocates those offsets and sizes after edits while retaining
the logical group-to-info relationship and physical chunk order.

`legacy::Root16` declares external `Group16` documents with 32-byte batches and
four-byte triangle metadata. `legacy::Bundle16` resolves their sibling filenames,
validates shared references, and returns coordinated root/group output. These
records are not version-17 structures carrying another version number.
`legacy::File14` and `legacy::File16` select the document role. Runtime callers
use `Profile::Alpha14` or `Profile::Legacy16` with `DynamicFile`.

Historical group liquids preserve the original unsigned magma UV fields, and
alpha lightmap records retain their established per-triangle and texture payloads.
All historical documents expose the same format-qualified `Read` and `Write`
traits and shared owned conversion as the later documents.

## Documents and framing

Typed root documents require exactly one MVER and MOHD. Typed groups require
exactly one MVER and MOGP. Their mandatory headers are stored separately from
editable payload chunks, with private slots retaining their original framing
positions. `header()` and `header_mut()` therefore do not need a missing-header
panic or an optional result. Reserved header tags introduced through an unknown
payload are rejected on encoding.

Root payloads are accessed through `chunks()` and `chunks_mut()` iterators.
`insert_chunk` inserts at a payload index and `remove_chunks` removes matching
payload tags. Group body chunks are accessed as a slice or editable vector;
repeated UV and vertex-color layers remain separate occurrences. WMO FourCCs
are logical tags such as `MOHD` in the API and reversed on the wire.

Group decoding handles MOGP containing its header and nested chunks, and MOGP
containing only the header followed by body chunks. `framing()` identifies the
native convention and `set_framing()` changes the next output's convention.
Both retain accepted frame order, including an intervening version frame in
header-only documents. Chunk lengths are rebuilt from the encoded payloads.

`RawDocument` performs checked framing without singleton-header, model-validity,
or client-profile assumptions. Its `RawChunk` values contain a canonical tag and
borrowed or owned payload bytes. Use this representation to inspect malformed
header arrangements or uninterpreted formats. A raw stream is not evidence that
its payload records are understood.

## Borrowed arrays and edits

`codec::Array<'data, T>` borrows a checked fixed-width record block. It does not
allocate a Rust record for every encoded element, and it works when the source
address is unaligned. `get()` returns an optional decoding result and `iter()`
yields decoded values. `as_bytes()` exposes the encoded block. `set()` copies a
borrowed array's bytes on its first edit; `replace()` replaces an entire record sequence.
`from_records()` constructs an owned encoded array for insertion into a document.

```rust
use wc3::model::formats::wmo::{Classic, Root};
use wc3::model::formats::wmo::root::RootChunk;
use wc3::model::formats::wmo::records::BlendMode;

fn change_blending(root: &mut Root<'_, Classic>) -> Result<(), Box<dyn std::error::Error>> {
    for chunk in root.chunks_mut() {
        if let RootChunk::Materials(materials) = chunk {
            for index in 0..materials.len() {
                let mut material = materials.get(index).unwrap()?;
                material.blend_mode = BlendMode::Alpha;
                materials.set(index, &material)?;
            }
        }
    }
    Ok(())
}
```

Documents provide `as_borrowed()` and `into_owned()` through shared ownership
support. Owned conversion permits releasing the original input buffers.
`from_reader` and `from_reader_group` buffer standard I/O sources for the
version-17 profile family. `from_reader_dynamic` also supports the historical
profiles; all three return owned documents. `to_writer` encodes before writing to a
standard I/O sink. Direct byte-buffer decoding retains borrowed arrays.

Ordinary records use `wmo::Read`, `wmo::Write`, `wmo::RecordLayout`, and
`wmo::Validate` derives. Read and write compose independently over fields;
`RecordLayout` is the fixed inline width. Variable payload structures do not
need a fixed layout to derive writing. Cross-field validation and context-dependent
chunk codecs are handwritten. There is no animation-array visitor solely for
symmetry with M2: WMO arrays carry their own record and reference meanings.

Choice fields use named enums with `Unknown(wire)` variants that preserve
unrecognized codes. Flag fields use `bitfield!` types with named getters and
setters; `bits()` retains the complete stored word. `wmo::Value` generates
exact code conversions for custom scalar enums alongside the codec derives.
Revision numbers, indices, and counts keep their numerical types.

## Resources and bundles

`Root::dependencies(owner)` returns group, material-texture, doodad-model,
skybox, and root-light-cookie identities; resolved bundles also report group-local
animated-light cookies. Native paths keep their byte spelling;
non-UTF-8 names use `FileKey::RawPath`. File-data IDs remain numeric identities.
The shared `FileKey` also represents unresolved sibling names without imposing a
filesystem, MPQ, or CASC provider.

MOTX presence selects material filename offsets, including transitional modern
files. Without MOTX, texture slots are interpreted as file-data IDs. Historical
color and runtime words are not treated as texture resources for ordinary
shaders. `texture_reference(material, slot)` resolves a slot to filename bytes or
a file-data ID. `rewrite_texture_paths` rewrites nonempty filename references and
updates their material offsets together, leaving absent texture slots absent.
`resources::StringBlock` supports byte-offset access and append operations for
callers editing other name tables.

Legacy group paths are derived as `root_000.wmo`, `root_001.wmo`, and so on.
GFID supplies group file-data IDs in group-order rows for successive LOD levels.
An inconsistent declared GFID stride is rejected before requesting files.
Changing a group's identity requires updating both the root reference and its
registered bundle identity.

```rust
use std::collections::BTreeMap;
use wc3::model::formats::wmo::{Bundle, Classic, FileKey};

fn load<'data>(
    root: &'data [u8],
    key: FileKey,
    sources: &'data BTreeMap<FileKey, Vec<u8>>,
) -> Result<Bundle<'data, Classic>, Box<dyn std::error::Error>> {
    let bundle = Bundle::decode(root, key, |dependency| {
        Ok(sources.get(&dependency.key).map(Vec::as_slice))
    })?;
    bundle.validate_complete()?;
    Ok(bundle)
}
```

A resolver is asked for WMO group documents. Texture and doodad dependencies are
reported without loading their external assets. Missing group files leave a
partial bundle; `validate_complete()` checks required base groups and declared
LOD groups as well as cross-document references. Staged applications can use
`Bundle::from_root`, `with_key`, and `insert_group` instead.

`Bundle::encode()` returns root bytes and a map of group outputs keyed by their
registered identities. It rejects stale or undeclared group identities and
incompatible outputs sharing one file identity. It permits partial assemblies;
call `validate_complete()` before publishing a complete model. File allocation,
archive packaging, and filesystem replacement remain application decisions.

## Validation and preserved payloads

Bounded decoding checks framing, mandatory headers, native record widths, and
nested payload bounds. `Validate::validate()` checks document-local relationships
such as geometry cardinality, index and batch spans, string terminators, portal
ranges, and BSP links and cycles. Bundle validation checks references to shared
materials, lights, doodads, fog, volumes, and portal windows. Repeated known
singleton tables are rejected by validation rather than silently choosing one.

Encoding updates root group, portal, and doodad-set counts from their owning
tables when present. `synchronize_counts()` applies those updates to the editable
header. Historical exporter counts for textures, lights, doodad names, and doodad definitions remain
stored fields because they need not match the contemporary tables. Group batch
partitions and reference indices remain authored data; callers must update them
when changing geometry and use validation to check the result.

`Group::decode_with_context` accepts a caller-supplied liquid interpretation.
Liquid grids expose raw eight-byte vertex records unless an explicit liquid
context selects water-flow or magma-UV representations. A liquid database ID
alone is insufficient to guess that layout. Tangent maps use the owning MOBA
batch count; standalone MOTA decoding without that context preserves a named
unresolved payload. No unavailable game database is invented by the decoder.

The following table describes the implemented payload families. A named opaque
payload retains a known chunk's bytes or established record width without
assigning unresolved fields a false interpretation.

| Document | Typed payload families | Named opaque payloads |
| --- | --- | --- |
| Alpha/beta | Native historical headers, materials, triangles, batches, lightmaps, liquids, embedded-group offsets, and shared unchanged chunks | Reserved alpha header bytes and unverified historical payloads |
| Root | Header, materials and UV animation, names and IDs, group metadata, portal geometry and relationships, visibility blocks, lights, doodads and sets, fog, ambient volumes, convex planes, GFID | MOM3 material payloads, MDDL detail doodads, fixed-width MPVD particulate records |
| Group | Triangle metadata and narrow/wide indices, positions and normals, repeated UV/color layers, batch families and overrides, light/reference tables, terrain-cut planes, query geometry, tangent maps, BSP, liquid grids | MOPB prepass records, MOLS/MOS2 spot-light records, legacy MOLM/MOLD/MOLV and MPB payloads whose layouts are unresolved |

Unknown tags retain their original bytes. Resizing geometry or rearranging
referenced data can invalidate relationships inside an opaque payload; preservation
does not establish that arbitrary edits remain game-compatible. Field comments
identify unresolved meanings, and successful structural round trips do not
establish rendering fidelity.
