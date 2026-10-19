# Editing model graphs

Use `wc3::model::Model` to work with hierarchy, geometry, materials, and animation
as a checked graph. Use [native documents](models.md) instead when you need to
preserve encoded records, unknown chunks, or file organization. A graph contains
supported authoring values and can borrow compatible arrays from a native document;
it does not keep that document as its backing representation.

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
fits the destination profile; it does not translate shaders or effects between games.

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
edit it with the builder's `metadata_mut()` or `set_metadata()`.

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
`ZeroVec` buffers without rebuilding them; parsing bytes checks their packed
representation, while model publication checks semantic validity.

`IntoPackedArray` accepts existing packed arrays, arrays, vectors, and slices of
semantic values. Cloning a borrowed array retains its buffer; cloning an owned
array copies it. `into_owned` copies borrowed buffers and moves owned ones.
`set` and `for_each_mut` copy a borrowed buffer on its first edit and then reuse
owned storage. Variable-length rows use `ValueGroups`, a packed element array
with encoded row offsets. These storage rules apply independently of graph
identity and transactional validation.


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
selecting their own supported combinations. The [API reference](https://docs.rs/wc3/latest/wc3/model/)
contains the trait contracts and individual payload definitions.
