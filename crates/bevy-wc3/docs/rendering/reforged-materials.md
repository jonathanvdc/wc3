# Reforged materials

Reforged materials combine diffuse, normal, surface, emissive, team-color, and
environment textures with animated controls. The renderer maps supported shaders
into Bevy's material pipeline. This guide explains texture roles, color spaces,
UV animation, render passes, and shader coverage.

## Material selection

`Shader_HD_DefaultUnit` uses one mesh pass per normalized HD layer. Older
version-900/1000 six-layer materials are normalized by `wc3` into texture roles;
SD shader IDs in newer files still use the SD material path. HD Crystal and
unknown shader IDs log a warning and render a diffuse fallback. The fallback uses their diffuse texture.

## Bevy integration

The renderer keeps `ExtendedMaterial<StandardMaterial, Wc3LayerState>` and Bevy's
forward lighting, shadows, fog, decals, and post-processing. It registers local
shaders without replacing any global Bevy shader. WC3 opaque materials explicitly
use forward rendering, including in scenes whose default is deferred. Forcing a
WC3 material's `opaque_render_method` to Deferred is unsupported.

## Texture roles and color spaces

| Slot | Interpretation |
| --- | --- |
| 0 | Diffuse RGB and source alpha; existing layer/geoset tint and alpha apply. |
| 1 | Linear RG tangent-space normal, reconstructing positive Z with a clamped square root. Authored UV0 tangents are preserved; other UV sets or missing tangents use Bevy's generated tangent basis. |
| 2 | Linear ORM: red occlusion, green perceptual roughness, blue metallic, alpha team-color mask. |
| 3 | Emissive color multiplied by animated emissive gain and geoset tint. Missing maps produce no emission. |
| 4 | Team-color texture, supplied through bitmap or replaceable bindings. Diffuse RGB multiplies `mix(white, team, ORM.a)`; alpha remains diffuse alpha. |
| 5 | Environment color sampled with Z-up equirectangular reflection coordinates, roughness-dependent mip bias, and a metallic/Schlick weighting. This is a reflection approximation. |

Normal/ORM bindings obtain private linear image variants when a source uses an
sRGB format. The source image is never modified, so ordinary Bevy materials can
use the same bitmap. Variants preserve bytes, sampler, format compression, and
mips; they are reused and invalidated by image modification/removal events.
This also applies to per-instance overrides and animated texture choices. Custom
image sources must remain accessible in `Assets<Image>` to create variants
(use `RenderAssetUsages::MAIN_WORLD`, included in the default usage).
The BLP loader honors `ImageLoaderSettings::is_srgb`.

## Animated surfaces and UV sets

Every texture-role track, emissive gain, and Fresnel color/opacity/team-color track
samples the existing sequence/global clocks. UV translation/rotation/scaling
uses the same center-pivot transform as ribbons. Each layer selects its
`CoordinateId` through a shared mesh variant; all available UV sets can be selected. An unavailable nonzero
UV set is a preparation error. Identical UV arrays share a mesh within a geoset
even across different CoordinateIds. Equality is exact; nearby coordinates remain distinct. UV0 with
authored tangents retains its separate basis.

Fresnel currently adds a fifth-power view-angle rim, colored by the stored
Fresnel color/team contribution, multiplied by opacity and geoset tint. It is
separate from Bevy's physical Fresnel. Emissive and environment contributions
enter the lit path before Bevy's post-lighting processing. Unshaded layers use
Bevy's unlit path; Unfogged layers disable Bevy fog. Missing ORM defaults to a
nonmetallic, rough surface; absent team/environment maps have no contribution.

## Render passes

HD Transparent layers use a 0.75 alpha cutoff. The forward, shadow/depth, normal,
and motion-vector passes share diffuse alpha, UV transforms, and skinning. The
normal prepass uses the same RG normal reconstruction as the forward pass.
Existing WC3 blend/depth settings remain. PriorityPlane is an approximate Bevy
material depth/sort bias. Mesh draws use Bevy render phases;
SortPrimsFarZ/SortPrimsNearZ do not reorder mesh primitives.

## Shader coverage

DefaultUnit uses the texture-role and Bevy shading rules above. Team masks
interpolate continuously, emissive gain is linear, Fresnel uses a fifth-power
view-angle rim, and environment color uses reflection coordinates. Crystal and
unknown shader variants render a warned diffuse fallback. Popcorn emitters and
FaceFX playback are separate unsupported features.
