# Geoset animation

[Documentation index](../README.md)

## Evaluation and rendering

Geoset animation records bind through their original `GeosetId`, including when
preparation skips empty or non-default LOD geosets. Static and animated alpha and
color apply to every material layer of the geoset. Color is enabled by the COLOR
flag; disabled or absent color uses white. Sampling uses the existing sequence
interval, interpolation, and global-sequence clock. When sampling cannot find a
value, the stored base is used, or white/full opacity when no base exists.
Sequence changes resample the tint instead of retaining the previous color.

## Color and opacity

The public `GeosetAnimation` color is RGB. The MDX codec swaps only the fixed
BGR color on read/write; KGAC keys and tangents remain RGB, as does MDL color.
Tint enters Bevy as a linear multiplier on the textured PBR base color. Layer
and geoset alpha multiply. Zero combined alpha hides a pass; partial geoset
alpha retains the existing AlphaToCoverage behavior for opaque/masked layers.
Blended and additive passes retain their WC3 blend state. Additive/AddAlpha use
Bevy's Blend shader path to preserve source alpha for the custom blend factors;
Bevy's Add shader path clears alpha and made those passes invisible.

## Material ownership

Each geoset with an animation record receives private material handles for all
its layers, even for static tint. Instances own separate materials and clocks.
Meshes and skinning remain shared/unchanged. Bone GeosetAnimId is not used to
apply material tint: the GEOA record's GeosetId selects the geometry.

## Verification

Missing-key sampling uses the explicit neutral fallback described above.

The checked-in `tests/fixtures/geoset_capture.mdl` has two quads sharing one
material: a static red geoset and a blue-to-green geoset that fades to zero.
Run from the repository root:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/geoset_capture.mdl /tmp/wc3-geoset-captures \
  --times 0,0.5,0.75,1 --fps 60 --eye 0,0,8 --target 0,0,0
```

GPU captures inspected at these times verify channel order, cyan midpoint,
partial fade, disappearance, and stable red geometry. Temporary variants reuse
this fixture with Transparent, Blend, Additive, and an extra AddAlpha material
layer, at the same camera and 60 FPS. They verify color/opacity propagation,
including additive fades with preserved source alpha.

Scarlet_footman.mdx under `data/hive-workshop-models/Definitive edition Scarlet
Footman AoW/Definitive edition Scarlet Footman AoW/` was captured at 0.25 and
1 second, sequence 0, 60 FPS. The visible textured mesh remains intact; several
external textures are unavailable, so this is a loading/rendering smoke check,
not a game-color fidelity check. Exact Warcraft appearance remains unverified.

Unit tests cover interpolation, sequence switches, global clocks, missing-key
fallback, disabled color, shared-material isolation, independent instances,
multiple layers, and alpha multiplication. Codec tests check the fixed BGR wire
bytes independently of round trips. Optional corpus round-trip checks encounter
a pre-existing byte mismatch on VashjHighborn.mdx.

## Implementation limits and game comparison

Exact Warcraft lighting, gamma/color-space behavior, filter-mode fade semantics,
and pass ordering remain unverified. PBR lighting and tonemapping affect the
observed tint, and Unshaded mesh flags select Bevy unlit rendering. The DropShadow
flag remains decoded but has no dedicated game-equivalent shadow behavior.
Geoset UV animation and HD surface controls are described in
[Reforged materials](reforged-materials.md). Skeletal deformation still has the
general renderer limitations.
