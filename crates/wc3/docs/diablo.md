# Reading and editing Diablo model resources

Diablo models live under `wc3::model::diablo`. A model is assembled from independently
versioned SNO resources rather than one self-contained file. The native codecs
resolve each document's internal address space without performing storage I/O;
applications supply actor, appearance, animation, and material files themselves.
Texture containers use the separate `wc3::tex` APIs. These codecs do not add
Diablo rendering to `bevy-wc3`.

## Select a native Diablo III revision

Choose a document family and its exact revision marker before decoding. For example,
`d3::Appearance<V260>` checks revision 260 and resolves its geometry, bones, looks,
and material variants. Revision markers select structural layouts, and an appearance
revision does not select the revisions of its animations or textures.

| Document family | Marker | Native group |
| --- | --- | ---: |
| `Appearance` | `V260` | 9 |
| `Animation` | `V118` | 6 |
| `AnimationSet` | `V24` | 8 |
| `Actor` | `V282` | 1 |
| `Material` | `V25` | 57 |
| `ShaderMapDocument` | `V26` | 36 |
| `Shader` | `V150` | 37 |

These are explicit supported layouts. Newer registered client revisions are not
accepted merely because they have the same document family. Fixed records retain
unknown scalar values and reserved bytes. Array descriptors use signed offsets and
byte lengths relative to the payload after the 16-byte SNO envelope; decoding checks
ranges, complete element sizes, tag-map prefixes, and the established redundant
array counts.

```rust
use wc3::model::diablo::d3::{Appearance, V260};

let mut appearance = Appearance::<V260>::default();
appearance.root.sno_id = 123;
let encoded = appearance.write()?;
let decoded = Appearance::<V260>::read(&encoded)?;
assert_eq!(decoded.root.sno_id, 123);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The editable root is available through `.root`, and the public native records live
under `d3::records`. Writers rebuild supported offset descriptors and redundant
counts from the edited arrays. As a result, adding a vertex or material variant does
not require adjusting offsets elsewhere in the document. Header words with unresolved
meaning remain available through `.reserved`.

## Select a native Diablo IV format hash

Diablo IV identifies each root layout with a format hash. The native `d4` module
supports the following exact hashes, whose root records are independent of one
another. A matching family name or a registered historical alias does not establish
that an older file uses the same layout.

| Document family | Layout marker | Format hash |
| --- | --- | --- |
| `Appearance` | `Appearance297828cc` | `0x297828CC` |
| `Animation` | `Animation2088d14a` | `0x2088D14A` |
| `Actor` | `ActorC23b892a` | `0xC23B892A` |
| `Material` | `MaterialBf40b801` | `0xBF40B801` |

The short family aliases select these established layouts. `Document<L>` exposes
an explicit layout parameter, and `DynamicDocument::read` dispatches by the exact
header hash. Native records expose named fields and reserved gaps; no source-file
image substitutes for the editable fields.

```rust
use wc3::model::diablo::d4::Appearance;

