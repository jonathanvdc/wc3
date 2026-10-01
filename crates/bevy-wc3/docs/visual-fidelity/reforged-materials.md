# Reforged materials

[Documentation index](../README.md)

## Implemented behavior

`Shader_HD_DefaultUnit` uses one mesh pass per normalized HD layer. Older
version-900/1000 six-layer materials are normalized by `wc3` into texture roles;
SD shader IDs in newer files still use the SD material path. HD Crystal and
unknown shader IDs log a warning and render a diffuse fallback. They are not
implemented HD shaders.

The renderer keeps `ExtendedMaterial<StandardMaterial, Wc3LayerState>` and Bevy's
forward lighting, shadows, fog, decals, and post-processing. It registers local
shaders without replacing any global Bevy shader. WC3 opaque materials explicitly
use forward rendering, including in scenes whose default is deferred. Forcing a
WC3 material's `opaque_render_method` to Deferred is unsupported.

| Slot | Interpretation |
| --- | --- |
| 0 | Diffuse RGB and source alpha; existing layer/geoset tint and alpha apply. |
| 1 | Linear RG tangent-space normal, reconstructing positive Z with a clamped square root. Authored UV0 tangents are preserved; other UV sets or missing tangents use Bevy's generated tangent basis. |
| 2 | Linear ORM: red occlusion, green perceptual roughness, blue metallic, alpha team-color mask. |
| 3 | Emissive color multiplied by animated emissive gain and geoset tint. Missing maps produce no emission. |
| 4 | Team-color texture, supplied through bitmap or replaceable bindings. Diffuse RGB multiplies `mix(white, team, ORM.a)`; alpha remains diffuse alpha. |
| 5 | Environment color sampled with Z-up equirectangular reflection coordinates, roughness-dependent mip bias, and a metallic/Schlick weighting. This is an approximation, not verified game IBL. |

Normal/ORM bindings obtain private linear image variants when a source uses an
sRGB format. The source image is never modified, so ordinary Bevy materials can
use the same bitmap. Variants preserve bytes, sampler, format compression, and
mips; they are reused and invalidated by image modification/removal events.
This also applies to per-instance overrides and animated texture choices. Custom
image sources must remain accessible in `Assets<Image>` to create variants
(use `RenderAssetUsages::MAIN_WORLD`, included in the default usage).
The BLP loader now honors `ImageLoaderSettings::is_srgb`.

Every texture-role track, emissive gain, and Fresnel color/opacity/team-color track
samples the existing sequence/global clocks. UV translation/rotation/scaling
uses the same center-pivot transform as ribbons. Each layer selects its
`CoordinateId` through a shared mesh variant; this supports arbitrary available
UV sets rather than truncating selection to Bevy UV0/UV1. An unavailable nonzero
UV set is a preparation error. Identical UV arrays share a mesh within a geoset
even across different CoordinateIds. Equality is exact; nearby coordinates remain distinct. UV0 with
authored tangents retains its separate basis.

Fresnel currently adds a fifth-power view-angle rim, colored by the stored
Fresnel color/team contribution, multiplied by opacity and geoset tint. It is
separate from Bevy's physical Fresnel. Emissive and environment contributions
enter the lit path before Bevy's post-lighting processing. Unshaded layers use
Bevy's unlit path; Unfogged layers disable Bevy fog. Missing ORM defaults to a
nonmetallic, rough surface; absent team/environment maps have no contribution.

HD Transparent layers use a 0.75 alpha cutoff. The forward, shadow/depth, normal,
and motion-vector passes share diffuse alpha, UV transforms, and skinning. The
normal prepass uses the same RG normal reconstruction as the forward pass.
Existing WC3 blend/depth settings remain. PriorityPlane is an approximate Bevy
material depth/sort bias; a dedicated Warcraft pass-ordering scheme remains
missing, including SortPrimsFarZ/SortPrimsNearZ semantics.

## Remaining fidelity work

The implementation uses continuous team-mask interpolation and Bevy lighting.
Emission uses a linear gain, Fresnel uses a fifth-power view-angle rim, and
environment sampling uses reflection coordinates. These are explicit renderer
semantics; their equivalence to Warcraft III remains unverified.

Scarlet Footman in `data/` contains SD and HD materials, six-role HD bindings,
a team-color replaceable bitmap, animated layer alpha, authored tangents, and
DDS normal/ORM maps. This exercises shader-ID dispatch and the one-pass role
mapping. Its missing game texture references prevent a complete appearance
comparison. Explicit bind poses are still ignored by the general renderer;
HD lighting, normal channel conventions, mirrored/nonuniform skin transforms,
Fresnel, environment layout, color spaces, alpha fades, and pass ordering still
need Warcraft III capture comparisons. Bevy PBR integration is implemented;
exact game fidelity is unverified. Crystal, other shader variants, Popcorn, and
FaceFX remain outside this implementation.

## Verification

Unit tests exercise UV-set selection and mirrored tangent handedness, authored
tangent preservation, missing-coordinate errors, slot animation/seeking, global
emissive animation, Fresnel sampling, independent instances, overrides, and
private linear image reuse. The existing geoset and instance tests remain in
place.

The checked-in `tests/fixtures/hd_capture.mdl` has two tinted, skinned quads with
asymmetric normal/ORM textures, animated emission/UV translation, and geoset
alpha. Capture with:

```sh
cargo run -p bevy-wc3 --example capture -- \
  crates/bevy-wc3/tests/fixtures/hd_capture.mdl /tmp/wc3-hd-captures \
  --times 0,0.5,1 --fps 60 --eye 0,-3,5 --target 0,0,0 \
  --prepasses --bevy-reference
```

GPU captures at those times were inspected: normal/material boundaries move
with UV translation, emission changes, the second geoset disappears at its
alpha endpoint, and an ordinary Bevy StandardMaterial sphere remains intact.
Depth/normal/motion prepasses and shadow pipelines compile with HD materials.
This does not independently establish the numerical accuracy of every prepass
attachment or shadow silhouette. The same camera/FPS with the Classic geoset
fixture checks the SD path. Footman captures at 0.25 and 1 second verify real
HD mesh loading and rendering, with several missing external texture warnings;
they are not game-fidelity evidence.
