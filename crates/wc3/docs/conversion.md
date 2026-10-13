# Converting model assets through WEM

The adapters under `wc3::model::conversion` use WEM as an editable intermediate
representation. Import maps native model fields into WEM geometry, nodes,
materials, animation, and profile-specific records. Export builds a typed native
model or bundle from those fields. The native format codecs continue to own
file decoding, dependency resolution, and encoding.

This separation lets an application inspect and edit the converted model before
choosing its destination dialect. Import does not encode a copy of the original
file into the result. Keep the original decoded model yourself when your
application also needs its native codec APIs.

## Importing and planning an export

Each adapter has `import` and `plan` entry points. The import returns an `Asset`
with its editable WEM document and an ordered report. Export options select a
model, material profile, and look; the destination type selects its native
layout. M2, M3, and WMO adapters operate on bundles so supporting documents
participate in conversion.

```rust
use wc3::model::{conversion, mdlx};
use conversion::{ExportOptions, ImportOptions};

fn convert(source: &mdlx::Model<'_, mdlx::V800>)
    -> Result<mdlx::Model<'static, mdlx::V800>, conversion::Error>
{
    let imported = conversion::mdlx::import(source, &ImportOptions::default())?;
    let plan = conversion::mdlx::plan::<mdlx::V800>(
        &imported.value, &ExportOptions::default(),
    )?;
    for issue in &plan.report().issues {
        println!("{}: {}", issue.path, issue.description);
    }
    Ok(plan.execute()?.value)
}
```

A plan already contains a validated native draft. `draft()` exposes it for
inspection, and `execute()` returns that draft with its report. Execution does
no external I/O. Use the destination model or bundle's codec to write its files.

## Editing common and native fields

`Asset::model` reads the selected model's common authoring values. Logical
material slots resolve through the selected look's bindings to the physical
material table. `wem::semantic::build` writes changed values back into WEM.
When the view carries an existing native WEM graph, it preserves native fields
and other models and looks while replacing mapped values. Changing the shape of
that graph currently requires explicit native graph editing; the builder checks
table cardinalities before retaining references. Retaining the active native
graph during a semantic rebuild currently requires the current model, node,
and clip record schemas; older schemas remain accessible through the native
WEM codec. Rebuilds prune unreachable payloads, retaining their slots as holes so live index identities stay stable,
and release the original container buffer before taking ownership.

A polygon mesh can expose several material draw sections. Face-domain integer
attributes named `section` assign faces to those sections. The common view
produces one triangle mesh per section and expands corner- or face-domain
attributes into explicit vertices, preserving shading seams. Its
`MeshProjection` records retain the original mesh, section, vertex, corner, and
face addresses. A semantic rebuild writes compatible edits back into those
original domains and preserves the polygon topology. Edits to repeated source
values must agree across draw views; changing projected topology requires
editing the native polygon records or constructing a new generic model.

Native export consumes these draw views directly. Features that depend on the
original vertex or section identities, such as M3 cloth cages, require an
explicit remapping and produce a diagnostic when that mapping is unavailable.
Edge-domain attributes likewise require an explicit edge identity table before
an export view can interpret them.

Profile-specific animation identities use `Target::NativeProperty`; their
owning adapter resolves those identities to native control fields. Common node
translation, rotation, and scale remain explicit target variants. `Key<T>` and
`Track<T>` keep integer values in their original scalar type; `IntegerTrack`
selects the signed, unsigned, Boolean, or packed vector channel layout. Controller
names, priorities, and concurrent execution settings live in `TrackLayer`.

The common view exposes the document's `space` and `unit_scale` metadata.
Changing metadata alone does not transform coordinates. Native export reports
when it preserves numeric coordinates across authoring-space tags and rejects
unit scales that require an unimplemented scene-wide rescale under strict policy.
Changing shared coordinate metadata through a single native model view is
rejected because it would also affect the other models in that document.

Native materials, emitters, cameras, lights, and physics are actual typed WEM
records. Edit those records through `Document::raw()` and the indexed storage
API when their common view does not expose the required control. A material's
`NativeSync` identifies whether the common or native representation is
authoritative. Common material edits made through the semantic builder mark the
common representation as changed so the adapter can update its native fields.

`semantic::build_with_options` selects document options for a new model,
including the producer ABI required by ABI-dependent native M3 materials. A view
of an existing document retains that document's options.

## Representability and diagnostics

Strict policies are the default. A conversion fails at a concrete field path
when a destination cannot represent a feature or requires a numeric
approximation. `LossPolicy::Drop` explicitly permits discarded semantics;
`ApproximationPolicy::Allow { tolerance }` permits an identified approximation
only within the configured absolute component error. Both produce report
entries. A successful conversion establishes representability under those
policies; it does not establish matching appearance in a game renderer.

Unknown native payloads follow a separate policy. `UnknownPolicy::Archive`
retains only those uninterpreted payloads in `Asset::opaque`, together with their
file identity, tag, revision, and count when available. They remain inactive and
are never inserted into rebuilt native output. `UnknownPolicy::Drop` reports
the discarded payloads; the default rejects them. Retaining unknown bytes does
not provide an active representation of unsupported, understood semantics.

## Finding implementation code

Shared policies, reports, assets, plans, and policy enforcement each have a
small module under `model/conversion`. Native adapters have separate modules
for geometry, hierarchy, animation, materials, effects, and metadata. Larger
families split further by payload or direction, keeping orchestration apart
from field mapping. WEM's semantic construction, reading, geometry projection,
triangulation, validation, and native editing helpers live under
`model/wem/semantic`.

Public conversion tests live under `crates/wc3/tests/model/conversion`. They
exercise rebuilding from WEM fields, native-field edits, selected looks,
companion requirements, and policy decisions. Private helper tests stay beside
the implementation they exercise.