let mut appearance = Appearance::default();
appearance.root.sno_id = 123;
let encoded = appearance.write_files()?;
let decoded = Appearance::read_with_payload(&encoded.root, &encoded.payload)?;
assert_eq!(decoded.root.sno_id, 123);
# Ok::<(), Box<dyn std::error::Error>>(())
```

An external array addresses a separately supplied payload body. Pass that body's
bytes to `read_with_payload`, excluding any storage-level SNO envelope. `read`
returns `MissingPayload` when a nonempty external array requires those bytes.
`write_files` relocates the main body and external payload independently and returns
both buffers. The simpler `write` rejects a document that produces external data,
so a caller cannot accidentally discard part of its output.

Appearance records retain native chunk, subobject, bone, look, material, and packed
stream declarations. Animation records retain permutation metadata and external
translation, rotation, and scale curves. These codecs expose the native fields;
they do not establish the game interpretation of every packed geometry or animation
value. Supported polymorphic layouts include texture animation, wind, IK, and trigger
event records. Unknown concrete type hashes and unsupported nonempty dynamic tag
maps fail explicitly. Specialized gameplay subclasses outside the supported
polymorphic families also require additional codecs. Nonzero polymorphic prefix
tables are rejected because their relocation semantics are not established.

## Edit borrowed geometry and animation leaves

Fixed leaf arrays use `ZeroVec`, including packed appearance vertices, skin influences,
indices, and animation keys. They borrow the original unaligned bytes until edited.
Metadata with nested descriptors, such as subobjects and animation permutations, is
resolved into editable vectors while its leaf buffers continue borrowing the source.
The common `Borrowed` and `IntoOwned` traits provide views and detached documents.
Diablo IV documents also expose the corresponding `as_ref` and `into_owned` methods.

Appearance revision 260 exposes the native 44-byte `FatVertex` representation. Its
methods decode biased byte normals, tangent and binormal vectors, RGBA color, and the
two fixed-point texture coordinate pairs. Raw packed fields remain editable so their
exact native state can be preserved. The later compressed vertex flag combination is
rejected explicitly; its payload must not be interpreted as 44-byte vertices.

Animation revision 118 retains permutation-local frame counts, timing scalars, bone
names, and translation, rotation, and scale curves. `Quaternion16::components` decodes
the signed normalized quaternion channels. The codec preserves attachment events and
root-motion records, but does not execute their game behavior.

## Follow native resource references

Resource fields retain their native signed SNO identities. Their containing fields
determine the resource family: an actor's `appearance` addresses an appearance,
`anim_set` addresses an animation set, and an animation set's tag maps address
animations. Material texture entries address texture resources, while uber materials
address shader maps. Keep the game namespace with these identities when combining
resources from different games; equal numeric identifiers do not establish identity
across games.

The material vocabulary distinguishes the native texture type, UV transform mode,
and UV flags. These correspond respectively to the historically misleading native
field names `dwSlotIndex`, `dwTextureFlags`, and `dwTextureType`. Unknown values remain
numeric rather than being replaced with guessed enum variants.

## Convert Diablo III appearances and resources

`wc3::model::conversion::diablo::d3::import` converts an appearance into an
independent `Asset<D3>`. Geometry streams borrow the native vertex and index
storage while exposing logical positions, directions, colors, UVs, and three
ordered skin contributions. Skeleton nodes retain their local rest transforms
and four independently authored bind declarations. Hardpoints retain their
actual skeleton relationships rather than numeric bone indices. Look-at controllers
also target actual skeleton bones, so renaming a bone updates its native look-at
reference; inactive unresolved spellings remain independently editable.

The semantic `d3::VertexStreams` type shares the `semantic::VertexData` layout,
with auxiliary colors, independent binormals, and cloth-vertex mappings in
`VertexExtras`. Influence rows use `[Influence; 3]`, preserving their ordered
zero-weight slots.

The appearance owns named material slots, slot-local variants, and named looks
that select actual variants. Renaming a slot preserves its identity and updates
native subobject references during export. Unused slots and variants remain
editable declarations. Slot spelling must be unambiguous at the native boundary;
independent slots cannot be exported with the same name. Native looks implicitly
select variants by ordinal, so an arbitrary logical selection that conflicts with
that ordering requires an explicit semantic transformation before export.

```rust
use wc3::model::{conversion, diablo::d3};
use conversion::{ExportOptions, ImportOptions};

let source = d3::Appearance::<d3::V260>::default();
let imported = conversion::diablo::d3::import(&source, &ImportOptions::default())?;
let planned = conversion::diablo::d3::plan(&imported.value, &ExportOptions::default())?;
let output = planned.execute()?.value;
let bytes = output.write()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

A caller-assembled `Bundle` can additionally supply animation revision 118
resources. Bone-name bindings become actual node identities, key times become
seconds, and quaternion curves retain normalized linear interpolation (`Nlerp`).
The animation's appearance identity must agree with the supplied appearance when
it declares one. `plan_bundle` rebuilds appearance and animation documents together;
planning an appearance alone requires an explicit loss decision for attached clips.
Root-motion samples, timing controls, and authored marker declarations remain
independent values. Trigger attachments have no semantic event mapping.

Standalone material revision 25, shader revision 150, and shader-map revision 26
resources use `import_material`, `import_shader`, and `import_shader_map`, with
matching `plan_*` operations. They share the Diablo shader vocabulary used by
WEM, including external texture and shader selections, pass order, and unresolved
option codes. WEM producer variant options remain in its own wrapper. These
resource adapters perform no dependency I/O and do not embed separately edited
shader resources into an appearance document.

Strict conversion rejects unsupported collision, cloth, octree, and static-light
projections. Unknown bone-pulse bytes follow `UnknownPolicy`; archived bytes remain
inactive attachments and are never reattached to a rebuilt graph. Packed direction
qualifiers remain optional encoding hints. Edited UVs, colors, directions, and
quaternion keys must be exactly encodable unless the configured approximation
policy permits their measured quantization error.

## Convert declared Diablo IV geometry

`wc3::model::conversion::diablo::d4` converts supported appearance declarations to
an independent `semantic::profiles::diablo::d4::D4` graph. The native record graph
remains the place to edit packed or procedural data whose semantic interpretation
has not been established. Choosing a supported SNO format hash does not establish
the meanings of every vertex-format code used by that file.

