# Importing, translating, and exporting model assets

Use `wc3::model::io` to import a native document into a checked
[model graph](semantic.md). When the destination has different semantics, use
`wc3::model::conversion` to prepare its profile explicitly, then plan a native
export. These operations work in memory. Your application supplies native
documents, resolves dependencies, and writes the encoded outputs.

The graph contains authoring values rather than an embedded source file.
Compatible geometry and animation arrays borrow native encoded storage through
`PackedArray`, so their decoded source must outlive those arrays. Adapters materialize
arrays once when the native layout differs from the semantic element layout. Retain the
original native document separately if the application also needs its codec APIs
or original physical layout.

Resource paths preserve byte spellings, including non-UTF-8 paths. MDLX
particles, popcorn effects, attachments, and facial animation references resolve
through the model's resource declarations, and native output writes those bytes
without Unicode replacement. A profile `FaceEffect` pairs its target name with
an `EffectResourceId`; removing that declaration while retaining the reference
fails normal graph validation. Display names and visibility-guide text still
use Unicode strings, so invalid UTF-8 in those fields requires an explicit
import loss policy.

M2 child-model declarations use `M2ModelResource::Path` for legacy filenames
and `FileDataId` for modern geometry (`GPID`) and emitter-template (`RPID`)
references. Their roles remain separate on each particle emitter. Nonzero chunk
entries take precedence over legacy filenames; zero entries retain the filename
fallback. Widening preserves these addresses as Universal resource locators,
and narrowing back to M2 accepts both forms. Malformed or ambiguous alignment
tables are rejected independently of accepted loss for other extensions.

Numeric child addresses require a chunked output envelope. Child resource
addresses do not contain native child documents. The semantic M2 export planner
currently accepts a model and export options, with no input for those documents,
so it rejects referenced child-model trees whose payloads are unavailable.
Edit the retained native bundle when rewriting a complete child-model tree.

For an independent Warcraft III model, `io::mdlx::decode_mdx` and `decode_mdl`
combine native decoding with owned import. Both return `Model<'static, Mdlx>`
and an import report, so their input can be released immediately. `import_owned`
provides the same ownership choice for an already decoded MDLX document. The
borrowing `import` entry point remains useful when the application retains its
native source and wants to reuse compatible encoded arrays.

## Selecting a vocabulary and layout

MDLX import returns `Model<'data, Mdlx>`; M2 and WMO imports return their matching
profiles. Native export requires that same vocabulary:
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

## Translating semantic behavior

Use `conversion::Policy::<TargetProfile>` to assemble the transformations that
an application explicitly accepts before exact narrowing. An empty policy
requires exact representability. Add reusable rules with `with_rule`, or supply
a closure accepting `&mut ModelBuilder` and `&mut conversion::Report`. Each rule
can inspect its current draft with `draft.model()` and use the regular builder
methods to change it. Rules run once in insertion order, and each completed
stage must form a valid universal graph. The final `try_narrow` checks the whole
target vocabulary, including data that no rule visited.

Planning takes a shared source reference and prepares an independent draft.
A failed rule, invalid relationship, or unsupported target meaning leaves the
source unchanged. A successful plan exposes a checked target model and ordered
decisions for review, just like a native output plan. Execute that semantic plan,
then pass its target model to the native format planner.

```rust
use wc3::model::{conversion, profiles::Mdlx, FeatureId, Model};
use conversion::{DropUnsupportedFeatures, Policy};

fn translate<'data>(source: &Model<'data>, selected: Vec<FeatureId>)
    -> Result<conversion::Plan<Model<'data, Mdlx>>, conversion::Error>
{
    Policy::<Mdlx>::new()
        .with_rule(DropUnsupportedFeatures::new(selected))
        .plan(source)
}
```

`DropUnsupportedFeatures` discards only selected complete attached behaviors
that the target cannot represent. It retains supported selected features and
reports each discarded behavior with its identity and rejection reason. It does
not discard dependencies, rewrite consumers, or remove unrelated unsupported
features. A remaining reference to a removed feature fails graph validation.
An animation channel left without a target-profile consumer can also prevent
exact narrowing; dropping the feature does not authorize discarding that channel.
Applications can instead write a specific rule that repairs relationships or
removes explicitly accepted orphan channels and records the consequences before
publishing its stage.

`ConstantLightVisibility` re-expresses selected constant light switches as
zero-or-one visibility for MDLX. It rejects animated switches rather than
freezing animation at one time. Other light properties still need to be exactly
representable.

`ConstantLightSwitch::new(selected, threshold, max_error)` approximates constant
fractional visibility with a boolean switch for M2. Visibility at or above the
explicit threshold enables the light. The rule accepts only visibility from
zero to one and rejects animated inputs. It measures the absolute difference
between the original visibility and the resulting zero-or-one activation,
rejects values exceeding `max_error`, and reports the original value, threshold,
and measured error. This bound describes dimensionless activation, rather than
rendered pixel differences. A zero-error change is reported as normalization;
a nonzero accepted error is reported as approximation.

Rules that approximate curves or effects should similarly state their accepted
conditions and record every changed meaning as an `IssueKind::Approximated` or
`IssueKind::Dropped` decision. User-defined rules are responsible for those
reports; graph validation proves structural consistency and final narrowing
proves target representability, rather than visual or numerical equivalence of
an application-defined approximation.

This structure keeps exact embedding, semantic transformation, and container
encoding separate. Reusable transformations may prepare several destinations,
while specialized destination rules can preserve more fidelity than a common
fallback. Universal retains distinct authored effects when their behavior
differs; translation does not require reducing every effect to a common minimum.

## Editing and inspecting an output plan

Edit the imported model with `Model::edit`, which publishes the changes only
after graph validation. Resources, placements, draws, and material assignments
have separate identities, so changing a placement need not change every user of
its mesh. The [graph guide](semantic.md) explains these relationships.

A plan exposes its proposed native value through `draft()` and its ordered
conversion decisions through `report()`. Inspect both before calling `execute()`.
Execution infallibly accepts the validated draft and returns the value with its report;
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
    let _output = plan.execute();
    Ok(())
}
```

Export uses the selected look's complete material assignments. It preserves
supported controls on their owning objects; geometry has no fallback material.

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

For byte-preserving record edits, use the [MDLX](models.md), [M2](m2.md), or
[WMO](wmo.md) native guide.
