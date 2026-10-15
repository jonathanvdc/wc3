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
Collection publication also checks the target profile's relationship and
authoring rules. Optional physical encoding hints and unknown attachments remain
outside the graph and survive
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

## Geometry representations and profile constraints

`MdlxGeometry`, `M2Geometry`, and `M3DisplayGeometry` retain distinct constrained
APIs while sharing vertex-stream storage. M2 construction requires a two-element
UV array and four weighted influence slots per vertex, and exposes no tangent or
color constructor. MDLX supports optional normals and tangents, ordered triangle
groups, and `JointInfluences` with either equal-weight matrix groups or four
weighted slots. M3 display geometry requires normals and weighted slots, and
permits tangents, colors, and up to five UV slots with meaningful gaps. Checked
constructors establish stream cardinalities and finite values; the profile types
keep unsupported representations out of subsequent authoring operations.

Widening these display types produces `UniversalGeometry::Data` containing
`GeometryData`, without a source-profile tag. The arrays move unchanged,
including borrowed slices, packed views, and shared owned buffers. Narrowing
checks the actual streams before constructing the destination's constrained
type. M2 geometry can therefore narrow to M3 display geometry, while M3 geometry
with colors cannot narrow to MDLX or M2. Missing normals, absent weighted rows,
unsupported UV gaps, and ordered primitive groups are rejected wherever the
target cannot retain them. Neither zero-weight influence slots nor matrix-group
ordering are discarded, and equal-weight groups are not converted into weighted
slots.

`GeometryData` combines common streams, custom attributes, and connectivity with
`GeometryInfluences`. Its influence variants retain absent influences,
variable-length weighted rows, three or four fixed weighted slots, and ordered
equal-weight matrix groups. Custom attributes can coexist with each influence
representation. Narrowing to a constrained display type rejects custom attributes
and influence representations that the destination cannot retain; even a
variable-length row containing four entries remains distinct from four authored
slots. `UniversalGeometry::Collision` keeps collision connectivity and
per-triangle controls separate from display geometry.

`VertexData` shares the typed vertex layout used by display geometry, WMO,
Diablo III, and collision. Its stream types select fixed UV arrays, optional
streams, multiple color sets, or `()` for unavailable shading data. An explicit
extras type owns additional vertex streams. Common remapping carries those
streams through the same vertex correspondence, while connectivity and joint
rows remain separate fields of their constrained geometry owner.

WMO retains its face controls, strip connectivity, and draw-selected tangent
pool. Its ordered color sets and baked-light coordinates participate in shared
vertex remapping, while face records and tangent pools keep their own domains.
Diablo III retains independent three-vector tangents and binormals, auxiliary
colors, and cloth mappings; its influence rows have the array type
`[Influence; 3]`. Those additional meanings keep both geometries in distinct
universal variants.

Diablo IV uses `GeometryData` and widens directly to `Data`.
Narrowing checks triangle connectivity and vertex-domain attributes before
constructing `d4::Geometry`. Weighted rows retain their variable or fixed width
without copying their arrays. Equal-weight matrix groups require an explicit
transformation before entering this weighted geometry vocabulary. Native adapters
also check whether the selected vertex layout can encode the authored streams.

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
those declarations does not transform geometry. Native imports, output plans,
and policies for values a destination cannot represent are described in the
[conversion guide](conversion.md).

## Common geometry editing

Editors can use `GeometrySchema::edited` to change geometry without selecting a
profile-specific constructor. `GeometryEdit` supplies atomic replacements for
positions, vertex normals, tangent frames, primary colors, and ordered UV streams.
An omitted field retains its existing value; an empty stream removes an optional
value. The operation returns a validated replacement and leaves the original
geometry unchanged on success or failure. Unchanged `Values` retain shared storage;
remapped streams are materialized as owned logical values.

`edit_capabilities` describes supported common streams, required normals or colors,
UV-set limits, and whether the current geometry permits vertex remapping. M2, for
example, requires normals and exactly two UV streams. Diablo III tangent and
binormal vectors remain distinct profile streams, so its common tangent-frame
operation is unavailable. Capabilities guide tools before they submit an edit;
validation still checks cardinalities, finite values, and the profile's complete
invariants. Union branches retain their constrained schema. Universal `Data`
geometry exposes the shared representation's broader editing capabilities: an
M2 mesh widened to `Universal` can acquire tangents, but subsequent M2 narrowing
rejects that added stream. Other universal variants retain their own complete
schema. Custom schemas opt in by implementing the same two methods; their default
implementation rejects common edits.

For structural edits, `VertexRemap` gives one old vertex index per new vertex and
replacement connectivity. Repeated indices duplicate vertices, permutations
reorder them, and unused vertices can be deleted. Each new corner must select the
same old vertex as its corresponding old corner. Primitive kinds, mixed groups,
polygon boundaries, face order, and corner order remain unchanged. Adapters carry
all vertex streams through this correspondence, including skin influences,
additional vertex attributes, Diablo auxiliary colors and cloth mappings, and WMO
lighting colors and baked-light coordinates. Face and corner attributes and their
scoped identities remain intact. Stream replacements apply after remapping, which
allows a tool to duplicate vertices and move the copies in one checked operation.

The following edit duplicates a shared vertex while retaining its triangle's
corner order. The same operation works for any schema that supports remapping.

```rust
use wc3::model::semantic::{
    GeometryData, GeometryEdit, GeometrySchema, Topology, VertexRemap,
};

let geometry = GeometryData::triangles(
    vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
    vec![[0, 1, 2]],
)?;
let edited = geometry.edited(GeometryEdit {
    remap: Some(VertexRemap {
        new_to_old: vec![0, 1, 2, 0],
        topology: Topology::Triangles(vec![[3, 1, 2]].into()),
    }),
    ..GeometryEdit::default()
})?;
assert_eq!(edited.positions.len(), 4);
assert_eq!(geometry.positions.len(), 3);
Ok::<(), wc3::model::semantic::Error>(())
```

To publish an edit, clone the mesh from the draft's model, replace its `geometry`
with the result of `edited`, and submit it through `ModelBuilder::set_mesh` inside
`SingleModel::edit` or `ModelCollection::edit_model`. Geometry editing does not
recompute mesh or model bounds, transform normals automatically when positions
change, or rewrite references held by other graph objects. Tools must update those
related declarations in the same collection transaction where needed. In
particular, vertex references in physics systems require their own correspondence
updates; graph validation cannot establish that an in-range index still identifies
the intended vertex.

The common protocol currently preserves topology structure. Adding or splitting
faces, interpolating new vertices, changing skinning representations, and creating
profile-specific attribute layers require explicit profile operations. WMO geometry
with strip indices rejects vertex remapping because draws retain separate strip
windows. WEM geometry with declared edges, triangulations, or repair history also
rejects remapping rather than guessing correspondence for those domains. Ordinary
stream edits remain available for those meshes. Native index widths, packing, and
quantization continue to be checked during export.
