# WoW characters and equipment

The `bevy_wc3::wow` module adapts WoW appearance choices to the
[general assembly pipeline](assembly.md). It supplies native geoset selection,
replacement texture categories and M2 joint matching. Applications that use other
content vocabularies can construct generic assemblies directly.

## Resolve WoW appearance choices

`CharacterRecipe` identifies a WoW race-table ID, one of the adapter's body-model
sex categories, customization option/choice IDs and equipment display IDs.
Customization choices are table-defined; `EquipmentSlot` supplies the WoW inventory
vocabulary. These fields belong to this optional adapter and impose no restrictions
on generic assembly plans or the M2 codec.

`CharacterDatabase` resolves these choices using one coherent client dataset.
`CharacterTables` supplies a concrete adapter for already-resolved rows: bodies
are keyed by race and sex, customization patches by option and choice, and
equipment patches by slot and display. `AppearancePatch` carries ordering,
controlled geoset alternatives, texture replacements, atlas layers and model
parts. Stable table keys resolve ordering ties. Missing exact rows are errors;
DB2 decoding and item-to-display lookup remain application responsibilities.
`CharacterRecipe::resolve` returns `RecipeResolveError<D::Error>`, distinguishing
invalid requested or returned definitions from the database's concrete error.
The adapter imposes no thread-safety bounds on that provider error. Invalid atlas
errors retain the replacement category, original part index (or the root), and
underlying `ModelError` and `AtlasError` through standard error sources.

The resulting `CharacterDefinition` names model resources, native geosets,
replacement categories and additional parts. Applications can construct it
directly. Resolved `wow::ModelPart`s have no inventory-slot requirement, so one
selection can contribute several models or auxiliary components. Numeric native
categories are translated into scoped `DrawId` and sampled-image selections before
compilation. Atlas plans and pixels use types from `bevy_wc3::assembly`.
For native character texture-layer modes, choose `AtlasBlend::Replace` for 0/1,
`Multiply` for 4, `Overlay` for 6, `Screen` for 7, `AlphaStraight` for 9 and
`Alpha` for 15. The generic `SourceOver` default retains normalized straight-alpha
composition and is a separate equation.

## Interpret native model structure

`CharacterCatalog::inspect` extracts M2 bone identities, skin-section selections,
replacement texture categories and attachment roles. Geoset selection retains all
passes and authored LODs for each selected identifier; unknown requested values
are rejected. The adapter does not infer race-specific defaults or geoset category
arithmetic. Body selections must retain palms and fingers beneath equipment cuffs.

`EquipmentBinding::Attachment` resolves an authored attachment identity and its
pivot-local frame. The resulting generic attachment binding retains activation
in both live rendering and offline baking. `EquipmentBinding::SharedSkeleton`
uses native nonzero name checksums first, then key-bone categories when no checksum
matches. Ambiguous identities require a unique matching checksum and model-space
pivot within 0.00001 authored units. Generated names and bone order are not keys.

Explicit joint mappings can override automatic matching. Only joints weighted by
selected compiled geometry require a match; other unambiguous bones also reuse
the body skeleton to avoid duplication. With `retain_unmatched_joints`, extra
chains may remain beneath a mapped ancestor. A weighted joint without such an
ancestor is rejected. Native frozen global clocks preserve their time-zero pose.

`compile_character` takes a `CharacterDefinition`, generic `Models`, Bevy
image assets and a decoded-image callback. It translates native selections and
binding instructions, then delegates image composition, caching, repeated-source
coalescing and compilation to the generic assembly pipeline. Each source is
compiled once per distinct appearance; joint matching inspects that compiled
geometry without a separate preparation pass. Compatible repeated collection
parts share a rig. Conflicting policies for one shared-skeleton resource are
rejected; independently imported resources can carry separate appearances.

Animation and grip policy remain application choices. Select an authored weapon
stance through `Playback`, and use `PoseOverrides` for finger-pose layers,
off-hand IK or gameplay controls. Equipping a model installs its socket placement.

## Facial bone corrections

`CharacterDefinition::bone_sets` selects imported BONE resources by their complete
resource identities; `AppearancePatch::bone_sets` carries the same selections from
client tables. An empty selection retains the base face. Unknown resources,
overlapping bone ownership and ambiguous resource declarations are errors.
Selecting a face does not modify shared animations or inverse-bind matrices.

For each fresh animated local skin matrix `A`, bind pivot `p` and selected correction
`C`, evaluation computes `q = A * p` and applies `T(q) * C * T(-q) * A` before
multiplication by the corrected parent matrix, following the documented
[native correction contract](https://github.com/wowmodelviewer/wowmodelviewer/blob/3e11ee5284f451d8f08318e3f38b08865708280f/docs/fbx-customization.md#combining-facial-shape-with-animation).
Thus corrections follow the animated
joint origin and propagate through ordinary children. They retain full affine
matrices, including shear from nonuniform corrections and animation rotation.
Pose baking and live skinning use the same evaluator; after ordinary Bevy transform
propagation, the renderer publishes exact affine globals for corrected rigs and
their descendants. Node `Transform` values remain an informational TRS approximation.

Correction targets and their descendants must use ordinary affine inheritance
without billboards, camera anchoring or selective inheritance. Unsupported paths
fail compilation. Raw FacePose 808 clips are a separate authored pose and must not
be layered over an already selected face correction. Application pose overrides
are sampled before the correction, so replacing a local animation sample preserves
the selected face. Recompile with an empty selection to clear it.

## Secondary motion

Skeletal animation deforms armor without a physics solver. Add `PhysicsSimulation`
for supported authored rigid-body secondary motion; mounted parts inherit its
settings unless overridden. Native M2 imports retain bodies and joints but supply
no explicit contact grants. Applications can select an inferred contact policy
without modifying the imported model.

The [secondary-motion guide](physics.md) covers coordinate frames, contact
admission, preparation budgets, observation and geometric approximations.
`PoseOverrides` remains available for application physics or IK. The built-in
solver approximates rigid-body behavior and does not establish WoW physics parity.

## Fidelity boundaries

Correct clothing atlases, visibility rules and model choices require a matching
client build's assets and table rows. This adapter does not supply a complete WoW
game database or universal default customization choices. Generic assembly imposes
no such database schema. Native material and physics limitations remain observable
in compiled diagnostics; see the [renderer overview](renderer.md) and
[semantic compiler guide](universal.md).
