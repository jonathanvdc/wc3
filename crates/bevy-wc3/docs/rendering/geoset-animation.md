# Geoset animation

Geoset animation controls the color and opacity of individual pieces of a
model. The renderer applies those tracks across every material layer while
keeping each geoset's tint independent. This guide explains track sampling,
alpha visibility, and material ownership across instances and geometry levels.

## Track sampling

Geoset animation records bind through their original `GeosetId`, including when
preparation skips empty geometry. Records for every authored LOD retain their
original bindings. Static and animated alpha and
color apply to every material layer of the geoset. Color is enabled by the COLOR
flag; disabled or absent color uses white. Sampling uses the existing sequence
interval, interpolation, and global-sequence clock. When sampling cannot find a
value, the stored base is used, or white/full opacity when no base exists.
Sequence changes resample the tint instead of retaining the previous color.

## Color and opacity

The public `GeosetAnimation` color is RGB. The MDX codec swaps only the fixed
BGR color on read/write; KGAC keys and tangents remain RGB, as does MDL color.
Tint enters Bevy as a linear multiplier on the textured PBR base color. Layer
and geoset alpha multiply. Zero combined alpha hides a pass independently of its parent
[LOD visibility group](lod.md); partial geoset
alpha retains the existing AlphaToCoverage behavior for opaque/masked layers.
Blended and additive passes retain their WC3 blend state. Additive/AddAlpha use
Bevy's Blend shader path to preserve source alpha for the custom blend factors.

## Material ownership

Passes with animated layer, geoset, or surface properties retain private material
handles and independent clocks. Constant passes from the same prepared layer
share a handle when their evaluated tint, alpha, and texture state match, including
static geoset tint. Texture binding changes detach the affected pass before
updating its material. Unchanged evaluated values do not mark material assets
modified. Meshes and bind poses remain shared.

For independent application PBR customization, insert `Wc3IndependentMaterial`
on the mesh pass and assign a cloned material asset before editing its settings.
The marker excludes that pass from sharing; authored animation still controls
its tint, opacity, texture slots, and surface properties. Bone GeosetAnimId is not used to
apply material tint: the GEOA record's GeosetId selects the geometry.

## Shading and shadow behavior

Tint is a linear multiplier before Bevy lighting and tonemapping. Unshaded mesh
layers use Bevy unlit shading. The DropShadow flag is decoded but does not
create a dedicated shadow pass. Layer filter modes and render ordering follow
the mesh material pipeline described in the [renderer overview](renderer.md).
[Reforged materials](reforged-materials.md) describes UV animation and HD surface
controls.
