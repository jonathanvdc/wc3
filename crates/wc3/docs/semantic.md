# Editing semantic model graphs

`wc3::model` describes model meaning independently of file layout.
Use it to inspect relationships, author changes, or prepare a native export.
The [evaluation guide](evaluation.md) describes pose sampling, editable draw
projection, and surface evaluation for offline tools and renderer integrations.
The MDLX, M2, M3, WMO, and Diablo codecs remain the APIs for their encoded records
and supporting documents. A semantic graph contains actual authoring values;
it does not retain a native model as its backing representation.

## Objects and dependencies

Hierarchy nodes describe placement, transform inheritance, and camera-facing
behavior. Meshes describe geometry, while mesh instances place that geometry at
a node. Each instance's draws select mesh-owned draw subsets and rendering controls.
Looks assign materials to those draws. A complete material
surface owns its rendering and sampling behavior, including the family's layers
or passes. Each texture input owns its coordinate transform, including animated
translation, rotation, and scale before sampling. These values belong to the
material input rather than the hierarchy or a shared transform table.

Attached features give a node behavior such as a camera, light, or emitter.
A node can anchor several independently identified features. Systems own
relationships that span several objects, such as physics constraints, cloth,
or world groups and portals. Model metadata contains declarations that apply to
the complete model. Additional controls belong directly to their actual owners,
so understanding a node or draw does not require consulting a parallel extension
table.

IDs identify objects within their owning model. An ID from
an unrelated owner cannot satisfy a relationship. Graph identity is independent
of native record ordering, so output codecs can reorder tables without changing
which objects refer to one another.

`Model<'data, P>` owns its graph as typed `Table<K, V>` collections. The key
parameter identifies an object family independently of its payload's profile and
borrowed lifetime. Collection accessors such as `nodes()`, `meshes()`, and
`materials()` expose immutable lookup through `get(id)`, membership through
`contains_key(id)`, and size through `len()` and `is_empty()`. Call `iter()` to
traverse `(Id<K>, &V)` pairs in stable identity order, or iterate the collection
directly in a `for` loop. `keys()` and `values()` provide the corresponding views.
Edits remain operations on a `ModelBuilder`, where object-specific checks and
complete graph validation apply.

External declarations are grouped under `model.dependencies()`, with typed
`images()`, `models()`, `effects()`, `physics()`, and `bone_overrides()` tables.
Materials and meshes remain graph objects with their own IDs. A material or
effect input refers to an `ImageResourceId` and owns its sampling behavior.
Inputs can share an image declaration while choosing independent addressing,
coordinate transforms, and color interpretation. `Texture` is an inline sampled-image
value used by MDLX and M2 inputs, including complete animated image selection;
it has no graph identity. M3, WMO, and Diablo IV shader inputs reference image
resources directly and compose their own sampling vocabulary. Child placements and
skyboxes use `ModelResourceId`, while external effect instances use
`EffectResourceId`. Each handle resolves only in its owning model: for example,
`model.dependencies().images().get(image)` resolves an image declaration.
Singular model helpers such as `image_resource(image)` remain convenient for
individual lookups.

Declare dependencies through `ModelBuilder::add_image_resource` and the
corresponding methods for other kinds. Reusing a handle explicitly shares a
declaration; adding an equal declaration creates an independent entry. The
`set_*_resource` methods replace shared declarations without changing their IDs.
Removing an entry leaves existing references invalid, so publication rejects the
draft until those references are repaired. Model edits are transactional, and
cloning preserves IDs and borrowed storage while creating independently editable
tables. Handles remain stable across table edits and profile conversions; native
codecs assign their own output indices.

Resource payloads belong to the profile: `Profile::ImageResource`,
`ModelResource`, `EffectResource`, `PhysicsResource`, and `BoneOverrideResource`
select the admitted declarations. For example, MDLX images admit raw paths and
replacement categories, while M2 images also admit file-data IDs. Unsupported
resource kinds use `Unavailable`, making their tables necessarily empty.
Universal images use `ImageResource`; external dependencies use
`ResourceLocator`, which excludes image-only replacement categories and embedded
bytes. Relative locators describe addressing expressions relative to another
locator or the caller's root, rather than edges between table entries. Resolution
remains an application responsibility.

```rust
use wc3::model::{Model, MdlxImageResource, Texture};
use wc3::model::profiles::Mdlx;

let mut draft = Model::<Mdlx>::builder("Shared image");
let image = draft.add_image_resource(MdlxImageResource::Path(b"body.blp".as_slice().into()));
let first = Texture::new(image);
let mut second = first;
second.sampler.u = wc3::model::AddressMode::Clamp;
assert_eq!(first.resource, second.resource);
let model = draft.finish()?;
assert_eq!(model.dependencies().images().len(), 1);
Ok::<(), wc3::model::Error>(())
```

`Profile::ModelMetadata` selects declarations belonging to the complete model.
Read them through `Model::metadata` and replace them through
`ModelBuilder::set_metadata` in a checked edit. For example, MDLX model metadata
contains the external animation filename, sequence blend time, and face-effect
resources. WMO doodads retain external model references and placements; resolving
those dependencies remains the application's responsibility.

## Choosing an authoring vocabulary

`Model<'data, P>` uses `P` to select the permitted authoring vocabulary. Built-in
profiles in `model::profiles` include `Mdlx`, `M2`, `M3`, `Wmo`, `D3`, and `D4`.
The default profile, `Universal`, accepts all built-in families. A profile selects
complete geometry and joint-binding schemas, material surfaces, attached
features, systems, dependency declarations, appearance assignments, animation programs, and the controls
carried directly by each owner.

