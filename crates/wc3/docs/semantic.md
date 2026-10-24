# Editing model graphs

Use `wc3::model::Model` to work with hierarchy, geometry, materials, and animation
as a checked graph. Use [native documents](models.md) instead when you need to
preserve encoded records, unknown chunks, or file organization. A graph contains
supported authoring values and can borrow compatible arrays from a native document;
it does not keep that document as its backing representation. MDLX and WMO properties
expose named behavior without residual `unclassified` or `unmapped_flags` fields.
Native bits formerly stored in those fields are discarded during semantic import
and are not reconstructed on export.

The usual workflow is to [import](conversion.md), edit through a builder, then
plan a native export or [evaluate the result](evaluation.md). This guide introduces
the relationships and editing rules shared by those tasks.

## Choose a profile

`Model<'data, P>` uses a profile to select the values you can author. `Mdlx`, `M2`,
and `Wmo` retain their model family's geometry, materials, effects, metadata, and
animation rules. `Universal`, the default, accepts all built-in families. Choose
the native profile for a format-specific editor and Universal for tools that need
to handle several families through one API.

A profile selects authoring behavior; a native layout such as `mdlx::V800` selects
what a particular file revision can encode. Export checks the latter separately.
Widening preserves values and identities. Narrowing checks that every value already
fits the destination profile. A [conversion policy](conversion.md) can explicitly
rewrite unsupported meanings before that exact projection.

```rust
use wc3::model::{Error, Model};
use wc3::model::profiles::{Mdlx, Universal};

let mut draft = Model::<Mdlx>::builder("Example");
let root = draft.add_named_node("Root")?;
let model = draft.finish()?;
let broader: Model<'_, Universal> = model.widen();
let restored = broader.try_narrow::<Mdlx>()?;
assert_eq!(restored.node(root).unwrap().name, "Root");
# Ok::<(), Error>(())
```

Universal construction requires an explicit `ModelMetadata` variant. Native
profiles with authored names also accept a name string as shorthand. Metadata
contains model-wide declarations such as bounds, blend time, and external animation
references where the native family supports them. Read it with `metadata()` and
edit it with the builder's `metadata_mut()` or `set_metadata()`. Model bounds
are optional for every native profile. M2 model and sequence empty-extent sentinels
become `None`; real extents still require finite, ordered bounds and a nonnegative
radius. M2 meshes derive conservative rest-position bounds when model bounds are
absent. Semantic M2 export writes one canonical empty sentinel for absence. Use
a native document when preserving the original sentinel representation matters.

## Follow the graph

Nodes define placement and transform inheritance. Meshes own geometry, and mesh
instances place that geometry at nodes. Each instance has identified draws that
select mesh-owned subsets. Looks assign a material to each draw. Features add
lights, cameras, attachments, and emitters to nodes; systems hold relationships
that span several objects, such as physics constraints or world portals.

IDs belong to their model, and mesh-local IDs belong to their mesh. Equal numeric
indices from another owner cannot satisfy a reference. IDs remain stable through
table edits, widening, and checked narrowing; native output assigns its own indices.
Cloning a graph preserves its IDs while allowing independent edits.

Each owner exposes its profile-specific authored data through `properties`.
`NodeProperties`, `MeshProperties`, `InstanceProperties`, `DrawProperties`, and
`ClipProperties` select those payloads in `Profile`; the native definitions live
beside their profile. For example, `mdlx::MeshProperties` contains selection
behavior and sequence bounds, while `m2::MeshProperties` describes sorting and
culling frames. These are properties of the mesh, separate from vertex geometry.
A WMO placement owns its `wmo::Region`, including links to lights and portals.
`NoProperties` represents an owner without additional profile data. Universal
retains the complete native property payload where no shared meaning exists.

Collections such as `nodes()`, `meshes()`, and `materials()` support `get(id)`,
`contains_key(id)`, `len()`, and `is_empty()`. Iterate `(Id, &value)` pairs in stable
identity order with `iter()`, or use `keys()` and `values()` for one side.

## Mathematical values and storage

Mathematical values use glam types, re-exported from `wc3::model`: `Vec2` for
UVs, `Vec3` for positions, directions, and RGB components, `Vec4` for RGBA and
component controls, and `Quat` for rotations. Color space and native channel order
remain explicit interpretation at the format and rendering boundaries.

Fixed-width collections use `PackedArray<'data, T>`. Its `PackedValue` mapping
keeps the encoded representation separate from the aligned mathematical value:
`PackedArray<Vec3>` stores three little-endian floats per element and decodes one
`Vec3` at a time through `get` or `iter`. Compatible native arrays borrow their
encoded storage without allocating an aligned vector collection. Keep that source
alive while using the graph. `from_storage` and `into_storage` bridge native
`ZeroVec` buffers; extracting a shared owned buffer copies its bytes, while a
unique buffer moves without rebuilding it. Parsing checks the packed
representation, while model publication checks semantic validity.

