# Editing semantic model graphs

`wc3::model::semantic` describes model meaning independently of file layout.
Use it to inspect relationships, author changes, or prepare a native export.
The MDLX, M2, M3, WMO, and WEM codecs remain the APIs for their encoded records
and supporting documents. A semantic graph contains actual authoring values;
it does not retain a native model or an encoded WEM document as its backing
representation.

## Objects and dependencies

Hierarchy nodes describe placement, transform inheritance, and camera-facing
behavior. Meshes describe geometry, while mesh instances place that geometry at
a node. Each instance's draws select geometry and materials. A complete material
surface owns its rendering and sampling behavior, including the family's layers
or passes. Shared texture-coordinate transforms describe animated translation,
rotation, and scale before texture sampling; they belong to the sampling domain
rather than the hierarchy.

Attached features give a node behavior such as a camera, light, or emitter.
A node can anchor several independently identified features. Systems own
relationships that span several objects, such as physics constraints, cloth,
or world groups and portals. Model controls contain declarations that apply to
the complete model. Additional controls belong directly to their actual owners,
so understanding a node or draw does not require consulting a parallel extension
table.

IDs identify objects within their owning graph or resource arena. An ID from
an unrelated owner cannot satisfy a relationship. Graph identity is independent
of native record ordering, so output codecs can reorder tables without changing
which objects refer to one another.

`ModelCollection<'data, P, C>` owns an ordered collection of graphs and one
`Resources` arena. Textures live in that arena, which owns a plain map of texture
definitions. Models store texture IDs but own no separate texture map or resource
handle. `ModelCollection::view` returns a `ModelView` borrowing a selected graph,
the collection's resources, and its collection controls. Resource lookup through
a view therefore uses the same owner as the rest of the collection.

The cardinality parameter `Single` requires exactly one graph; `Multiple`
permits several. Native MDLX, M2, M3, WMO, and Diablo assets use `Single`, while WEM can
contain multiple models with shared dependencies. Collection controls describe
shared behavior, including WEM animation selection banks. They are
separate from declarations that belong to an individual model.

## Choosing an authoring vocabulary

`Model<'data, P>` uses `P` to select the permitted authoring vocabulary. Built-in
profiles in `semantic::profiles` include `Mdlx`, `M2`, `M3`, `Wmo`, `D3`, `D4`, and `Wem`.
The default profile, `Universal`, accepts all built-in families. A profile selects
complete geometry and joint-binding schemas, material surfaces, attached
features, systems, appearance assignments, animation programs, and the controls
carried directly by each owner.

A profile chooses a required controls payload where absence has no meaning,
`Option<Payload>` where the declaration is genuinely optional, or the zero-sized
`NoControls` value where that owner has no additional controls. `Unavailable`
prevents constructing an unsupported payload family. These distinctions apply
to the actual types rather than a uniform outer option on every owner. WEM uses
`wem::physics::Physics` directly as its system payload, and M2 uses
`Option<m2::ShadowControls>` for draw controls. Unresolved Diablo shader codes
and WEM warp modes use distinct numeric newtypes, preserving their producer
values without an artificial `Unknown` branch. Embedded image resources carry
bytes whose encoding interpretation remains unspecified.

A semantic profile and a native dialect answer different questions. `Mdlx`
permits Warcraft III authoring behavior. A native dialect such as `mdlx::V800`
selects record layouts that can encode some of that behavior. Choosing a semantic
profile does not guarantee that every native revision supports every authored
value. Format-specific imports and exports use their concrete profile; translating
an incompatible vocabulary requires a separate representability decision.

```rust
use wc3::model::semantic::{Model, Node, SingleModel};
use wc3::model::semantic::profiles::{Mdlx, Universal};

let mut draft = Model::<Mdlx>::builder("Example");
let root = draft.add_node(Node::named("Root"))?;
let model = draft.finish()?;
let broader: SingleModel<'_, Universal> = model.widen();
assert_eq!(broader.node(root).unwrap().name, "Root");
let restored = broader.try_narrow::<Mdlx>()?;
assert_eq!(restored.node(root).unwrap().name, "Root");
Ok::<(), wc3::model::semantic::Error>(())
```

