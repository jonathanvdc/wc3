# Mesh color and opacity

Semantic mesh color and alpha controls apply across every selected material pass.
MDLX geoset animation imports into these controls without using runtime geoset
indices. Empty geometry, material mesh sharing, and authored LODs therefore do not
renumber their targets.

## Sampling and visibility

The semantic animation programme samples enabled RGB and opacity tracks on the same
clip/shared clocks as node transforms. Disabled color uses white. Mesh tint
multiplies surface color in linear space; mesh and material opacity multiply. Zero
combined alpha hides a pass independently of its LOD or world-group gate.

Partial opacity on opaque/masked MDLX passes uses AlphaToCoverage; blended and
additive passes retain their authored blend factors. Unlit layers retain tint
without scene lighting. Surface and mesh colors switch to destination values
directly during local-pose transitions.

## Ownership

Constant passes share a material only when their compiled state, including static
mesh tint, agrees. Animated bindings retain private assets per instance. Unchanged
samples do not mark material assets modified. Meshes and bind palettes remain
shared. Application edits to a shared material affect every user of that handle;
clone and assign an asset explicitly when independent customization is needed.
Authored animation continues to own its bound properties.

MDLX fixed BGR color storage is handled by its codec; semantic colors and keyed RGB
remain RGB. Bone GeosetAnimId does not select material tint. Dedicated DropShadow
passes are not generated. See [materials](reforged-materials.md) for HD
role-specific use of mesh tint.