The shared definitions live directly in topic modules: `model::materials`,
`features`, `scene`, `controls`, `geometry`, `systems`, `appearance`, `animation`,
and `values`. Profiles select those definitions and apply their admissibility
rules. The root model namespace reexports common
building blocks; the profile namespaces expose their chosen compositions.
Choosing a profile therefore constrains authoring without introducing a second
copy of the same data structure.

Parameters express independent choices within a definition. For example,
`AmbientDiffuseLight<Kind, Activation>` selects the illumination selector and
activation domain; `AimedCamera<FieldOfView, Convention>` selects static or
animated projection and its controls. `SocketDefinition<Identity,
Visibility, Controls>` separates identity from visibility and unresolved controls.
`ParticleComposition<Emission, Motion, Lifetime, Appearance, Details>` combines
complete emitter components. Narrow profiles choose exact types for these
parameters; Universal chooses explicit alternatives that admit more behavior.
No optional field is needed merely because another profile uses a different
interpretation.

A profile chooses a required controls payload where absence has no meaning,
`Option<Payload>` where the declaration is genuinely optional, or the zero-sized
`NoControls` value where that owner has no additional controls. `Unavailable`
prevents constructing an unsupported payload family. These distinctions apply
to the actual types rather than a uniform outer option on every owner. M2 uses
`Option<m2::ShadowControls>` for draw controls. Universal controller controls use
`Option<m3::ControllerControls>` because M3 is the built-in controller vocabulary.
Unresolved Diablo shader codes use distinct numeric newtypes, preserving their
producer values without an artificial `Unknown` branch. Embedded image resources carry
bytes whose encoding interpretation remains unspecified.

A semantic profile and a native dialect answer different questions. `Mdlx`
permits Warcraft III authoring behavior. A native dialect such as `mdlx::V800`
selects record layouts that can encode some of that behavior. Choosing a semantic
profile does not guarantee that every native revision supports every authored
value. Format-specific imports and exports use their concrete profile; translating
an incompatible vocabulary requires a separate representability decision.

```rust
use wc3::model::{Model, Node};
use wc3::model::profiles::{Mdlx, Universal};

let mut draft = Model::<Mdlx>::builder("Example");
let root = draft.add_node(Node::named("Root"))?;
let model = draft.finish()?;
let broader: Model<'_, Universal> = model.widen();
assert_eq!(broader.node(root).unwrap().name, "Root");
let restored = broader.try_narrow::<Mdlx>()?;
assert_eq!(restored.node(root).unwrap().name, "Root");
Ok::<(), wc3::model::Error>(())
```

Widening preserves graph identities, meanings, and encoded array storage. Its
`Extends` bound proves that the destination vocabulary can embed every source
payload. The separate `try_narrow::<Target>()` operation checks an exact
projection from `Universal`. It consumes the model, moves owned resource
paths, and retains scoped identities and borrowed arrays. Each owner must already
carry a complete payload supported by the target; missing required metadata or
controls and foreign behavior return an error. Publication checks the complete
model against the target profile's relationship and authoring rules.

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

There is no separate Portable or Generic profile. Format-independent features and
checked simulation arenas live under
`model::scene`, within the universal vocabulary. Their scoped shape, body, and
joint identities cannot be substituted across simulation systems.

Applications can implement `Profile` with their own payloads. Relationship
traversal through `ValidateLinks` makes those payloads participate in checked
publication. M2 and M3 publication requires a bounding declaration; MDLX and
historical WMO retain optional bounds. A named MDLX model or one with animation-file
or blend-time controls requires its model-info declaration. WMO bounds use an
authored axis-aligned box with a sphere radius derived from its half-diagonal.
Image inputs retain sampling declarations permitted by their profile. M2 can
preserve an alternate filename alongside a replaceable-image or file-data-ID
selection; that filename is another shared image dependency. Unknown numeric
sampling selectors stay within their shader vocabulary. Common `ColorSpace` and
`Sampler` values describe understood transfer functions and U/V addressing,
while profile validation rejects choices with no established native mapping.
MDLX layer bits historically labeled wrap hints remain unresolved layer controls;
they do not provide another sampler authority. Surfaces additionally implement `SurfacePayload`, attached
features implement `FeaturePayload` and `FeatureSchema`, and systems implement
`SystemPayload`. Geometry schemas define their full attribute and connectivity vocabulary rather than
attaching arbitrary streams to a narrower native schema.

## Materials, shader inputs, and draw state

A material owns a program whose authored steps describe shader execution or
material composition. Universal's `MaterialSurface` is a
`MaterialProgram<MaterialStep>`, with an ordered `steps` collection. Applications
can reuse `MaterialProgram<S>` as storage within their own surface schema, with
their own step vocabulary and validation.

`MaterialStep` selects the dialect of one operation: `Mdlx`, `M2`, `Wmo`, `M3`,
`Diablo3`, or `Diablo4`. Each dialect lives under the matching
`model::materials` namespace and defines its own `MaterialStep`. For example,
`MaterialStep::M3(m3::MaterialStep::Standard(...))` retains M3 Standard shader semantics;
it does not describe a portable standard shader. A single program can combine
steps from different dialects. Shared texture bindings, blend operations, depth
state, and rasterization components are reused where their meanings agree.
Shader-specific inputs and rendering settings remain together in their concrete
step definitions.

A step can expand into several rendering passes. `m3::MaterialStep::Composite` references
complete materials with independently animated weights and a composition priority.
References remain graph links; compilation expands them recursively without
replacing the authored references or copying their definitions. A Diablo III step
retains shared shader inputs and separate ordered opaque and translucent phase
declarations. A Diablo IV step carries resolved shader inputs and parameters.
An unresolved Diablo IV selection retains its external references on its
appearance variant and uses an empty material program.