Widening preserves graph identities, meanings, and shared array storage. Its
`Extends` bound proves that the destination vocabulary can embed every source
payload. The separate `try_narrow::<Target>()` operation checks an exact
projection from `Universal`. It consumes the collection, moves owned resource
paths, and retains scoped identities and borrowed arrays. Each owner must already
carry a complete payload supported by the target; missing required controls,
foreign behavior, and an incompatible asset model count return an error.
Collection publication also
checks the target profile's relationship and authoring rules. Optional physical
encoding hints and unknown attachments remain outside the graph and survive
`Asset::try_narrow` unchanged.

Universal enum variants distinguish data, interpretation, or supported operations,
rather than recording the source format. MDLX, M3, and WMO appearances share
`UniversalAppearance::Single`, while M2 and general draw alternatives share
`UniversalAppearance::Draw`. Narrowing accepts these shared payloads wherever the
target supports them; the other owners still need their own compatible payloads.
The single-assignment limit survives widening. Source provenance alone does not
make an otherwise identical appearance incompatible.

Custom profiles can implement `Narrows<SourceProfile>` to define the same checked
per-owner projection from another vocabulary. Narrowing does not guess replacement
materials, synthesize controls, or perform approximate semantic translation; those require a separate
explicit transformation before narrowing.

The `union::<OtherProfile>()` operation creates a `Union` vocabulary: existing
payloads occupy `Either::Left`, and the other vocabulary can occupy
`Either::Right`. Combining vocabularies does not translate one game's shader or
emitter into another game's behavior. There is no separate Portable or Generic
profile. Format-independent features and checked simulation arenas live under
`semantic::scene`, within the universal vocabulary. Their scoped shape, body, and
joint identities cannot be substituted across simulation systems.

Applications can implement `Profile` with their own payloads. Relationship
traversal through `ValidateLinks` makes those payloads participate in checked
publication. M2, M3, and WEM publication requires a bounding declaration; MDLX and
historical WMO retain optional bounds. A named MDLX model or one with animation-file
or blend-time controls requires its model-info declaration. WMO bounds use an
authored axis-aligned box with a sphere radius derived from its half-diagonal.
Native image definitions also retain the profile's sampler and supplemental texture
controls. M2 can preserve an alternate filename alongside a replaceable-image or
file-data-ID selection. Surfaces additionally implement `SurfacePayload`, attached
features implement `FeaturePayload`, and systems implement `SystemPayload`. Geometry
schemas define their full attribute and connectivity vocabulary rather than
attaching arbitrary streams to a narrower native schema.

## Appearance and playback

Appearance schemas retain the family's actual assignment domain. A native
family may use one material assignment, draw alternatives, or named material
slots. WEM retains its shader-convention sets and alternatives, with slot bindings
and a selected default. Diablo IV owns hashed material slots, slot-local variants, and hashed looks that
select actual variants. These identities remain stable when native tables are
rebuilt. Its geometry selects an actual material slot; it does not duplicate those
assignments in common draw overrides. D3 uses the same ownership pattern with named slots and named alternatives.
Both profiles retain unused variants independently of the looks that select them;
neither infers slot identity from mutable material display names.

A material mount identifies a particular set, alternative,
and slot; an animated shader contribution can therefore retain the authored
selection rather than binding permanently to whichever material was initially
visible.

WEM can mix payload families within one model. Its Diablo material declaration
wraps the shared `profiles::diablo::d3` shader vocabulary, retaining WEM producer
variant options separately from the game shader state. Its document declarations select
default conventions, while each surface and attached feature retains its own
meaning. Its specialized Warcraft and World of Warcraft shader definitions
retain external texture and transform selections alongside their independently
authored common surface. The declared representation authority selects their
relationship; an edit does not silently reconcile one representation from the
other. Those selections refer to the originating shader's lookup domain, so they are distinct from document
`TextureId` handles. StarCraft texture-layer paths instead resolve to shared
collection texture definitions, retaining arbitrary path bytes and one edit
authority for repeated filenames. An emitter field declared to address a WEM
texture or material-slot table uses the corresponding checked document handle.

Diablo resource identities include a game namespace: a D3 group/ID pair does not
identify the D4 resource with the same numbers. WEM texture keys retain an
unspecified namespace because their native record does not encode one. Export
requires an explicit translation before a qualified identity can be written to
that unqualified field.

