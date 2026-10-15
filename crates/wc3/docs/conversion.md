# Converting native model assets

The adapters under `wc3::model::conversion` connect native model APIs to
independent [semantic model graphs](semantic.md). Import maps decoded meanings
into checked objects and relationships. An export planner constructs a native
model or bundle from those objects and reports how the destination represents
them. File decoding, dependency resolution, and encoding remain the native
codec's responsibility; conversion performs no external I/O.

The graph contains authoring values rather than an embedded source file.
Geometry and animation arrays can borrow native storage through
`semantic::Values`, so their decoded source must outlive those views. Retain the
original native model separately if the application also needs its codec APIs
or original physical layout.

## Selecting a vocabulary and layout

MDLX import returns `Asset<'data, Mdlx>`. The M2, M3, WMO, and Diablo adapters use their
own concrete profiles as well. Native export requires the matching vocabulary:
for example, the MDLX planner accepts `Asset<'data, Mdlx>`, rather than implicitly
projecting a `Universal` graph into Warcraft III behavior. Widening an asset
preserves its meanings, identities, and shared arrays; it does not translate
materials, effects, or playback semantics into another game's equivalents.
A union vocabulary likewise retains both families distinctly. Use
`try_narrow::<TargetProfile>()` when every owner already carries a complete
payload supported by the target, including capabilities shared across profiles.
It moves the existing values, checks required controls and references, and rejects
incompatible model cardinality. Shared vertex geometry is checked against the
target's stream, topology, and influence constraints, independently of its source
profile. Narrowing is an exact projection; translating another game's behavior
requires an explicit semantic transformation before the native planner can accept
it.

The destination type separately selects a native dialect or envelope. A profile
can express behavior that an older output revision cannot encode. The export
boundary checks that revision's capacities, field availability, and numeric
representations. M2, M3, and WMO also construct their native supporting documents
through bundles rather than leaving offsets or references for the caller to
repair after writing.

WEM's semantic vocabulary permits multiple models, shared resources, shader
convention sets, and mixed payload families. Its typed domains are distinct from
the native WEM `Document` and `RawDocument` APIs. `conversion::wem::import`
accepts a decoded generation-three `RawDocument` and returns `Asset<Wem>`.
Generation-one and generation-two envelopes remain accessible through the raw
codec; the adapter does not interpret their distinct historical schemas. The WEM
planner constructs a new generation-three `RawDocument` directly from the checked
collection. Output record revisions
follow actual authored declarations, including meaningful absence; the native
document codec performs the subsequent byte encoding.

Diablo III appearance import produces `Asset<D3>`. Caller-assembled bundles can
include independently versioned animation documents, which bind to actual skeleton
nodes by their authored bone names. Bundle output plans the appearance and
animation resources together. Shader vocabulary is shared with WEM, but WEM's
producer options remain outside the native Diablo shader definition.

Diablo IV appearance import produces `Asset<D4>`. Native numeric vertex semantics
and encodings require explicit caller-established `BufferLayout` declarations;
matching the root hash alone does not establish every vertex interpretation.
The output `VertexLayout` separately selects native packing. Packed arrays become
logical borrowed streams, while actual chunks, material slots, variants, and
segment palettes retain checked identities. Compressed D4 animation payloads
remain accessible through the native codec; semantic animation decoding is not
implemented. The [Diablo guide](diablo.md) explains the supported projection and
its strict loss boundaries.

## Editing and inspecting an output plan

An `Asset` owns a checked `ModelCollection`, its resources, optional encoding
hints, and explicit unknown attachments. Use `Asset::edit_model` or
`Asset::edit_resources` to stage edits and publish them atomically after
validation. For a standalone graph, `SingleModel::edit` provides the same
checked workflow. Models, dependencies, mesh placements, draws, and material
assignments retain separate identities, so changing one placement need not
modify every user of a shared mesh.

A plan exposes its proposed native value through `draft()` and its ordered
conversion decisions through `report()`. Inspect both before calling `execute()`.
Execution accepts the prepared draft and returns the value with its report;
it does not perform file writes. Use the resulting model or bundle's codec when
ready to encode it.