`mdlx::MaterialStep` distinguishes `Classic(ClassicMaterialStep)`,
`ReforgedDefault(ReforgedDefaultMaterialStep)`, and
`Unresolved(UnresolvedMaterialStep)`. Classic and Reforged default steps share
`MaterialInputs` for their primary texture, opacity, emissive gain, and draw state;
only Reforged default steps expose additional texture slots and Fresnel inputs.
Unresolved steps retain the complete uninterpreted layer. Numeric layer selectors
take precedence over shader names when interpreting native inputs. Texture slot
order, duplicate selectors, and unknown roles remain intact.

The shader variant is the authority for known shader selection. Known steps carry
neither shader IDs nor shader names, and their native selectors are derived on
projection: the implicit classic shader or the Reforged default selector. Known
native spellings normalize through the semantic program. Unresolved steps retain
their material shader name and layer selector because their interpretation is not
known; the native codecs and native-profile layers remain available for editing
raw encodings.

Each step carries priority, sorting, and additional draw flags in a
`MaterialDeclaration`. Checked MDLX projection requires those declarations to
agree, because the native material stores them once. Unresolved names must also
agree when several unresolved steps share a native material. Known variants can
coexist with an unresolved name through explicit layer selectors. An empty
program carries no shader declarations. Nondefault headers on empty native
materials remain editable through the native codecs; semantic import rejects
them unless an explicit loss policy accepts dropping them.