`IntoPackedArray` accepts existing packed arrays, arrays, vectors, and slices of
semantic values. Cloning retains borrowed buffers and shares owned packed buffers.
`into_owned` copies borrowed buffers and retains already owned storage without
copying. `set` and `for_each_mut` copy borrowed or shared storage before its first
edit, then reuse the unique owned buffer. Other clones keep their original values.
Variable-length rows use `ValueGroups`, a packed element array
with encoded row offsets. These storage rules apply independently of graph
identity and transactional validation.

Call `Model::into_owned()` when the whole graph must outlive its native document
and input buffers. It returns the same profile with a `'static` storage lifetime,
copying borrowed strings and arrays and moving buffers that are already owned.
It preserves graph and property scopes, every ID, animation targets and clocks,
material assignments, and local physics identities. Ownership conversion does
not encode a file, reimport the model, or validate a different set of meanings.
All four built-in profiles support it:

```rust
use wc3::model::{Model, profiles::M2};

fn retain(model: Model<'_, M2>) -> Model<'static, M2> {
    model.into_owned()
}
```


## Declare resources once

`dependencies()` groups external images, models, effects, physics, and bone
overrides in typed tables. Reusing a resource handle shares a declaration; adding
an equal declaration creates another entry. Materials remain graph objects.
Each texture input owns its sampling choices and coordinate transform, so two
inputs can share an image while using different wrapping or animation.

```rust
use wc3::model::{AddressMode, Error, MdlxImageResource, Model, Texture};
use wc3::model::profiles::Mdlx;

let mut draft = Model::<Mdlx>::builder("Shared image");
let image = draft.add_image_resource(MdlxImageResource::Path(b"body.blp".as_slice().into()));
let first = Texture::new(image);
let mut second = first;
second.sampler.u = AddressMode::Clamp;
assert_eq!(first.resource, second.resource);
let model = draft.finish()?;
assert_eq!(model.dependencies().images().len(), 1);
# Ok::<(), Error>(())
```

The profile determines the resource identities it accepts: MDLX images support
paths and replacement categories; M2 also supports file-data IDs. Universal image
resources use `ImageResource`, while other dependencies use `ResourceLocator`.
Relative locators describe paths relative to another locator or the application's
root. Resolution and loading remain application responsibilities.

Use `set_*_resource` to replace a declaration without changing its identity.
Removing a referenced resource requires repairing its users before publication.

## Publish edits atomically

A `ModelBuilder` holds an unpublished draft. `finish()` validates local values,
references, ownership, and profile rules before returning a model. Reserved IDs
allow forward references where supported, but every required definition must exist
by publication time.

Published models are immutable. `Model::edit` stages changes in a builder and
publishes them only after the complete graph validates. A failed edit leaves the
original unchanged. Update related declarations in the same transaction—for
example, removing a draw also requires removing its assignments from every look.

For example, clone the node you want to change and replace it within the edit:

```rust
use wc3::model::{Error, Model};
use wc3::model::profiles::Mdlx;

let mut draft = Model::<Mdlx>::builder("Example");
let root = draft.add_named_node("Root")?;
let mut model = draft.finish()?;
model.edit(|draft| {
    let mut node = draft.model().node(root).unwrap().clone();
    node.name = "Renamed".into();
    draft.set_node(root, node)
})?;
assert_eq!(model.node(root).unwrap().name, "Renamed");
# Ok::<(), Error>(())
```

MDLX and WMO use one complete look, edited with `set_look`. M2 supports alternative
looks, each assigning exactly one material to every draw. `material_for_draw`
uses the explicitly selected look or the default; there is no fallback material
on geometry. `add_instance_with_materials` adds a placement and extends the default
look with its draw assignments.


## Continue with a task

The detailed guides separate common editing tasks from the graph overview:

- [Geometry](geometry.md) covers streams, skinning, subsets, and vertex remapping.
- [Materials](materials.md) covers ordered steps, shader inputs, and sampling ownership.
- [Animation](animation.md) covers value slots, curves, clips, clocks, and events.
- [Features](features.md) covers shared inspection and native effect definitions.
- [Evaluation](evaluation.md) covers pose sampling, deformation, and surface values.
- [Conversion](conversion.md) covers output plans and explicit loss policies.

Custom profiles implement `Profile` and relationship validation through
`ValidateLinks`. Their payloads can reuse the topic modules under `model` while
selecting their own supported combinations. Detachment is optional: existing
profiles need no additional methods. `Model::into_owned` is available when the
selected payloads implement the ownership conversion traits in `model::common`;
those implementations must transform every borrowed payload into its matching
owned type. The `IntoOwned` derive handles declared lifetimes; a generic component
whose type parameters contain borrows needs an explicit `Detached` implementation
that maps those components. The [API reference](https://docs.rs/wc3/latest/wc3/model/)
contains the trait contracts and individual payload definitions.
