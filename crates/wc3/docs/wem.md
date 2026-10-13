# WEM documents

WEM is an indexed asset document format that can hold several models, texture
references, animation clips, and named animation sets in one file. Its profiles
identify the intended game conventions and material representations. Profile
selection is stored data: reading a document does not convert its geometry,
material models, or animation channels into another game’s conventions.

This guide is for tools that inspect and edit the native file representation.
The codec lives under `wc3::model::wem`; conversion, rendering, and editor
operations remain separate from file-format support. The [conversion guide](conversion.md)
explains importing native models into WEM and building typed native exports.

## Reading a document

Use the typed document API when the container version is known. The root’s
record marker selects its independently versioned schema; it does not change the
container version or upgrade other records.

```rust
use wc3::model::wem::{Document, V3};
use wc3::model::wem::records::{DocumentRecord, V1};

fn inspect(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let document = Document::<V3>::read(bytes)?;
    let root = document.root::<DocumentRecord<V1>>()?;
    let name = document.string(root.name)?;
    println!("{} models in {:?}", root.models.entries, name);
    Ok(document.write()?)
}
```

`RawDocument` exposes the indexed container when its version or record schemas
are not known. Its sections retain encoded payloads and native metadata. Typed
array access checks a target’s tag, revision, and requested element count before
interpreting records; choosing a different record type is not a conversion.

## The native graph

A WEM version 3 file begins with a 32-byte header and uses 16-byte index entries
to identify sections by tag, offset, element count, and record revision. Tags
appear in their literal byte order, including `WOEM` and `WDOC`. The container
version and individual record revisions have separate meanings; the version 3
container uses a revision 1 document root.

A reference stores an element count, an index-table slot, and a flags word.
Several records can share one target section. An empty reference has zero
elements; its other words still belong to the native representation. Section
identities belong to the document that created them, which prevents an edit
from accidentally selecting a similarly numbered section in another file.

The root declares profiles, the default profile, coordinate space, unit scale,
and bounds, followed by links to models, textures, clips, and animation sets.
A checked document requires its default profile and every understood material
profile set to belong to that declared list. Unrecognized profile codes remain
valid when explicitly declared; future profile-set layouts stay opaque instead
of borrowing the current revision’s interpretation.

A zero header profile mask is accepted as unspecified and retained, because
existing producers may declare profiles only in the root. A nonzero mask carries
explicit profile bits. Unit scale describes the authored units. It does not cause
the codec to rescale positions. Likewise, coordinate-space metadata is retained
without applying a basis conversion.

## Records and variable fields

WEM records include geometry, scene nodes, materials, animation, and physics
metadata. They mix scalar fields with references to separate sections. A
material’s kind-specific body and feature payloads are separate referenced
records, allowing surrounding fields to remain readable when one payload is
unrecognized.

Revision-dependent families use a record dialect such as `NativeEntry<V1>` or
`NativeEntry<V2>`. Fields absent in a selected revision have a zero-width
representation instead of a runtime flag. Scalar code domains expose named
constants while retaining unrecognized numerical values.

`Array` keeps the encoded section bytes borrowed and scans record boundaries
without resolving its target sections. Reading a row decodes its inline fields;
changing a row owns the affected byte buffer and updates later row boundaries.
Inline lists can therefore allocate when a row is inspected while the containing
section still retains its borrowed representation.

Some native M3 material payloads encode a machine-sized word whose width is
not identified by the WEM header. Supply `Options::native_abi` when the producer’s
32-bit or 64-bit convention is known. Without that context, those payloads stay
opaque; the reader does not infer a machine ABI from file length.

The record API follows the file’s section families, including the connections
between shared resources and profile-specific material data. These groups are
useful starting points when locating a record rather than treating every section
as an untyped byte block.

| Record group | Native contents |
| --- | --- |
| Document and models | Profiles, model membership, bounds, slot bindings, and texture references. |
| Geometry | Meshes, sections, attributes, skin influences, split vertices, and repair metadata. |
| Scene and pose | Nodes, transforms, pose sources, pose deltas, and pose schemas. |
| Materials | Common state, kind-specific bodies, features, and native game material payloads. |
| Animation | Channels, sub-tracks, clips, events, stages, and named animation sets. |
| Physics | Bodies, shapes, joints, cloth bindings and colliders, and soft-body records. |

Record boundaries require care because some fields have variable width. An
inline list stores its count followed by its elements in the same record; it
does not create a separate section. An optional field stores a one-byte presence
marker and writes its value only when present. Consequently, element count alone
does not determine the byte size of a section containing such records.

Animation channel metadata determines the interpretation of a track’s value
bytes. Interpolation also affects key width: spline keys carry their values and
tangent data together. Retaining encoded keys allows tools to preserve animation
without evaluating it or imposing a runtime animation representation.

## Editing and preservation

Transfer a checked document into raw storage to make edits, then revalidate it
before output. Existing section IDs survive that transfer. Updating a shared
section updates its encoded payload once; individual references retain their
own requested counts and flags.

```rust
use wc3::model::wem::{Document, Ref, V3};
use wc3::model::wem::records::{Character, DocumentRecord, V1};

fn rename(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let document = Document::<V3>::read(bytes)?;
    let root_id = document.root_id();
    let mut root = document.root::<DocumentRecord<V1>>()?;
    let mut raw = document.into_raw();
    let characters: Vec<_> = b"Renamed".iter()
        .map(|value| Character { value: *value }).collect();
    let name_id = raw.append(&characters)?;
    root.name = Ref::new(characters.len() as u32, name_id.index(), 0);
    raw.replace(root_id, &[root])?;
    Ok(Document::<V3>::from_raw(raw)?.write()?)
}
```

This example appends a name section so other references to the old name keep
their original target. Replacing the existing section instead retains its slot
and affects every reference to that slot. `clear` rejects a section with known
incoming references and keeps successful removals as empty slots.

Known references identify index-table slots, so shared sections retain their
identity through edits. An unchanged source is emitted byte for byte. Unknown
sections also keep their original tags, revisions, counts, and bytes. Holes in the index remain distinct from live
sections rather than being reused implicitly.

An unsupported inline discriminator makes its containing section opaque because
the width of that alternative is not established. Truncation encountered while
reading a known alternative remains an error.
The opaque section is preserved without claiming that its unresolved graph links
have been validated.

An unknown payload may contain references or byte offsets that the codec cannot
inspect. Preserving its bytes does not establish that arbitrary relocation is
safe. With opaque sections present, output keeps the original address space and
accepts only changes that retain section lengths and slot counts. Appending or
resizing sections fails at output rather than relocating uninterpreted bytes.
Removing sections is also rejected while opaque references may exist.