```rust
use wc3::model::{conversion, mdlx};
use conversion::{ExportOptions, ImportOptions};

fn prepare(source: &mdlx::Model<'_, mdlx::V800>) -> Result<(), conversion::Error> {
    let asset = conversion::mdlx::import(source, &ImportOptions::default())?.value;
    let plan = conversion::mdlx::plan::<mdlx::V800>(&asset, &ExportOptions::default())?;
    let _proposed_model = plan.draft();
    let _decisions = plan.report();
    let _output = plan.execute()?;
    Ok(())
}
```

Selected appearance resolves the family's actual assignment domain. Native
single-look formats use their chosen assignment; a broader appearance vocabulary
can retain complete alternative draw assignments or WEM slot/set mounts. Native
draw-based imports construct a complete default look, and export resolves each
draw through the selected look without a base-material fallback. Export preserves
known owner-local controls instead of synchronizing a common projection against
a parallel native extension table.

Geometry normalization is a representation decision. World-model export can
join compatible placements within one group and detail level while preserving
draw order and selected materials. The join rehomes complete look assignments
using stable draw identities and relocates connectivity, strip and
tangent selections, and historical lighting windows; it leaves the source graph
unchanged. Native limits still apply, including the historical header's capacity
for four disjoint lighting windows. Incompatible region behavior or geometry
streams produces a concrete diagnostic rather than an implicit change of meaning.

## Representability and diagnostics

Strict policies are the default. Unsupported semantics fail at a concrete field
path. `LossPolicy::Drop` permits identified semantics to be discarded, while
`ApproximationPolicy::Allow { tolerance }` permits an identified numeric
approximation within the configured absolute component error. Both record their
decisions. Normalization reports an equivalent change of representation, such
as rebuilding a collision acceleration structure from its reachable membership
instead of retaining inert tree nodes.

Coordinate basis and length units describe numeric authoring values. Changing
them does not transform geometry. A destination that requires a coordinate or
unit transformation must perform it or report the limitation. Export also checks
integer clock ranges, spline representations, texture-selection domains, and
other native capacities. Successful conversion establishes representability
under the selected policies; it does not establish matching appearance in a game
renderer.

Some native records have an established binary layout but an unresolved authored
meaning. M3 import reports populated attachment and camera addon tables and the
obsolete trailing-model effect through the loss policy. Cooked physics connectivity
also has explicit representability limits. Strict conversion rejects these cases;
retaining a known record layout does not establish its behavior, and unknown-byte
archiving does not make it an editable semantic feature.

Unknown data follows a separate policy. `UnknownPolicy::Archive` retains actual
uninterpreted payload bytes in `Asset::opaque`, with their source file, tag,
revision, and count when available. These attachments are inactive and are not
reattached to rebuilt output. `UnknownPolicy::Drop` reports discarded payloads;
the default rejects them. Archiving unknown bytes cannot replace an active
semantic definition of a known feature.

Encoding hints are also separate from model meaning. They can retain physical
qualifiers such as a native shader revision, an affine UV quantization, or otherwise
uninterpreted fourth bytes of D3 packed direction vectors.
The logical values remain authoritative when they change. Hints neither store
whole-file snapshots nor impose native layout restrictions on graph editing.

## Where the implementation lives

Shared assets, policies, reports, plans, and policy enforcement live under
`model/conversion`. Each adapter divides geometry, hierarchy, animation,
materials, effects, and supporting documents into focused modules. The checked
graph and borrowed values live under `model/semantic`; authoring vocabularies
live in its `profiles` namespace. Format-independent features and scoped simulation
systems live under `semantic::scene`. Native WEM records and indexed-section editing
remain under `model/wem`, independently of the semantic graph.

Public adapter tests live under `crates/wc3/tests/model/conversion`; private
mapping tests stay beside their implementation. They check the stated graph,
editing, or native-output behavior. Renderer fidelity requires separate rendering
validation.
