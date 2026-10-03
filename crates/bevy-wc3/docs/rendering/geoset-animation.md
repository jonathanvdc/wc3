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
Bevy's Blend shader path to preserve source alpha for the custom blend factors;

## Material ownership

Each geoset with an animation record receives private material handles for all
its layers, even for static tint. Instances own separate materials and clocks.
Meshes and bind poses remain shared. Bone GeosetAnimId is not used to
apply material tint: the GEOA record's GeosetId selects the geometry.

## Shading and shadow behavior

Tint is a linear multiplier before Bevy lighting and tonemapping. Unshaded mesh
layers use Bevy unlit shading. The DropShadow flag is decoded but does not
create a dedicated shadow pass. Layer filter modes and render ordering follow
the mesh material pipeline described in the [renderer overview](renderer.md).
[Reforged materials](reforged-materials.md) describes UV animation and HD surface
controls.