Geometry import therefore requires caller-established `BufferLayout` declarations
through `import_with_layouts`. Each declaration selects a chunk-local source buffer
and supplies a complete `VertexLayout`: its native format selector and stride, plus
one `VertexElement` for every native element. Each element binds the native semantic
code, format code, and byte offset to an explicit logical meaning and scalar encoding.
The adapter checks these declarations against the file, including duplicate meanings,
missing elements, overlapping byte ranges, complete record lengths, and skin-pair
cardinalities. It cannot verify that a caller's chosen code meanings agree with the
producing game.

The supported scalar encodings cover binary32 vectors, binary16 pairs and four-vectors,
normalized byte four-vectors, and unsigned joint-index four-vectors. Logical positions,
normals, tangents, primary colors, consecutive texture-coordinate sets, and paired
joint indices and weights become ordinary typed semantic streams. Nonzero bytes
outside the supplied element interpretations require an explicit reported loss.
There is no built-in table that guesses a game's numeric vertex codes.

Native segment windows become logical vertex and triangle windows. Palette entries
become actual skeleton-node identities, and imported vertex contributions address the
model's explicit skin binding. Native output remaps those identities through each
segment's authored palette and rebuilds byte offsets and indices. It requires the
logical segments to partition their subobject's geometry in order. An explicit output
`VertexLayout` selects the resulting scalar encoding; quantization follows the shared
conversion approximation policy.

Geometry chunks, material slots, slot variants, and hashed appearance alternatives
retain scoped identities independent of native array positions. A mesh selects its
actual chunk and material slot. An alternative selects actual variants owned by actual
slots, including variants unused by any alternative. The appearance owns material
selection; independent draw overrides would introduce conflicting edit authorities.
Bone bind transforms, inverse skinning transforms, hardpoint controls, and the
structure's independently authored sphere remain separate declarations rather than
being reconstructed from unrelated bounds or poses.

The default `import` has no vertex declarations. Strict policy rejects populated
geometry that lacks an interpretation; an explicit drop policy can retain supported
skeleton and material-reference declarations and reports the omitted geometry.
Similarly, unsupported cloth, collision, procedural skeleton controls, additional
skeleton blocks, foliage, wind, look-variant maps, and texture overrides require an
explicit loss decision. The current semantic adapter supports a complete base skeleton
without a separate cloth-bone partition. Diablo IV compressed animation curves remain
available through the native animation codec and have no semantic decoder.

An appearance's material references remain external dependencies. Supplying or editing
a standalone material uses the separate material import and planning APIs; saving an
appearance alone does not embed those resource edits. Native codecs retain their wider
supported record vocabulary even when the semantic adapter rejects a projection.

## Convert standalone Diablo IV materials

`conversion::diablo::d4::import_material` creates a checked material-only graph
from a native material resource. The shader definition retains external shader-map
and value-set selections, ordered texture inputs, constant scalar/vector parameters,
and its material-resource controls. Texture SNOs are interned in the collection's
resource arena with the Diablo IV namespace, so repeated inputs share one editable
image identity. Negative absent-image selections remain distinct from image handles.

`plan_material` selects an actual material identity and rebuilds its native document.
Appearance output keeps its external material selections and cannot silently absorb
separate shader edits. Image-level sampler and color-space overrides have no native
material-resource field and are rejected by the D4 profile. Shader texture-animation
programs, active parameter graphs, decal variants, biome/clutter declarations, and
playback-controller paths currently require an explicit reported loss. Native codecs
retain those wider layouts for applications that need their encoded records.

The implemented native layouts and semantic projections have synthetic fixture
coverage. Compatibility across real-game asset corpora and game-rendering fidelity
remain unverified.

## Native record declarations

D3 and D4 records retain serialized field layouts and numeric conventions.
Native packed-value decoding preserves declared scales and channel order;
coordinate changes belong to conversion code. Rust declarations encode the
supported D4 layouts, root format hashes, and polymorphic type-hash dispatch.

Each maintained D4 record orders its fields as they are encoded, including
explicit reserved bytes. `d4::Read`, `d4::Write`, and `d4::RecordLayout` derive
field access, relocated descriptors, and the fixed record width. The
[native layout tests](../tests/diablo_d4_layout.rs) check those computed widths
against the documented sizes. `common::Ownership` supplies borrowed views and
independent storage through the same API as other model formats. See the
[D4 derive reference](../../wc3-derive/src/d4.md) for the attributes and examples.
Document layout markers continue to select each resource's exact format hash.
