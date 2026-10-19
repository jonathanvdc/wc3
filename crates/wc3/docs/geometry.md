# Editing geometry and draw assignments

A mesh owns geometry; instances place it in the hierarchy. Draws choose parts of
the mesh, and looks assign their materials. This guide explains those relationships
and the common editing operations available across [model profiles](semantic.md#choose-a-profile).
To obtain renderer-ready primitive streams or deform a sampled pose, use the
[evaluation guide](evaluation.md#projecting-and-deforming-geometry).

For a common stream replacement or vertex remap, jump to
[common geometry editing](#common-geometry-editing). The sections below explain
the representations and relationships those edits must preserve.

## Geometry representations and profile constraints

`MdlxGeometry` and `M2Geometry` retain distinct constrained APIs built on the
same packed vertex-stream types. M2 construction requires a two-element UV array
and four weighted influence slots per vertex, and exposes no tangent or color
constructor. MDLX supports optional normals and tangents, ordered triangle groups,
and `JointInfluences` with either equal-weight matrix groups or four weighted
slots. Checked constructors establish stream cardinalities and finite values; the
profile types keep unsupported representations out of subsequent authoring operations.

Widening these geometry types produces `GeometryData` directly, without a
source-profile tag or an additional geometry wrapper. The arrays move unchanged,
including borrowed encoded arrays and owned encoded buffers. Narrowing
checks the actual streams before constructing the destination's constrained
type. Geometry with unsupported colors cannot narrow to MDLX or M2. Missing normals, absent weighted rows,
unsupported UV gaps, and ordered primitive groups are rejected wherever the
target cannot retain them. Neither zero-weight influence slots nor matrix-group
ordering are discarded, and equal-weight groups are not converted into weighted
slots.

`GeometryData` stores positions, connectivity sets, typed attributes, and
influences. Each connectivity set can select triangles, strips, polygons, points,
lines, or ordered mixed primitives. Alternative sets are independent selections.

Attributes are keyed by semantic and domain: vertex, corner, face, mesh, or an
independently indexed pool. Insertion rejects duplicate keys; `set` replaces a
definition and `update` applies a checked edit. Common color and UV accessors borrow
these same attributes. Projection prefers the most specific authored domain:
corner, face, vertex, then mesh. Pool cardinality is independent of vertex count.

The influence variants retain absent influences,
variable-length weighted rows, four fixed weighted slots, and ordered
equal-weight matrix groups. Custom attributes can coexist with each influence
representation. Narrowing to a constrained display type rejects custom attributes
and influence representations that the destination cannot retain; even a
variable-length row containing four entries remains distinct from four authored
slots.

Collision geometry also widens to `GeometryData`, retaining
`GeometryKind::Collision` as its authored purpose. Face normals use face-domain
`Normal` attributes, and per-triangle scalar controls use
`CollisionTriangleControl`. Widening and common edits retain this purpose.

WMO also widens to `GeometryData`. Ordered color sets use vertex-domain `Color(n)`
attributes, retaining even empty authored slots on empty geometry. Baked-light
coordinates use `LightmapCoordinates`. `TriangleProperties` retain named
face behavior, including optional material references, query and rendering controls,
and authored selectors. Unclassified native flag bits are discarded. Their material
references participate in graph validation.

WMO face controls, tangent pools, and supplemental strips keep their own domains
and mesh-owned subset bindings. Publication validates these selections. Narrowing
to WMO requires unskinned display triangles, contiguous color slots, and supported
attributes; MDLX and M2 reject those additional domains. Generic projection needs
an explicit translation for triangle properties and indexed pools.

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
example, requires normals and exactly two UV streams. Capabilities guide tools before they submit an edit;
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
additional vertex attributes and WMO
lighting colors and baked-light coordinates. Face and corner attributes and their
scoped identities remain intact. Stream replacements apply after remapping, which
allows a tool to duplicate vertices and move the copies in one checked operation.

The following edit duplicates a shared vertex while retaining its triangle's
corner order. The same operation works for any schema that supports remapping.

```rust
use wc3::model::common::IntoPackedArray;
use wc3::model::{
    GeometryData, GeometryEdit, GeometrySchema, Topology, Vec3, VertexRemap,
};

let geometry = GeometryData::triangles(
    vec![Vec3::ZERO, Vec3::X, Vec3::Y],
    vec![[0, 1, 2]],
)?;
let edited = geometry.edited(GeometryEdit {
    remap: Some(VertexRemap {
        new_to_old: vec![0, 1, 2, 0],
        topology: Topology::Triangles(vec![[3, 1, 2]].into_packed_array()),
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
