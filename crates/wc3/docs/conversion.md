# Importing and exporting model assets

The adapters under `wc3::model::io` connect native documents to
independent [semantic model graphs](semantic.md). Import maps decoded meanings
into checked objects and relationships. An export planner constructs a native
document or bundle from those objects and reports how the destination represents
them. File decoding, dependency resolution, and encoding remain the native
codec's responsibility; conversion performs no external I/O.

The graph contains authoring values rather than an embedded source file.
Compatible geometry and animation arrays borrow native encoded storage through
`ZeroVec`, so their decoded source must outlive those arrays. Adapters materialize
arrays once when the native layout differs from the semantic element layout. Retain the
original native document separately if the application also needs its codec APIs
or original physical layout.

For an independent Warcraft III model, `io::mdlx::decode_mdx` and `decode_mdl`
combine native decoding with owned import. Both return `Model<'static, Mdlx>`
and an import report, so their input can be released immediately. `import_owned`
provides the same ownership choice for an already decoded MDLX document. The
borrowing `import` entry point remains useful when the application retains its
native source and wants to reuse compatible encoded arrays.

## Selecting a vocabulary and layout

MDLX import returns `model::Model<'data, Mdlx>` directly. The M2 and WMO adapters use their own concrete profiles as well. Native export requires the matching vocabulary:
for example, the MDLX planner accepts `Model<'data, Mdlx>`, rather than implicitly
projecting a `Universal` graph into Warcraft III behavior. Widening a model
preserves its meanings, identities, and encoded arrays; it does not translate
materials, effects, or playback semantics into another game's equivalents.
Use `try_narrow::<TargetProfile>()` when every owner already carries a complete
payload supported by the target, including capabilities shared across profiles.
It moves the existing values and checks required metadata, controls, and references.
Shared vertex geometry is checked against the
target's stream, topology, and influence constraints, independently of its source
profile. Narrowing is an exact projection; translating another game's behavior
requires an explicit semantic transformation before the native planner can accept
it.

The destination type separately selects a native dialect or envelope. A profile
can express behavior that an older output revision cannot encode. The export
boundary checks that revision's capacities, field availability, and numeric
representations. M2 and WMO also construct their native supporting documents
through bundles rather than leaving offsets or references for the caller to
repair after writing.

## Editing and inspecting an output plan

A `model::Model` owns its checked graph, resources, and profile-specific
metadata. Conversion adapters return this same model. `metadata()` exposes the
payload selected by `Profile::ModelMetadata`; use `ModelBuilder::set_metadata`
inside `Model::edit` to replace it. `Model::edit` stages graph, texture, and
metadata changes and publishes them atomically after validation. Dependencies, mesh
placements, draws, and material assignments retain separate identities, so
changing one placement need not modify every user of a shared mesh.

A plan exposes its proposed native value through `draft()` and its ordered
conversion decisions through `report()`. Inspect both before calling `execute()`.
Execution accepts the prepared draft and returns the value with its report;
it does not perform file writes. Use the resulting document or bundle's codec when
ready to encode it.

```rust
use wc3::model::{conversion, formats::mdlx, io};
use conversion::{ExportOptions, ImportOptions};

fn prepare(source: &mdlx::Document<'_, mdlx::V800>) -> Result<(), conversion::Error> {
    let model = io::mdlx::import(source, &ImportOptions::default())?.value;
    let plan = io::mdlx::plan::<mdlx::V800>(&model, &ExportOptions::default())?;
    let _proposed_model = plan.draft();
    let _decisions = plan.report();
    let _output = plan.execute()?;
    Ok(())
}
```

Selected appearance resolves the family's actual assignment domain. Native
single-look formats use their chosen assignment; a broader appearance vocabulary
can retain complete alternative draw assignments. Native
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

Unknown source data follows an import policy. `UnknownPolicy::Reject` is the
default; `UnknownPolicy::Drop` discards it and reports the decision. The semantic
model stores supported authoring values, so export needs no unknown-byte policy.
Keep the native document separately when an application needs its original
unknown records or bytes.

Native codec APIs retain their own byte-preserving editing behavior.

## Where the implementation lives

Policies, reports, plans, and policy enforcement live under `model/conversion`.
Native adapters live under `model/io`; each divides geometry, hierarchy, animation,
materials, effects, and supporting documents into focused modules. The checked
graph and encoded values live under `model`; authoring vocabularies
live in its `profiles` namespace. Format-independent features and scoped simulation
systems live under `model::scene`. Native records remain under `model::formats`, independently of the semantic graph.

Public adapter tests live under `crates/wc3/tests/model/conversion`; private
mapping tests stay beside their implementation. They check the stated graph,
editing, or native-output behavior. Renderer fidelity requires separate rendering
validation.