Playback programs belong to their actual clip or controller owners. MDLX
non-global channels use one absolute authoring timeline, and sequences select
intervals on it. Global sequences use shared clocks. WEM owns typed channel
declarations on the model and typed program tracks on controller controls.
Each declaration carries its target and initialization; each track refers to
that declaration and owns its curve. The initialization and an actor payload's
baseline remain separate authored values. WEM controller tracks are authoritative;
they are not duplicated into the common controller channel list.

D3 quaternion curves use explicit normalized linear interpolation (`Nlerp`),
which remains distinct from spherical interpolation (`Slerp`). Both are restricted
to quaternion-valued curves. Native exports reject an interpolation mode they
cannot represent rather than substituting another rotation path.

Pose stages and their animation sources have scoped identities within the model's
pose pipeline. Stage controls address those identities, rather than preserving
array indices as editable relationships. Simulation controls similarly select an
actual body, rig, cloth, soft-body effect, or lump within its owning physics system.
Their exact scalar, boolean, vector, and integer value domains remain distinct.

## Typed animation curves

A `Track<T, P>` connects a `Property<T, P>` to a checked `Curve<T>` and an explicit
clock. The value type determines which curve values are permitted, while the
profile determines the structural property vocabulary. `Profile::Property` selects
an enum implementing `PropertyPayload`. `PropertyTarget<P>` describes common
transform and sampling destinations directly, or carries an owner ID and one of
that profile's selectors. Neither display labels nor formatted paths participate
in equality, hashing, validation, or native routing.

Common destinations come from typed ID methods such as
`NodeId::translation::<Mdlx>()` and `MaterialId::texture::<Mdlx>(slot)`; the profile
can also be inferred from the surrounding channel. Profile descriptors fix an
owner identity type, value type, and property enum. For example,
`profiles::mdlx::properties::light::INTENSITY.bind(feature)` returns a
`Property<f32, Mdlx>` selecting the explicit light-intensity variant, while
`profiles::mdlx::properties::Layer(index).texture_slot(slot)` selects a material
layer and sampling slot through integer fields. M2 and M3 publish their selectors
under their own `properties` modules. M3's nested fields use `FieldPath` with enum
field segments and collection ordinals. WEM keeps Warcraft, World of Warcraft,
and StarCraft actor selectors distinct even when their field labels coincide.
Publication checks owner identities, selector families, existing layer and slot
ordinals, and declared value domains.

`Animated<T, P>` keeps a rest value and optional binding separate from playback
programs. Built-in owner payloads retain bindings in the universal structural
vocabulary so their complete payloads can move unchanged into containing profiles.
`Animated::new` embeds a supplied property into that vocabulary when the
`Extends` relationship permits it. Direct assignment to a payload's `property`
field can use `property.widen::<Universal>()`. Custom payloads may instead use
`Animated<T, ApplicationProfile>` to constrain their own bindings directly.
Program channels always carry their model's profile.

Widening maps program selectors into the containing enum while sharing the existing
curve arrays. A union carries selectors in `Either::Left` or `Either::Right` along
with its other payloads. These wrappers preserve canonical structural identity,
so the channels still address their owner fields. Exact narrowing projects accepted
selectors into the destination enum and rejects foreign vocabularies. Relationship
traversal uses transient, profile-independent structural identities to compare
bindings from different containing vocabularies; active targets retain their actual
enums. Model-level owners carry a scoped `ModelId`, just as feature and material
owners carry their own scoped IDs.

Applications define their own property enum through `Profile::Property` and
implement `PropertyPayload` for it. The default identity is the enum value itself;
containing enums delegate identity to the selected child vocabulary. A custom
`PropertyDescriptor` specifies its `Profile`, `Owner`, and `Value` types and
returns an enum selector from `key`. `Property::profile` binds that selector to an
owner. Feature and surface payloads can implement `supports_property` to check
which selectors and indexed elements they actually expose. This extends the
structural schema without registering property names or manufacturing numeric
surrogate keys.