Diablo appearance variants own external resource selections independently of the
shader material they execute, as described under [appearance and playback](#appearance-and-playback).
Native-profile surfaces convert into the universal program without retaining
another copy. MDLX conversion uses checked `TryFrom` because empty declarations
may be unrepresentable. Checked projections reject unsupported steps or extra
operations rather than dropping them. A native M3 surface requires one matching
shader or composite step; Diablo IV accepts zero or one resolved shader step.

M2 and WMO use shader-specific `MaterialStep` enums in both their native profiles
and Universal. There is no general material input struct between those profiles.
The M2 native profile stores an ordered program of those steps; WMO retains one
step per native material. M2 steps distinguish modulation, additive equations, weighted interpolation,
masked crossfades, facing opacity, and guild composition. WMO steps distinguish
diffuse, reflection, layered, emission, and modulation equations. Each payload
names its active texture roles; a `MaskedDualCrossfadeMaterialStep`, for example,
owns `base`, `first`, `second`, and `alpha_mask` inputs. Guild palettes belong to
the guild payloads. Shared draw declarations remain on each step.

Known steps derive their native selectors from the equation and coordinate
choices. Native encoding aliases normalize to the same definition where their
semantics agree. Unresolved selectors and incompatible resource layouts retain
their raw selection and complete binding order on `UnresolvedMaterialStep`.
Legacy M2 surplus inputs and unused WMO slots remain explicit unused bindings,
preserving graph references without turning them into active shader inputs.
M2 input ordinals remain stable through `inputs()` and `inputs_mut()`, so
existing texture-weight and transform animation targets keep their meaning.

MDLX and M2 animation selectors address the ordinal in `steps`. Reordering steps
therefore requires updating selectors that address their fields. M2's `Material`
is a `MaterialProgram<MaterialStep>`; `properties::MaterialStep(index)` selects
that step's color, opacity, or texture-input properties. The `COLOR`, `OPACITY`,
and `TextureUnit(index)` convenience selectors address the first step. M3
selectors have no step ordinal: a selector must identify exactly one operation in
the program, and publication rejects ambiguous targets.

M2 import sorts each skin section’s batches by native `material_layer`, retaining
source order for ties. Consecutive batches with the same auxiliary section word
become one draw with an ordered material program. Export derives dense layer
numbers from draw and step order within each section. The auxiliary section word belongs to
M2-specific draw controls and is not a geometry reference. Material steps retain
shader inputs and composition state without pointing back to their geometry.

The word layer describes different operations in these dialects. A Warcraft III
draw layer becomes an MDLX step. An M3 texture layer supplies sampling,
coordinates, color processing, and a contribution to one shader input; it does
not add a step. An M3 composite section instead references another complete
material and can expand into several rendering passes. Native profiles retain
these exact authoring schemas. Native MDLX surfaces use `LayeredSurface` and its
`SurfaceLayer` vocabulary; M2 inputs implement `ShaderInput` for property selection,
local validation, and graph traversal.

Texture inputs compose through `TextureBinding<Image, Coordinates, Sampling,
Contribution>`. The selected image representation can retain animated texture
choices, runtime replacement categories, external resource identities, or explicit
absent-image codes. `Coordinates<Source, Transform>` separates mesh UV or generated
coordinate selection from its input-owned transform. Animated TRS values,
Diablo matrices and parametric animation, and M3 UV programs retain distinct
types because their behavior differs. Native transform conventions and units
are preserved. MDLX transform properties address the owning material layer,
so editing one imported use does not affect another use of the same native record.

Native import expands texture and transform table references into complete
input values while sharing image declarations. Animated MDLX image choices carry
the selected image's sampler and native options with each key. Export rebuilds
native indirection tables and interns equal records; native table identity and
unused records are not semantic authoring objects. `Model::sampled_images()`
collects the distinct inline sampled-image values currently used by materials,
effects, and animation channels rather than exposing a stored texture table.

Sampling belongs to the input that uses it. M3 bindings retain sampling programs for flipbook, video, and noise,
together with independent channel-selection contributions. Simpler bindings
select smaller components. WMO steps retain two independently authored scrolling
slots even when their image bindings are absent. Diablo matrix rows and unresolved
animation controls retain their authored order. These distinctions do not require
optional fields on every texture binding.

The following M2 step names its modulation input while reusing the shared texture
binding vocabulary:

```rust
use wc3::model::materials::m2::{MaterialStep, ModulateMaterialStep, TextureUnit};
use wc3::model::Animated;

let mut modulation = TextureUnit::default();
modulation.coordinates.source = 1;
modulation.contribution = Animated::constant(0.75);
let step = MaterialStep::Modulate(ModulateMaterialStep {
    modulation,
    ..Default::default()
});
let MaterialStep::Modulate(value) = step else { unreachable!() };
assert_eq!(value.modulation.contribution.default, 0.75);
```

A binding supplies a shader input; a step describes an authored operation.
MDLX layers therefore retain their ordered execution behavior, while M3 texture
layers remain inputs to a shader. `PassState<Blend, Rasterization,
Ordering>` owns draw state. `Rasterization<Depth, Faces, Lighting, Controls>`
selects independent domains, with `DepthState` separating comparison, writes,
and bias. Common operations have shared names, while each composition limits
its admitted blend operations. Explicit blend equations retain their factors,
operation, enablement, and constant color. Unresolved codes and additional
operations remain in their selected domains. M2 rasterization helpers separate
known lighting, fog, face, and depth controls while retaining the remaining flag bits.

`SurfaceContribution<Input, Parameters>` couples an input with its response
parameters. M3 Standard surfaces group diffuse, specular, emissive,
opacity, and normal contributions without replacing their shader model with PBR.
Sparse primary and secondary inputs retain their exact slots. A specular response
keeps its color and gloss roles; normal contributions retain independent masks and
the full authored weight vector.

Shader parameters use `ParameterEntry<Key, Value, Controls>`. Their ordered
collections retain duplicate keys. Diablo tags, Diablo value selectors, and
M3 hashed names remain different identity domains. Opaque words and byte
payloads remain distinct from interpreted scalar or vector values; a bit pattern
that resembles a nonfinite float is valid opaque data. Graph-absence sentinels,
light groups, unresolved parameter types, and independently stored hashes are
preserved. Composition traverses every selected component during graph validation,
including application-defined resource, coordinate, contribution, and parameter types.

## Attached behavior and shared feature inspection

Universal's `NodeFeature` selects behavior directly: lights, cameras, attachments,
particle and ribbon emitters, events, collision shapes, and geometry animation.
It also admits light and external particle programs, particle copies, projectors,
force and deformation fields, attachment and hit-test volumes, billboard and
turret behaviors, inverse-kinematics solvers, and animated bounds. These are
independently identified features anchored to ordinary hierarchy nodes. There is
no outer source-format variant to unwrap before discovering a feature's purpose.

`NodeFeature` is an alias for `FeatureSet<Universal>`. Native feature aliases
select the same enum with `Mdlx`, `M2`, `M3`, or `Wmo` as its vocabulary.
`FeatureVocabulary` chooses each family's payload and supplies its inspection,
property, and relationship rules. Unsupported families select `Unavailable`.
For example, an MDLX particle carries a constrained `ParticleComposition`,
while Universal's particle family also admits resource instances and broader
simulated definitions. The family names describe behavior in every selection:
`Particle`, `Ribbon`, `Attachment`, `Event`, and `CollisionShape`.

Shared structures retain common authored fields while explicit alternatives
preserve different interpretations. `AmbientDiffuseLight` owns independently
animated diffuse and ambient emission and distance attenuation. Its
`LightActivation` distinguishes fractional visibility with optional shadow and
falloff controls from boolean enablement. Specular, world, region-point, and
portable lighting retain their complete definitions as separate illumination
models. Unresolved light-kind selectors retain their original vocabulary, so
changing activation cannot reinterpret an unknown code in another format.
Unresolved controls remain authored values.

`AimedCamera` shares eye displacement, target and target displacement, clipping,
roll, and field of view. `CameraConvention` distinguishes vertical projection
with visibility and photographic controls from diagonal projection with an
application camera role. Widening a static vertical field of view represents it
as a constant animated value; adding a binding prevents exact MDLX narrowing.
Optical camera programs retain their initial/reset values and animation-reference
metadata rather than collapsing them into a simpler rest value.

Attachments compose identity, visibility, and placement independently.
`Socket` exposes a named identity, an application role, or an MDLX point
index; those namespaces remain distinct even when their numeric values coincide.
Its `visibility: Animated<f32>` describes mounted content visibility; constant
`1.0` represents an uncontrolled socket. Native M2 boolean bindings are read as
scalar zero or one through `ScalarBinding::FromBoolean`, preserving the source
identity, shared channels, and clocks. The socket also retains unresolved authored
controls. Exact M2 narrowing requires binary defaults and either an original
boolean source or binary step curves on the shared `socket::VISIBILITY` property;
fractional values and fades are rejected. Native formats without a visibility
control require constant-one visibility. Exact export checks identity namespace,
visibility representability, and control presence
against the destination vocabulary. `ChildModel` combines a resource with an
optional socket and placement appearance. `Attachment::Socket` exposes a socket
without instantiating a child; `Attachment::Child` declares a child reference. MDLX
attachments and WMO doodads therefore share the child model structure, while
world tint and light selection remain an explicit appearance program. MDLX path strings retain their borrowed or owned
storage as byte paths; exact MDLX narrowing requires UTF-8.

`ModelMount` describes where a child follows the carrying model, independently of
resource selection and playback. `Attachment::mount` derives this description
for an authored child reference. Its `MountTarget` selects a node or a socket,
with socket targets retaining authored visibility. A local transform supplies the
offset. `CoordinatePolicy::Host` uses the host's authored axes and units;
`CoordinatePolicy::Child` retains the child's standalone axes and units.

`ParticleDefinition` is Universal's selection of `ParticleComposition`, owning
shared emission, motion, lifetime, and particle-age appearance components.
Native profiles select the same composition with narrower component types.
All simulated particles expose their rate, initial speed, speed variation,
gravity, duration, and lifetime variation through those same fields.
`ParticleScalar` distinguishes rest-value animation from programmed values with
independent initial/reset values and animation-reference metadata. The emission
trigger distinguishes continuous rates, keyed burst counts, and programmed
burst controls. Fractional visibility and boolean activation remain separate.
Directional gravity, animated vector acceleration, and programmed gravity words
retain their distinct interpretations.

`ParticleAppearance` distinguishes resource-defined appearance, staged color,
opacity and scale, arbitrary age curves, and programmed age controls. It is
owned independently of emitter timing and simulation details. The remaining
`ParticleDetails` select the simulation, rendering, and source geometry rules:
resource particles, atlas sprites and tails, age-driven particles, or programmed
effects. Atlas and age-driven definitions group rendering and emission shape. Model-clock animation changes emitter parameters, while particle-age
programs control the appearance of each living particle. Widening does not
convert degrees to radians, reinterpret unknown controls, or bake an appearance
program into a different simulation.

MDLX and M2 ribbons select `TrailComposition` with their exact rate, atlas,
rendering, and physical-control types. They widen to `TrailEmitter`, the
broader Universal selection of that composition. The shared structure owns section
geometry, animated appearance, rendering bindings, and physical controls.
Universal generation rate uses `f64`, which exactly retains either native `u32` or `f32`
values. Checked export requires an integral in-range rate for MDLX or an exactly
representable `f32` rate for M2. Atlas dimensions similarly use a shared unsigned
width, while animated atlas selectors retain their `u16` or `u32` value domain
and structural property identity. Gravity and emission angle retain separate
meanings; layered materials and ordered texture/surface bindings remain explicit
rendering programs.

Programmed particles and ribbons also share `ParticleMotion`, `ParticleLifetime`,
`ProgrammedDirection`, particle-age appearance, force response, and noise
components. Their remaining geometry and simulation controls stay on their own
definitions. Particle copies retain a checked reference to a compatible independent
programmed emitter. Events distinguish named sources, four-character gameplay
identifiers, and structured engine commands.

`FeatureSchema::feature` borrows a `FeatureView` from any profile's feature
payload, including custom enums. Application-specific behaviors can return
`FeatureView::Custom` while retaining their concrete profile vocabulary.
Consumers can discover behavior before widening an entire model. `LightView::ambient_diffuse` and `CameraView::aimed` expose common
borrowed fields across the constrained and universal representations. The views
also retain access to complete definitions for specialized controls.
Particle emission views distinguish continuous rates, keyed bursts, external
program inputs, and programmed rates with independent burst controls. Lifetime
views preserve static seconds, animated seconds, and programmed values. Ribbon
edge-strip views share color, opacity, heights, and section lifetime while
retaining exact generation rates and the atlas selector's original animation
value domain. Inspection neither clones payloads nor materializes their borrowed
curves. After widening, the shared owned components can be edited without selecting a
native payload:

```rust
use wc3::model::profiles::mdlx;
use wc3::model::{Animated, ParticleDefinition, ParticleScalar};

let mut emitter = ParticleDefinition::from(mdlx::Particle2::default());
emitter.emission.rate = ParticleScalar::Animated(Animated::constant(24.));
emitter.motion.speed = ParticleScalar::Animated(Animated::constant(8.));
let exported = mdlx::Particle2::try_from(emitter).unwrap();
assert_eq!(exported.emission.rate.default, 24.);
assert_eq!(exported.motion.speed.default, 8.);
```

Widening moves each feature into its behavior representation. Exact narrowing
checks its current fields and interpretations, preserving resources, animation
bindings, and relationship identities. For example, editing shared diffuse
intensity survives MDLX narrowing; changing boolean enablement into fractional
visibility requires an explicit authoring decision. Unsupported features and
unrepresentable additions return errors instead of being discarded.

## Geometry representations and profile constraints

`MdlxGeometry`, `M2Geometry`, and `M3DisplayGeometry` retain distinct constrained
APIs built on the same packed vertex-stream types. M2 construction requires a two-element
UV array and four weighted influence slots per vertex, and exposes no tangent or
color constructor. MDLX supports optional normals and tangents, ordered triangle
groups, and `JointInfluences` with either equal-weight matrix groups or four
weighted slots. M3 display geometry requires normals and weighted slots, and
permits tangents, colors, and up to five UV slots with meaningful gaps. Checked
constructors establish stream cardinalities and finite values; the profile types
keep unsupported representations out of subsequent authoring operations.
M3 mesh controls can additionally carry an optional `UvEncoding` preference for
region UV packing. Universal controls retain it unchanged; export checks it
against current UV values as described in the [conversion guide](conversion.md).

Widening these display types produces `GeometryData` directly, without a
source-profile tag or an additional geometry wrapper. The arrays move unchanged,
including borrowed encoded arrays and owned encoded buffers. Narrowing
checks the actual streams before constructing the destination's constrained
type. M2 geometry can therefore narrow to M3 display geometry, while M3 geometry
with colors cannot narrow to MDLX or M2. Missing normals, absent weighted rows,
unsupported UV gaps, and ordered primitive groups are rejected wherever the
target cannot retain them. Neither zero-weight influence slots nor matrix-group
ordering are discarded, and equal-weight groups are not converted into weighted
slots.

`GeometryData` keeps positions and authored purpose explicit, with `ConnectivitySets`,
`GeometryAttributes`, and `GeometryInfluences` owning the remaining domains.
Connectivity sets retain primary topology and independently selectable alternatives.
Each set can contain triangles, triangle strips, polygons, points, lines, or an
ordered `Mixed` topology. Alternatives are separate selections, rather than extra
faces appended to the primary topology.
Normals, tangent frames, color and UV slots, custom data, surface records, and
independently indexed pools all use the same attribute collection, keyed by
semantic and domain. There is no separate backing field for a primary color or
UV stream that could disagree with its attribute definition.

Attribute insertion checks value types and rejects duplicate semantic/domain
keys. `set` explicitly replaces a definition, while `update` checks types and
keys before applying an atomic attribute edit. The common stream accessors borrow
the corresponding typed attribute values. `TextureCoordinates` provides an ordered borrowed view
over either profile-owned slots or canonical attributes, preserving empty gaps
and sharing array storage. Attribute domains establish vertex, corner, face,
mesh, or independently indexed correspondence; pool cardinality is independent
of vertex count. Native projection selects the most specific authored domain
when a semantic occurs in several domains: corner, face, vertex, then mesh.

The influence variants retain absent influences,
variable-length weighted rows, three or four fixed weighted slots, and ordered
equal-weight matrix groups. Custom attributes can coexist with each influence
representation. Narrowing to a constrained display type rejects custom attributes
and influence representations that the destination cannot retain; even a
variable-length row containing four entries remains distinct from four authored
slots.

Collision geometry also widens to `GeometryData`, retaining `GeometryKind::Collision`
as its authored purpose. Face normals use face-domain `Normal` attributes, and
per-triangle scalar controls use `CollisionTriangleControl`. Checked M3 narrowing
requires collision purpose, supported connectivity, and correctly typed optional
triangle attributes. It rejects shading and skinning streams; display geometry
types reject collision purpose even when the mesh carries only positions and
connectivity. Widening, common edits, and native projection retain this purpose.

`VertexData` shares the typed vertex layout used by display geometry, WMO,
Diablo III, and collision. Its stream types select fixed UV arrays, optional
streams, multiple color sets, or `()` for unavailable shading data. An explicit
extras type owns additional vertex streams. Common remapping carries those
streams through the same vertex correspondence, while connectivity and joint
rows remain separate fields of their constrained geometry owner.

WMO also widens to `GeometryData`. Ordered color sets use vertex-domain `Color(n)`
attributes, retaining even empty authored slots on empty geometry. Baked-light
coordinates use `LightmapCoordinates`. `SurfaceTriangleControls` retain complete
face behavior, including optional material references, query and rendering flags,
and unresolved selectors and bits. Their material references participate in
graph validation.

Surface records use face-domain `SurfaceControls` attributes with
`AttributeValues::SurfaceControls`. The tangent pool uses a pool-domain `Tangent`
attribute, while secondary strips are a `TriangleStrip` connectivity set. Mesh-owned
draw subsets select these domains on the owning geometry;
neither stream acquires vertex-domain cardinality. Checked publication validates
these selections in both WMO and universal graphs. WMO narrowing requires
unskinned display triangles, contiguous color slots, and supported typed vertex attributes, while retaining face controls
and pools without copying. Other constrained geometry types reject these additional
domains. Generic draw projection requires an explicit domain translation before
handling surface controls, supplemental declarations, or indexed pools.

Diablo III also widens to `GeometryData`. Its independent three-vector tangents and
binormals use the typed vertex attributes `TangentVector` and `Binormal`, auxiliary
colors use `Color(1)`, and cloth mappings use `ClothVertex`. Widening preserves
these streams independently, without deriving a tangent frame or handedness.
Optional auxiliary direction bytes use the named vertex attribute
`d3::DIRECTION_AUXILIARY_ATTRIBUTE` with `AttributeValues::Byte3`, ordered normal,
tangent, then binormal. These values retain their unresolved meanings and follow
vertex remaps alongside the other streams. Its influence rows retain the array type `[Influence; 3]`. Checked narrowing
requires these complete vertex attributes with their exact value types, exactly
two UV streams, and triangle connectivity. Unsupported or duplicate attributes,
tangent frames, and other influence representations are rejected.

Diablo IV uses `GeometryData` and widens without a geometry wrapper.
Narrowing checks triangle connectivity and vertex-domain attributes before
constructing `d4::Geometry`. Weighted rows retain their variable or fixed width
without copying their arrays. Equal-weight matrix groups require an explicit
transformation before entering this weighted geometry vocabulary. Native adapters
also check whether the selected vertex layout can encode the authored streams.

## Draw subsets and instance bindings

A mesh owns reusable `DrawSubset` declarations. Each subset selects a scoped
`ConnectivityId` and a typed `PrimitiveRange`: `Primitives` counts logical
primitives, while `StripIndices` counts authored strip indices. `Mixed` describes
one ordered primitive collection within a set; it does not combine independently
selected alternatives. An instance's `Draw` stores a `DrawSubsetId` and rendering
controls. `Draw::new` creates its stable `DrawId`, retained through edits and
reordering. Looks reference a draw through `(InstanceId, DrawId)`, so several
instances can reuse the same geometry selection with different materials. Draw,
subset, and connectivity identities survive widening and checked narrowing.
Cloning a draw preserves its identity; use `Draw::new` for an independent binding.

`Mesh::add_primary_subset` creates a primitive range in primary topology.
`Mesh::add_connectivity` adds a profile-supported alternative and returns its
mesh-scoped identity. `Mesh::add_subset` accepts an explicit selection and its
bindings. Publication checks ownership, range counting domains, bounds, and the
profile's supported subset operations. A subset from another mesh cannot satisfy
an instance draw, even when both meshes have identical connectivity.

WMO subsets retain their ordinary triangle selection together with optional
strip declarations, vertex windows, and tangent-pool associations. Their instance
draws retain lighting, pass, and other rendering behavior. Import does
not turn a supplemental strip declaration into another draw. Export resolves
subset identities into native batch ranges, and joining world placements relocates
the mesh-owned selections and pool associations together.

`Face` and `Corner` attributes describe primary topology. `ConnectivityFace(id)`
and `ConnectivityCorner(id)` explicitly address another connectivity set on the
owning mesh. Publication checks those identities and domain cardinalities, and
rejects duplicate definitions that alias the same semantic and domain. Native
projection uses the selected set's attributes and preserves strip winding when
expanding strips into triangles.

## Appearance and playback

Appearance schemas retain the family's actual assignment domain. A native
family may use one material assignment, draw alternatives, or named material
slots. Diablo IV owns hashed material slots, slot-local variants, and hashed looks that
select actual variants. These identities remain stable when native tables are
rebuilt. Its geometry selects an actual material slot; it does not duplicate those
assignments in common draw looks. D3 uses the same ownership pattern with named slots and named alternatives.
Both profiles retain unused variants independently of the looks that select them;
neither infers slot identity from mutable material display names.

Each variant's typed `selection` retains its base-material, cloth, effect, and
appearance qualifiers, including when no Diablo IV shader has been resolved.
Use `add_variant_with_selection` to author these assignments and
`slot.variant_definition_mut(id)` to edit a complete variant. `variants()` exposes
variant/material pairs for assignment traversal. Changing the shader material
with `set_variant` preserves the variant's selections, so two variants can share
one shader definition while selecting different external resources.

Draw-based looks are complete assignments, with exactly one material for every
instance draw. Draws and geometry attributes carry no fallback material. Selecting
an explicit look uses only that look; otherwise `material_for_draw` uses the
default. Missing selections or assignments produce errors. Publication rejects
incomplete looks, duplicate assignments, and references to absent draws or materials.
MDLX, M3, and WMO use `SingleLook`, which always contains exactly one look,
initially with empty bindings. That look is always the default; edit it through
`set_look` rather than adding or removing an alternative. MDLX import places each
geoset in one mesh instance with one draw and stores its material in that complete
look.

`ModelBuilder::add_instance_with_materials` inserts a placement and extends its
default look with one material per draw, in the supplied order. When authoring
alternatives, complete each look before publication. `set_look` replaces an
assignment while preserving its identity and selection. Removing a draw likewise
requires removing its assignments from every look in the same checked edit.

Diablo resource identities include a game namespace: a D3 group/ID pair does not
identify the D4 resource with the same numbers. Profile publication checks that
resource identities belong to the selected game vocabulary.

Playback programs belong to their actual clip or controller owners. MDLX
non-global channels use one absolute authoring timeline, and sequences select
intervals on it. Global sequences use shared clocks.

D3 quaternion curves use explicit normalized linear interpolation (`Nlerp`),
which remains distinct from spherical interpolation (`Slerp`). Both are restricted
to quaternion-valued curves. Native exports reject an interpolation mode they
cannot represent rather than substituting another rotation path.

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
field segments and collection ordinals. Shared feature destinations live under `feature_properties::light` and
`feature_properties::camera`. Diffuse and ambient illumination, attenuation, and
camera roll share structural identity where their value and spatial semantics
agree. Native selectors resolve those same identities for exact export. Universal also
admits `camera::VERTICAL_FIELD_OF_VIEW` for vertical aimed cameras; MDLX narrowing
rejects this binding because its field of view is static. `feature_properties::trail`
provides shared tint, opacity, and cross-section extent destinations, while
`feature_properties::particle` exposes continuous rate, lifetime, and initial
speed where those quantities have established shared units. Keyed burst counts
remain distinct from continuous emission rates. Fields
whose meanings differ retain distinct selectors, including camera projection
and boolean enablement versus fractional visibility.
Publication checks owner identities, selector families, existing layer and slot
ordinals, and declared value domains.

`Animated<T, P>` keeps a rest value and an optional typed read binding separate
from playback programs. `AnimatedValue` selects the binding representation for
its value domain through `Binding<T, P>`. Most domains read a `Property<T, P>`
directly. Scalars use `ScalarBinding`, whose `Property` variant reads a scalar
source and whose `FromBoolean` variant reads a boolean source as zero or one.
The source keeps its identity and clocks; the conversion happens when sampling
the binding. Tracks still target direct `Property<T, P>` addresses, so a converted
read cannot become a channel destination.

Built-in owner payloads retain bindings in the universal structural vocabulary
so their complete payloads can move unchanged into containing profiles.
`Animated::new` embeds a supplied direct property when the `Extends` relationship
permits it. Direct assignment to a payload's `binding` field can use
`Some(property.widen::<Universal>().into())`. Custom payloads may instead use
`Animated<T, ApplicationProfile>` to constrain their own bindings directly.
Program channels always carry their model's profile.

Widening maps program selectors into the containing enum while moving the existing
curve arrays. Containing property enums preserve canonical structural identity,
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

Custom value types implement `AnimatedValue` to select their read binding
(usually `Property<Self, P>`), `AsULE` for encoded storage, and `CurveValue` for
curve behavior, selecting `DiscreteDomain`,
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

The array constructors accept separate `ZeroVec` arrays for times, values, and
controls through `IntoZeroVec`. Existing encoded arrays move unchanged; native
slices, arrays, and vectors are encoded into owned storage. Typed `ValueKeyframe`,
`HermiteKeyframe`, `BezierKeyframe`, and `TcbKeyframe` records provide a convenient
authoring interface through the corresponding `*_keys` constructors. Those
constructors materialize separate encoded arrays from the interleaved records.
For example, a quaternion Hermite curve can carry zero derivatives explicitly:

```rust
use wc3::model::{Curve, HermiteKeyframe, Quaternion, Time};

let keys = [HermiteKeyframe {
    time: Time::seconds(0.)?,
    value: Quaternion::IDENTITY,
    incoming: [0.; 4],
    outgoing: [0.; 4],
}];
let curve = Curve::hermite_keys(keys)?;
assert!(curve.values().is_owned());
assert_eq!(curve.hermite_derivatives().unwrap().incoming.get(0), Some([0.; 4]));
Ok::<(), wc3::model::Error>(())
```

## Borrowing and checked edits

Fixed-width semantic arrays use `ZeroVec<'data, T>` directly. Their encoded
storage exposes logical elements through `get` and `iter`, with `AsULE` defining
the element representation. Compatible native arrays can retain borrowed encoded
buffers; conversions between different layouts materialize the logical values
once. The decoded source must outlive arrays borrowing its storage. Cloning a
borrowed array retains its buffer, while cloning an owned array copies its
encoded storage.

Variable-length rows use `ValueGroups<'data, T>`, which stores one flat `ZeroVec`
plus encoded row offsets. `from_rows` materializes authoring rows; `get` and
`iter` return borrowed encoded row views. Matrix groups and variable influence
rows therefore retain their grouping without nested allocations or an abstract
source behind each array.

A `ModelBuilder` collects an unpublished draft. Reserved IDs permit forward
references where supported; `finish` checks their definitions, ownership, and
required relationship kinds, together with local values such as geometry ranges
and curve parameters. It returns a `Model` containing the checked graph,
metadata, and resources. `Model::standalone(name)` creates an empty universal
model when no draft edits are needed; use `Model::<P>::builder(name)` to author
a selected vocabulary.

Published models provide immutable access. `Model::edit` stages graph, metadata,
and texture changes in a `ModelBuilder` and publishes only after the complete
model validates. A failed edit leaves the original model unchanged. Conversion adapters import into
this same model and plan native output directly from it. Typed profile fields
retain source details with an identified owner and participate in checked edits
and exact narrowing. Retain the decoded native document separately when its
unsupported records or original physical layout are needed.

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
geometry unchanged on success or failure. Unchanged arrays retain their values and
borrowed buffers; owned buffers are copied when cloning the geometry. Remapped
streams are materialized as owned encoded arrays.

`edit_capabilities` describes supported common streams, required normals or colors,
UV-set limits, and whether the current geometry permits vertex remapping. M2, for
example, requires normals and exactly two UV streams. Diablo III tangent and
binormal vectors remain distinct profile streams, so its common tangent-frame
operation is unavailable. Capabilities guide tools before they submit an edit;
validation still checks cardinalities, finite values, and the profile's complete
invariants. Universal `GeometryData` exposes the shared representation's broader
editing capabilities: an
M2 mesh widened to `Universal` can acquire tangents, but subsequent M2 narrowing
rejects that added stream. Custom schemas opt in by implementing the same two
methods; their default implementation rejects common edits.

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
use wc3::model::common::IntoZeroVec;
use wc3::model::{
    GeometryData, GeometryEdit, GeometrySchema, Topology, VertexRemap,
};

let geometry = GeometryData::triangles(
    vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
    vec![[0, 1, 2]],
)?;
let edited = geometry.edited(GeometryEdit {
    remap: Some(VertexRemap {
        new_to_old: vec![0, 1, 2, 0],
        topology: Topology::Triangles(vec![[3, 1, 2]].into_zero_vec()),
    }),
    ..GeometryEdit::default()
})?;
assert_eq!(edited.positions.len(), 4);
assert_eq!(geometry.positions.len(), 3);
Ok::<(), wc3::model::Error>(())
```

Use `Mesh::edited` to apply a geometry edit and validate its subset references
atomically. Remapping a subset with vertex windows, supplemental strips, or
tangent-pool bindings requires an explicit correspondence update; the helper
rejects those edits instead of guessing new associations.

To publish an edit, clone the mesh from the draft's model, apply `Mesh::edited`,
and submit the result through `ModelBuilder::set_mesh` inside
`Model::edit`. Geometry editing does not
recompute mesh or model bounds, transform normals automatically when positions
change, or rewrite references held by other graph objects. Tools must update those
related declarations in the same model transaction where needed. In
particular, vertex references in physics systems require their own correspondence
updates; graph validation cannot establish that an in-range index still identifies
the intended vertex.

The common protocol currently preserves topology structure. Adding or splitting
faces, interpolating new vertices, changing skinning representations, and creating
profile-specific attribute layers require explicit profile operations. Shared
geometry with alternative connectivity or indexed pools rejects vertex remapping
because those domains require separate correspondence. The constrained WMO API retains its existing strip
restriction; pool selections need their own correspondence when vertices change.
Ordinary stream edits remain available for those meshes. Native index widths, packing, and
quantization continue to be checked during export.
