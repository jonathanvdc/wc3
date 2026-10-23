# Reforged surfaces

DefaultUnit and Crystal materials combine authored HD inputs within Bevy's PBR
pipeline. They share role preparation and animation while retaining distinct
surface equations. This guide describes their roles and remaining shading limits.

A WC3 `MaterialStep` keeps pass state separate from its typed shader inputs.
`Shader::Classic`, `Shader::ReforgedDefault`, `Shader::ReforgedCrystal`, and
`Shader::Unresolved` distinguish
their interpretation. `MaterialInputs` supplies the primary texture, opacity,
and emissive gain; `ReforgedInputs` adds HD texture roles and Fresnel controls.
Native import chooses the shader, with numeric layer selection taking precedence
over the material shader name.

Modern native classic layers can carry mandatory neutral Fresnel storage and a
canonical diffuse slot zero. Import normalizes those fields into the classic
primary input, preserving its texture animation; export restores diffuse-slot
storage for versions 1100 and later. Nondefault or animated Fresnel, duplicate or
unknown slots, and conflicting legacy bindings remain unresolved rather than
losing authored information.

In versions 900 and 1000, a material named `Shader_HD_DefaultUnit` stores diffuse,
normal, ORM, emissive, team, and environment maps in successive native layers.
Import combines these into one semantic step, retaining each role's animated
texture selection. The first layer supplies draw state, opacity, UV animation,
emissive gain, and Fresnel; the remaining layers supply images rather than draw
passes. Modern layouts retain their explicit per-layer shader and texture slots.
A legacy material named `Shader_HD_Crystal` groups exactly six layers in the same
role order into one Crystal pass, preserving first-layer state and each image
track. Other layer counts remain independent passes. Modern numeric shader 24
selects Crystal directly. Unknown shaders retain a diagnosed diffuse fallback;
several textures alone do not establish either HD interpretation.

## PBR integration and roles

The extension preserves Bevy forward PBR, dynamic lighting, shadows, fog controls,
and post-processing. It registers local shader assets without replacing Bevy's
global pipeline. Compiled HD opaque surfaces use forward rendering; forcing them
through deferred shading is unsupported.

| Role | Interpretation |
| --- | --- |
| Diffuse | Color and alpha, multiplied by surface and mesh tint/opacity. |
| Normal | Linear RG tangent normal; positive Z reconstructed with a clamped square root. |
| ORM | Linear green perceptual roughness, blue metallic, alpha team mask. Red is DefaultUnit occlusion or Crystal diffuse-displacement mask. |
| Emissive | Color multiplied by animated emissive gain and mesh tint. |
| Team | Diffuse multiplies white-to-team color according to ORM alpha. |
| Reflection | DefaultUnit equirectangular environment approximation. Crystal retains and exports this declaration but does not consume/request its image. |

Reflection is an approximation of authored native shading. Bevy world directions are
mapped into the authored environment convention. Missing normal/ORM roles retain a
rough nonmetallic base; absent team/emissive/environment maps contribute nothing.
Ordinary Bevy reflectance remains active on shaded surfaces.

## Images and animated coordinates

Role-aware `TextureRequest` values request linear normal/ORM images and sRGB color
images unless an explicit semantic declaration overrides that choice. The loader
creates separately labeled sampler/color-space variants without modifying source
images. The same resource can safely appear as both color and numeric data. Custom
resolvers must return images with the requested interpretation. Reforged models
also need matching HD texture variants: `_hd.w3mod` and `_de.w3mod` can contain
different atlases at the same logical path. Color-space selection does not choose
a game-content variant; that choice belongs to the resource provider.

Role texture choices, gain, Fresnel controls, and UV transforms animate through the
same semantic programme as nodes. Each layer selects its authored coordinate set.
Equal compiled coordinates share geometry, exact differences retain distinct meshes,
and authored tangents retain their source basis. Missing requested coordinates
produce compilation errors.

The native emissive-gain header admits NaN as an unspecified unity default.
Semantic import resolves that header to one and reports the representation
normalization while retaining its complete animation. Other nonfinite values
and invalid animation keys remain rejected.

DefaultUnit Fresnel contributes an authored fifth-power rim with color/team influence, opacity,
and mesh tint, separate from Bevy's physical surface Fresnel. Common unshaded
lighting and unfogged controls follow their Bevy equivalents. The independent
native layer `Unlit`, U/V wrap, `BackFacesForShadows`, and `AmbientOcclusion`
declarations are retained for native export but have no renderer interpretation
yet. Compilation reports that these declarations require an additional interpreter;
their game behavior remains unverified. Surface properties sample the
destination directly during node-pose transitions.

## Crystal surface composition

Crystal samples the same diffuse texture at the primary UV and at an offset.
ORM red mixes displaced RGB toward center RGB; center alpha is retained. The
offset is `.02 * geometric_view_normal.xy / abs(geometric_view_normal.z)`.
Geometric normals follow Bevy's inverse-transpose/skinning path, normalized per
vertex and interpolated before conversion to the current view. Normal-map bumps
affect shading and rim response, never the diffuse offset. The denominator is
clamped to `.0001` for finite grazing behavior; both that safeguard and Bevy's
normal transform are explicit approximations to the source-derived convention.

Crystal uses raw RG normal reconstruction and keeps ORM red out of PBR occlusion.
Team masking, roughness, metallic and emissive remain ordinary HD inputs at the
center UV. Its squared rim uses the shading normal and view direction. Authored
Fresnel opacity is remapped by the displacement mask before rim weighting:
`q = mix(.2, opacity, mask) * (1 - clamp(dot(N,V),0,1))²`.
The lit Bevy result mixes toward the authored Fresnel/team response by q, while
alpha is `clamp(center_alpha * (surface_vertex_opacity + q),0,1)`.
DefaultUnit keeps its existing fifth-power response. Crystal's retained sixth
legacy image is not a separate environment sample; scene IBL remains Bevy-owned.

One shared Crystal preparation function supplies main color and depth/shadow
coverage, including center alpha, normal mapping and rim opacity. Depth-only
variants retain the necessary normal/tangent frame; they omit the displaced RGB
fetch. Both ordinary uniform and bindless material bindings use the same function.
The ordinary ModelMaterial mesh pipeline is the supported route; meshlet-resolved
depth layouts do not provide this full frame and are not fidelity-validated.

These are intended Crystal material equations established from rendering-source
behavior. They are not a claim about every native client version's shader-24
routing, nor a replacement for Bevy lighting, shadows and post-processing.

## Passes and limits

HD Transparent uses a 0.75 alpha cutoff. Forward and supported prepasses retain
matching alpha/UV/skinning interpretation, including eight influences. Normal
prepass reconstructs the same RG normal. Independent blend/depth specialization
preserves authored states. A default extension used directly with an application's
StandardMaterial leaves its base blend/depth policy intact.

Priority is a Bevy sort/depth bias. Native primitive sorting is not applied to mesh
triangles, and a dedicated native mesh/effect ordering scheme is absent. Unknown
shader programmes and facial animation require additional interpreters. External
programme coverage is described separately in the [programme guide](popcorn.md). Implemented DefaultUnit behavior remains an authored PBR
interpretation, not a guarantee of exact game lighting.