Choose interpolation through `Curve::step`, `linear`, `hermite`, `bezier`, `tcb`,
`slerp`, or `nlerp`. The constructor's trait bounds express the permitted value
capabilities. Boolean and texture-selection values permit stepped curves;
numeric domains permit linear curves; floating-point scalars, vectors, and
quaternions also permit splines. Explicit spherical and normalized linear
interpolation require rotation values. Integer domains retain their exact value
types rather than passing through floating-point key storage. Constructors still
check finite values, ordered times, and matching array lengths. Times are finite
seconds and can precede playback. Coincident keys retain their input order: the
first key defines the incoming limit, while the last defines the boundary value
and outgoing segment.

Custom value types implement `CurveValue` and select `DiscreteDomain`,
`NumericDomain`, `ContinuousDomain`, or `RotationDomain` through its associated
`Domain` type. That choice derives the constructor capability traits and supplies
the native decoder's checks. Applications do not implement those capability
traits separately or repeat their runtime flags.

Each curve stores exactly the controls required by its interpolation. Hermite
curves expose `hermite_derivatives`, Bezier curves expose `bezier_controls`, and
TCB curves expose `tcb_parameters`; the other accessors return `None`. Derivatives
and control points have distinct associated domains. For quaternion keys both
use finite four-component vectors, allowing zero derivatives and component
controls that are not themselves rotations. Hermite derivatives retain the
source curve's segment convention; constructing a semantic curve does not
rescale them to derivatives per second or infer a new quaternion spline convention.
Native adapters check decoded sampling directives against these capabilities
and translate component layouts without changing the interpolation mode.

The array constructors accept separate `Values` for times, values, and controls,
so native adapters can retain borrowed arrays and packed projections. Typed
`ValueKeyframe`, `HermiteKeyframe`, `BezierKeyframe`, and `TcbKeyframe` records
provide a convenient authoring interface through the corresponding `*_keys`
constructors. Those constructors project their input lazily, so borrowing an
interleaved keyframe slice does not require allocating separate value arrays.
For example, a quaternion Hermite curve can carry zero derivatives explicitly:

```rust
use wc3::model::semantic::{Curve, HermiteKeyframe, Quaternion, Time};

let keys = [HermiteKeyframe {
    time: Time::seconds(0.)?,
    value: Quaternion::IDENTITY,
    incoming: [0.; 4],
    outgoing: [0.; 4],
}];
let curve = Curve::hermite_keys(&keys[..])?;
assert!(curve.values().is_borrowed());
assert_eq!(curve.hermite_derivatives().unwrap().incoming.get(0), Some([0.; 4]));
Ok::<(), wc3::model::semantic::Error>(())
```

## Borrowing and checked edits

`Values<'data, T>` exposes logical elements independently of their native
packing. It can borrow slices, decode packed elements on access, or project an
indexed source through a view. Cloning shares that storage. Imported geometry
and animation can therefore remain borrowed instead of allocating a second
copy. The decoded source must outlive every view borrowing its storage. Canonical
packed WEM attribute views also retain the borrowed primitive byte span, allowing
an output codec to reuse that span when the destination representation matches.
Edited or projected values retain their logical meaning and are encoded from those
values when direct reuse is unavailable.

A `ModelBuilder` collects an unpublished draft. Reserved IDs permit forward
references where supported; `finish` checks their definitions, ownership, and
required relationship kinds, together with local values such as geometry ranges
and curve parameters. It returns a `SingleModel`, containing the checked graph
and its resources. This standalone container dereferences to its graph for
inspection and resolves dependencies in the same ownership context.
`Model::standalone(name)` creates an empty universal container when no draft edits
are needed; use `Model::<P>::builder(name)` to author a selected vocabulary.

Published graphs provide immutable access. Use `SingleModel::edit` for a
standalone model or `ModelCollection::edit_model` within a collection.
`edit_resources` changes the shared dependency arena. Edits stage changes with
shared array storage and publish only after the affected graph and collection
relationships validate. A failed edit leaves the original collection unchanged.
Conversion `Asset` wraps this checked collection with optional encoding hints
and explicitly archived unknown attachments, and delegates the same checked
editing operations.

Coordinate basis and length units describe authored numeric values. Changing
those declarations does not transform geometry. Continue with the
[conversion guide](conversion.md) for native imports, output plans, and policies
for values a destination cannot represent.
