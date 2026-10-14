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
permits several. Native MDLX, M2, M3, and WMO assets use `Single`, while WEM can
contain multiple models with shared dependencies. Collection controls describe
shared behavior, including WEM animation selection banks. They are
separate from declarations that belong to an individual model.

## Choosing an authoring vocabulary

`Model<'data, P>` uses `P` to select the permitted authoring vocabulary. Built-in
profiles in `semantic::profiles` include `Mdlx`, `M2`, `M3`, `Wmo`, and `Wem`.
The default profile, `Universal`, accepts all built-in families. A profile selects
complete geometry and joint-binding schemas, material surfaces, attached
features, systems, appearance assignments, animation programs, and the controls
carried directly by each owner.

A profile chooses a required controls payload where absence has no meaning,
`Option<Payload>` where the declaration is genuinely optional, or the zero-sized
`NoControls` value where that owner has no additional controls. `Unavailable`
prevents constructing an unsupported payload family. These distinctions apply
to the actual types rather than a uniform outer option on every owner.

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
carry the target family's complete payload; missing required controls, foreign behavior,
and an incompatible asset model count return an error. Collection publication also
checks the target profile's relationship and authoring rules. Optional physical
encoding hints and unknown attachments remain outside the graph and survive
`Asset::try_narrow` unchanged.

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
and a selected default. A material mount identifies a particular set, alternative,
and slot; an animated shader contribution can therefore retain the authored
selection rather than binding permanently to whichever material was initially
visible.

WEM can mix payload families within one model. Its document declarations select
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

Playback programs belong to their actual clip or controller owners. MDLX
non-global channels use one absolute authoring timeline, and sequences select
intervals on it. Global sequences use shared clocks. WEM owns typed channel
declarations on the model and typed program tracks on controller controls.
Each declaration carries its target and initialization; each track refers to
that declaration and owns its curve. The initialization and an actor payload's
baseline remain separate authored values. WEM controller tracks are authoritative;
they are not duplicated into the common controller channel list.

Pose stages and their animation sources have scoped identities within the model's
pose pipeline. Stage controls address those identities, rather than preserving
array indices as editable relationships. Simulation controls similarly select an
actual body, rig, cloth, soft-body effect, or lump within its owning physics system.
Their exact scalar, boolean, vector, and integer value domains remain distinct.

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
